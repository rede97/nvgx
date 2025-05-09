use crate::renderer::Scissor;
#[cfg(feature = "wirelines")]
use crate::PaintPattern;
use crate::{Color, LineJoin, Paint};
use crate::{PathDir, Point, Rect, RendererDevice};

use super::*;
use clamped::Clamp;

impl<R: RendererDevice> Context<R> {
    pub fn fill(&mut self) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        self.path_cache.fill_type = state.fill_type;
        let (draw_call_count, fill_triangles_count) = Self::expand_fill_path(
            &mut self.renderer,
            &mut self.path_cache,
            &state.paint,
            self.dist_tol,
            self.tess_tol,
            self.fringe_width,
            state.composite_operation,
            &state.scissor,
        )?;
        self.draw_call_count += draw_call_count;
        self.fill_triangles_count += fill_triangles_count;
        Ok(())
    }

    pub fn stroke(&mut self) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        let (draw_call_count, fill_triangles_count) = Self::expand_stroke_path(
            &mut self.renderer,
            &mut self.path_cache,
            &state.paint,
            state.xform.average_scale(),
            self.device_pixel_ratio,
            self.dist_tol,
            self.tess_tol,
            self.fringe_width,
            state.composite_operation,
            &state.scissor,
        )?;
        self.draw_call_count += draw_call_count;
        self.fill_triangles_count += fill_triangles_count;
        Ok(())
    }

    #[cfg(feature = "wirelines")]
    pub fn wirelines(&mut self) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        let draw_call_count = Self::expand_wirelines_path(
            &mut self.renderer,
            &mut self.path_cache,
            &state.paint.stroke,
            self.dist_tol,
            self.tess_tol,
            state.composite_operation,
            &state.scissor,
        )?;
        self.draw_call_count += draw_call_count;
        Ok(())
    }

    #[inline]
    fn expand_fill_path<FE: FlattenExpandPath>(
        renderer: &mut R,
        path_cache: &mut FE,
        paint: &Paint,
        dist_tol: f32,
        tess_tol: f32,
        fringe_width: f32,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
    ) -> anyhow::Result<(usize, usize)> {
        let mut fill_paint = paint.fill.clone();

        path_cache.flatten_paths(dist_tol, tess_tol);
        let bounds_offset = if paint.antialias && paint.antialias {
            path_cache.expand_fill(fringe_width, LineJoin::Miter, 2.4, fringe_width)
        } else {
            path_cache.expand_fill(0.0, LineJoin::Miter, 2.4, fringe_width)
        };

        fill_paint.inner_color.a *= paint.alpha;
        fill_paint.outer_color.a *= paint.alpha;

        renderer.fill(
            None,
            &fill_paint,
            composite_operation,
            path_cache.path_commands().fill_type,
            &scissor,
            fringe_width,
            bounds_offset,
            path_cache.get_fill_slice(),
        )?;

        let mut fill_triangles_count = 0;
        let mut draw_call_count = 0;
        for path_slice in path_cache.get_fill_slice() {
            if path_slice.num_fill > 2 {
                fill_triangles_count += path_slice.num_fill - 2;
            }
            if path_slice.num_stroke > 2 {
                fill_triangles_count += path_slice.num_stroke - 2;
            }
            draw_call_count += 2;
        }

        Ok((draw_call_count, fill_triangles_count))
    }

    #[inline]
    fn expand_stroke_path<FE: FlattenExpandPath>(
        renderer: &mut R,
        path_cache: &mut FE,
        paint: &Paint,
        average_scale: f32,
        device_pixel_ratio: f32,
        dist_tol: f32,
        tess_tol: f32,
        fringe_width: f32,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
    ) -> anyhow::Result<(usize, usize)> {
        let mut stroke_width =
            (paint.stroke_width * device_pixel_ratio * average_scale).clamped(0.0, 200.0);
        let mut stroke_paint = paint.stroke;
        path_cache.flatten_paths(dist_tol, tess_tol);

        if renderer.edge_antialias() && paint.antialias {
            if stroke_width < fringe_width {
                let alpha = (stroke_width / fringe_width).clamped(0.0, 1.0);
                stroke_paint.inner_color.a *= alpha * alpha;
                stroke_paint.outer_color.a *= alpha * alpha;
                stroke_width = fringe_width;
            }

            stroke_paint.inner_color.a *= paint.alpha;
            stroke_paint.outer_color.a *= paint.alpha;

            path_cache.expand_stroke(
                stroke_width * 0.5,
                fringe_width,
                paint.line_cap,
                paint.line_join,
                paint.miter_limit,
                tess_tol,
            );
        } else {
            path_cache.expand_stroke(
                stroke_width * 0.5,
                0.0,
                paint.line_cap,
                paint.line_join,
                paint.miter_limit,
                tess_tol,
            );
        }

        renderer.stroke(
            None,
            &stroke_paint,
            composite_operation,
            &scissor,
            fringe_width,
            stroke_width,
            path_cache.get_stroke_slice(),
        )?;
        let mut fill_triangles_count = 0;
        let mut draw_call_count = 0;
        for path_slice in path_cache.get_stroke_slice() {
            fill_triangles_count += path_slice.num_stroke - 2;
            draw_call_count += 1;
        }

        Ok((draw_call_count, fill_triangles_count))
    }

    #[cfg(feature = "wirelines")]
    #[inline]
    fn expand_wirelines_path<FE: FlattenExpandPath>(
        renderer: &mut R,
        path_cache: &mut FE,
        stroke: &PaintPattern,
        dist_tol: f32,
        tess_tol: f32,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
    ) -> anyhow::Result<usize> {
        path_cache.flatten_paths(dist_tol, tess_tol);
        path_cache.expand_lines();

        renderer.wirelines(
            None,
            &stroke,
            composite_operation,
            &scissor,
            path_cache.get_lines_slice(),
        )?;

        let mut draw_call_count = 0;
        for _path in path_cache.get_lines_slice() {
            draw_call_count += 1;
        }
        Ok(draw_call_count)
    }

    #[inline]
    fn fill_path(&mut self, path_cache: &mut PathRefWithCache<R>) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        self.path_cache.fill_type = state.fill_type;
        let (draw_call_count, fill_triangles_count) = Self::expand_fill_path(
            &mut self.renderer,
            path_cache,
            &path_cache.paint,
            self.dist_tol,
            self.tess_tol,
            self.fringe_width,
            state.composite_operation,
            &state.scissor,
        )?;
        self.draw_call_count += draw_call_count;
        self.fill_triangles_count += fill_triangles_count;
        Ok(())
    }

    #[inline]
    fn stroke_path(&mut self, path_cache: &mut PathRefWithCache<R>) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        let (draw_call_count, fill_triangles_count) = Context::<R>::expand_stroke_path(
            &mut self.renderer,
            path_cache,
            &path_cache.paint,
            state.xform.average_scale(),
            self.device_pixel_ratio,
            self.dist_tol,
            self.tess_tol,
            self.fringe_width,
            state.composite_operation,
            &state.scissor,
        )?;
        self.draw_call_count += draw_call_count;
        self.fill_triangles_count += fill_triangles_count;

        Ok(())
    }

    #[cfg(feature = "wirelines")]
    #[inline]
    pub fn wirelines_path(&mut self, path_cache: &mut PathRefWithCache<R>) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        let draw_call_count = Self::expand_wirelines_path(
            &mut self.renderer,
            &mut self.path_cache,
            &path_cache.paint,
            self.dist_tol,
            self.tess_tol,
            state.composite_operation,
            &state.scissor,
        )?;
        self.draw_call_count += draw_call_count;
        Ok(())
    }

    pub fn draw_path<'a>(
        &'a mut self,
        path: &'a Path<R>,
        paint: &'a Paint,
    ) -> DrawPathContext<'a, R> {
        return DrawPathContext {
            path_cache: PathRefWithCache::new(path, paint),
            context: self,
        };
    }
}

