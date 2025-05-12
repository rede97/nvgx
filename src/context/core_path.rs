use std::ops::Range;

use crate::{DrawPathStyle, RendererDevice};
use crate::{Instances, Paint};

use super::core_path_cache::PathRefWithCache;
use super::*;

impl<R: RendererDevice> Context<R> {
    pub fn fill(&mut self) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        let paint = &state.paint;
        self.path_cache.fill_type = state.fill_type;
        let bounds_offset = Self::expand_fill_path(
            &mut self.path_cache,
            &state.paint,
            self.dist_tol,
            self.tess_tol,
            self.fringe_width,
        );

        let fill_slice = self.path_cache.get_fill_slice();
        self.renderer.fill(
            None,
            None,
            &paint.get_fill(),
            state.composite_operation,
            self.path_cache.path_commands().fill_type,
            &state.scissor,
            self.fringe_width,
            bounds_offset,
            fill_slice,
        )?;

        self.draw_call_count += fill_slice.len();
        self.fill_triangles_count += fill_slice.iter().fold(0, |mut acc, path_slice| {
            if path_slice.num_fill > 2 {
                acc += path_slice.num_fill - 2;
            }
            if path_slice.num_stroke > 2 {
                acc += path_slice.num_stroke - 2;
            }
            acc
        });
        Ok(())
    }

    pub fn stroke(&mut self) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        let paint = &state.paint;
        let antialias = self.renderer.edge_antialias() && paint.antialias;
        let (stroke_paint, stroke_width) = paint.get_stroke(
            antialias,
            self.fringe_width,
            state.xform.average_scale(),
            self.device_pixel_ratio,
        );

        Self::expand_stroke_path(
            &mut self.path_cache,
            antialias,
            stroke_width,
            &paint,
            self.dist_tol,
            self.tess_tol,
            self.fringe_width,
        );

        let stroke_slice = self.path_cache.get_stroke_slice();
        self.renderer.stroke(
            None,
            None,
            &stroke_paint,
            state.composite_operation,
            &state.scissor,
            self.fringe_width,
            stroke_width,
            stroke_slice,
        )?;

        self.draw_call_count += stroke_slice.len();
        self.fill_triangles_count += stroke_slice
            .iter()
            .fold(0, |acc, path_slice| acc + path_slice.num_stroke - 2);
        Ok(())
    }

    #[cfg(feature = "wirelines")]
    pub fn wirelines(&mut self) -> anyhow::Result<()> {
        Self::expand_wirelines_path(&mut self.path_cache, self.dist_tol, self.tess_tol);
        let state = self.states.last().unwrap();
        let (stroke_paint, _) = state.paint.get_stroke(false, 1.0, 1.0, 1.0);
        let lines_slice = self.path_cache.get_lines_slice();

        self.renderer.wirelines(
            None,
            None,
            &stroke_paint,
            state.composite_operation,
            &state.scissor,
            lines_slice,
        )?;
        self.draw_call_count += lines_slice.len();
        Ok(())
    }

    pub fn update_instances(&mut self, instances: &Instances<R>) -> anyhow::Result<()> {
        let instances_data = bytemuck::cast_slice(&instances.transforms);
        if !instances.is_empty() {
            let mut inner = instances.inner.borrow_mut();
            let try_update = inner.vertex_buffer.as_ref().and_then(|buffer| {
                self.renderer
                    .update_vertex_buffer(Some(&buffer), instances_data)
                    .ok()
            });
            if try_update.is_none() {
                let buffer = self.renderer.create_vertex_buffer(instances_data.len())?;
                self.renderer
                    .update_vertex_buffer(Some(&buffer), instances_data)?;
                inner.vertex_buffer = Some(buffer);
            }
        }
        Ok(())
    }

    pub fn draw_path<'a>(
        &'a mut self,
        path: &'a Path<R>,
        paint: &'a Paint,
        style: DrawPathStyle,
        instances: Option<(&Instances<R>, Range<u32>)>,
    ) -> anyhow::Result<()> {
        if style.is_empty() {
            return Ok(());
        }
        let cached_style = path.inner_cached_style();

        let (fill_cmd, stroke_cmd, lines_cmd) = if cached_style.contains(style) {
            let fill_cmd = if style.contains(DrawPathStyle::FILL) {
                Some(path.inner.borrow().draw_slice.bounds_offset)
            } else {
                None
            };
            let stroke_cmd = if style.contains(DrawPathStyle::STROKE) {
                let antialias = self.renderer.edge_antialias() && paint.antialias;
                Some(paint.get_stroke(
                    antialias,
                    self.fringe_width,
                    path.xform.average_scale(),
                    self.device_pixel_ratio,
                ))
            } else {
                None
            };
            let lines_cmd = if style.contains(DrawPathStyle::WIRELINES) {
                Some(())
            } else {
                None
            };
            (fill_cmd, stroke_cmd, lines_cmd)
        } else {
            // Update Vertex buffer
            let mut path_cache = PathRefWithCache::new(path);
            let new_style = cached_style | style;
            let fill_cmd = if new_style.contains(DrawPathStyle::FILL) {
                Some(Self::expand_fill_path(
                    &mut path_cache,
                    &paint,
                    self.dist_tol,
                    self.tess_tol,
                    self.fringe_width,
                ))
            } else {
                None
            };
            let stroke_cmd = if new_style.contains(DrawPathStyle::STROKE) {
                let antialias = self.renderer.edge_antialias() && paint.antialias;
                let (stroke_paint, stroke_width) = paint.get_stroke(
                    antialias,
                    self.fringe_width,
                    path.xform.average_scale(),
                    self.device_pixel_ratio,
                );
                Self::expand_stroke_path(
                    &mut path_cache,
                    antialias,
                    stroke_width,
                    &paint,
                    self.dist_tol,
                    self.tess_tol,
                    self.fringe_width,
                );
                Some((stroke_paint, stroke_width))
            } else {
                None
            };
            let lines_cmd = if style.contains(DrawPathStyle::WIRELINES) {
                Self::expand_wirelines_path(&mut path_cache, self.dist_tol, self.tess_tol);
                Some(())
            } else {
                None
            };
            let vertex_data = bytemuck::cast_slice(&path_cache.cache.vertices);
            if !path_cache.cache.vertices.is_empty() {
                let try_update =
                    path_cache
                        .path_mut_inner
                        .vertex_buffer
                        .as_ref()
                        .and_then(|buffer| {
                            self.renderer
                                .update_vertex_buffer(Some(&buffer), vertex_data)
                                .ok()
                        });
                if try_update.is_none() {
                    let buffer = self.renderer.create_vertex_buffer(vertex_data.len())?;
                    self.renderer
                        .update_vertex_buffer(Some(&buffer), vertex_data)?;
                    path_cache.path_mut_inner.vertex_buffer = Some(buffer);
                }
            }

            path_cache.path_mut_inner.style = new_style;
            (fill_cmd, stroke_cmd, lines_cmd)
        };

        let instances = instances.and_then(|(insts, range)| {
            if !insts.is_empty() {
                let inner = insts.inner.borrow();
                inner
                    .vertex_buffer
                    .clone()
                    .and_then(|buffer| Some((buffer, range)))
            } else {
                None
            }
        });

        // Start Draw-CALLs
        let state = self.states.last().unwrap();
        let inner = path.inner.borrow();
        if let Some(bounds_offset) = fill_cmd {
            let fill_slice = &inner.draw_slice.fill;
            self.renderer.fill(
                inner.vertex_buffer.clone(),
                instances.clone(),
                &paint.get_fill(),
                state.composite_operation,
                path.path_comands.fill_type,
                &state.scissor,
                self.fringe_width,
                bounds_offset,
                fill_slice,
            )?;

            self.draw_call_count += fill_slice.len();
            self.fill_triangles_count += fill_slice.iter().fold(0, |mut acc, path_slice| {
                if path_slice.num_fill > 2 {
                    acc += path_slice.num_fill - 2;
                }
                if path_slice.num_stroke > 2 {
                    acc += path_slice.num_stroke - 2;
                }
                acc
            });
        }

        if let Some((stroke_paint, stroke_width)) = stroke_cmd {
            let stroke_slice = &inner.draw_slice.stroke;
            self.renderer.stroke(
                inner.vertex_buffer.clone(),
                instances.clone(),
                &stroke_paint,
                state.composite_operation,
                &state.scissor,
                self.fringe_width,
                stroke_width,
                stroke_slice,
            )?;

            self.draw_call_count += stroke_slice.len();
            self.fill_triangles_count += stroke_slice
                .iter()
                .fold(0, |acc, path_slice| acc + path_slice.num_stroke - 2);
        }

        if let Some(_) = lines_cmd {
            let lines_slice = &inner.draw_slice.lines;
            let (stroke_paint, _) = paint.get_stroke(false, 1.0, 1.0, 1.0);
            self.renderer.wirelines(
                inner.vertex_buffer.clone(),
                instances.clone(),
                &stroke_paint,
                state.composite_operation,
                &state.scissor,
                lines_slice,
            )?;
            self.draw_call_count += lines_slice.len();
        }

        Ok(())
    }
}