pub struct DrawPathContext<'a, R: RendererDevice> {
    pub(crate) path_cache: PathRefWithCache<'a, R>,
    pub(crate) context: &'a mut Context<R>,
}

impl<'a, R: RendererDevice> DrawPathContext<'a, R> {
    #[inline]
    pub fn stroke(&mut self, wirelines: bool) -> anyhow::Result<()> {
        if wirelines {
            return self.context.wirelines_path(&mut self.path_cache);
        } else {
            return self.context.stroke_path(&mut self.path_cache);
        }

        // match paint.style {
        //     PaintStyle::Stroke => {
        //         let (draw_call_count, fill_triangles_count) = Self::stroke_path(
        //             &mut self.renderer,
        //             path,
        //             paint,
        //             state.xform.average_scale(),
        //             self.device_pixel_ratio,
        //             self.dist_tol,
        //             self.tess_tol,
        //             self.fringe_width,
        //             state.composite_operation,
        //             &state.scissor,
        //         )?;
        //         self.draw_call_count += draw_call_count;
        //         self.fill_triangles_count += fill_triangles_count;
        //     }
        //     PaintStyle::Fill => {
        //         let (draw_call_count, fill_triangles_count) = Self::fill_path(
        //             &mut self.renderer,
        //             path,
        //             paint,
        //             self.dist_tol,
        //             self.tess_tol,
        //             self.fringe_width,
        //             state.composite_operation,
        //             &state.scissor,
        //         )?;
        //         self.draw_call_count += draw_call_count;
        //         self.fill_triangles_count += fill_triangles_count;
        //     }
        //     PaintStyle::StrokeAndFill => {
        //         let (draw_call_count, fill_triangles_count) = Self::fill_path(
        //             &mut self.renderer,
        //             path,
        //             paint,
        //             self.dist_tol,
        //             self.tess_tol,
        //             self.fringe_width,
        //             state.composite_operation,
        //             &state.scissor,
        //         )?;
        //         self.draw_call_count += draw_call_count;
        //         self.fill_triangles_count += fill_triangles_count;
        //         let (draw_call_count, fill_triangles_count) = Self::stroke_path(
        //             &mut self.renderer,
        //             path,
        //             paint,
        //             state.xform.average_scale(),
        //             self.device_pixel_ratio,
        //             self.dist_tol,
        //             self.tess_tol,
        //             self.fringe_width,
        //             state.composite_operation,
        //             &state.scissor,
        //         )?;
        //         self.draw_call_count += draw_call_count;
        //         self.fill_triangles_count += fill_triangles_count;
        //     }
        // };
        // self.renderer
        //     .update_vertex_buffer(path.vertex_buffer, &path.cache.borrow().vertexes)?;
    }

    pub fn fill(&mut self) {}

    pub fn lines(&mut self) {}

    fn finish(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
