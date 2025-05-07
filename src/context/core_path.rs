use crate::{renderer::Scissor, PaintPattern, Path};
use crate::{Color, LineJoin, Paint};
use crate::{PaintStyle, PathDir, Point, Rect, RendererDevice};

use super::*;
use clamped::Clamp;

impl<R: RendererDevice> Context<R> {
    #[inline]
    pub fn move_to<P: Into<Point>>(&mut self, pt: P) {
        self.path.xform = self.state_mut().xform;
        self.path.move_to(pt);
    }

    #[inline]
    pub fn line_to<P: Into<Point>>(&mut self, pt: P) {
        self.path.xform = self.state_mut().xform;
        self.path.line_to(pt);
    }

    #[inline]
    pub fn bezier_to<P: Into<Point>>(&mut self, cp1: P, cp2: P, pt: P) {
        self.path.xform = self.state_mut().xform;
        self.path.bezier_to(cp1, cp2, pt);
    }

    #[inline]
    pub fn quad_to<P: Into<Point>>(&mut self, cp: P, pt: P) {
        self.path.xform = self.state_mut().xform;
        self.path.quad_to(cp, pt);
    }

    #[inline]
    pub fn arc_to<P: Into<Point>>(&mut self, pt1: P, pt2: P, radius: f32) {
        self.path.xform = self.state_mut().xform;
        if self.path.commands.is_empty() {
            return;
        }
        let pt0 = self.path.last_position;
        let pt1 = pt1.into();
        let pt2 = pt2.into();
        if pt0.equals(pt1, self.dist_tol)
            || pt1.equals(pt2, self.dist_tol)
            || pt1.dist_pt_seg(pt0, pt2) < self.dist_tol * self.dist_tol
            || radius < self.dist_tol
        {
            self.line_to(pt1);
            return;
        }
        self.path.inner_arc_to(pt0, pt1, pt2, radius);
    }

    #[inline]
    pub fn arc<P: Into<Point>>(&mut self, cp: P, radius: f32, a0: f32, a1: f32, dir: PathDir) {
        self.path.xform = self.state_mut().xform;
        self.path.arc(cp, radius, a0, a1, dir);
    }

    #[inline]
    pub fn rect<T: Into<Rect>>(&mut self, rect: T) {
        self.path.xform = self.state_mut().xform;
        self.path.rect(rect);
    }

    #[inline]
    pub fn rounded_rect<T: Into<Rect>>(&mut self, rect: T, radius: f32) {
        self.path.xform = self.state_mut().xform;
        self.path.rounded_rect(rect, radius);
    }

    #[inline]
    pub fn rounded_rect_varying<T: Into<Rect>>(
        &mut self,
        rect: T,
        lt: f32,
        rt: f32,
        rb: f32,
        lb: f32,
    ) {
        self.path.xform = self.state_mut().xform;
        self.path.rounded_rect_varying(rect, lt, rt, rb, lb);
    }

    #[inline]
    pub fn ellipse<P: Into<Point>>(&mut self, center: P, radius_x: f32, radius_y: f32) {
        self.path.xform = self.state_mut().xform;
        self.path.ellipse(center, radius_x, radius_y);
    }

    #[inline]
    pub fn circle<P: Into<Point>>(&mut self, center: P, radius: f32) {
        self.path.xform = self.state_mut().xform;
        self.path.circle(center, radius);
    }

    #[inline]
    pub fn path_winding<D: Into<PathDir>>(&mut self, dir: D) {
        self.path.path_winding(dir);
    }

    #[inline]
    pub fn begin_path(&mut self) {
        self.path.clear();
    }

    #[inline]
    pub fn close_path(&mut self) {
        self.path.close_path();
    }

    pub fn fill(&mut self) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        self.path.fill_type = state.fill_type;
        let (draw_call_count, fill_triangles_count) = Self::fill_path(
            &mut self.renderer,
            &self.path,
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
        let (draw_call_count, fill_triangles_count) = Self::stroke_path(
            &mut self.renderer,
            &self.path,
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

    pub fn clear(&mut self, color: Color) -> anyhow::Result<()> {
        return self.renderer.clear(color);
    }

    #[cfg(feature = "wirelines")]
    #[inline]
    fn wirelines_path(
        renderer: &mut R,
        path: &Path,
        stroke: &PaintPattern,
        dist_tol: f32,
        tess_tol: f32,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
    ) -> anyhow::Result<usize> {
        let mut cache = path.cache.borrow_mut();
        cache.flatten_paths(&path.commands, dist_tol, tess_tol);
        cache.expand_lines();

        renderer.wirelines(&stroke, composite_operation, &scissor, &cache.paths)?;

        let mut draw_call_count = 0;
        for _path in &cache.paths {
            draw_call_count += 1;
        }
        Ok(draw_call_count)
    }

    #[cfg(feature = "wirelines")]
    pub fn wirelines(&mut self) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        let draw_call_count = Self::wirelines_path(
            &mut self.renderer,
            &self.path,
            &state.paint.stroke,
            self.dist_tol,
            self.tess_tol,
            state.composite_operation,
            &state.scissor,
        )?;
        self.draw_call_count += draw_call_count;
        Ok(())
    }

    #[cfg(feature = "wirelines")]
    pub fn draw_wirelines_path(
        &mut self,
        path: &Path,
        stroke: &PaintPattern,
    ) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        let draw_call_count = Self::wirelines_path(
            &mut self.renderer,
            path,
            stroke,
            self.dist_tol,
            self.tess_tol,
            state.composite_operation,
            &state.scissor,
        )?;
        self.draw_call_count += draw_call_count;
        Ok(())
    }

    #[inline]
    fn stroke_path(
        renderer: &mut R,
        path: &Path,
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
        let mut cache = path.cache.borrow_mut();
        cache.flatten_paths(&path.commands, dist_tol, tess_tol);

        if renderer.edge_antialias() && paint.antialias {
            if stroke_width < fringe_width {
                let alpha = (stroke_width / fringe_width).clamped(0.0, 1.0);
                stroke_paint.inner_color.a *= alpha * alpha;
                stroke_paint.outer_color.a *= alpha * alpha;
                stroke_width = fringe_width;
            }

            stroke_paint.inner_color.a *= paint.alpha;
            stroke_paint.outer_color.a *= paint.alpha;

            cache.expand_stroke(
                stroke_width * 0.5,
                fringe_width,
                paint.line_cap,
                paint.line_join,
                paint.miter_limit,
                tess_tol,
            );
        } else {
            cache.expand_stroke(
                stroke_width * 0.5,
                0.0,
                paint.line_cap,
                paint.line_join,
                paint.miter_limit,
                tess_tol,
            );
        }

        renderer.stroke(
            &stroke_paint,
            composite_operation,
            &scissor,
            fringe_width,
            stroke_width,
            &cache.paths,
        )?;
        let mut fill_triangles_count = 0;
        let mut draw_call_count = 0;
        for path in &cache.paths {
            fill_triangles_count += path.num_stroke - 2;
            draw_call_count += 1;
        }

        Ok((draw_call_count, fill_triangles_count))
    }

    #[inline]
    fn fill_path(
        renderer: &mut R,
        path: &Path,
        paint: &Paint,
        dist_tol: f32,
        tess_tol: f32,
        fringe_width: f32,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
    ) -> anyhow::Result<(usize, usize)> {
        let mut fill_paint = paint.fill.clone();
        let mut cache = path.cache.borrow_mut();

        cache.flatten_paths(&path.commands, dist_tol, tess_tol);
        if paint.antialias && paint.antialias {
            cache.expand_fill(fringe_width, LineJoin::Miter, 2.4, fringe_width);
        } else {
            cache.expand_fill(0.0, LineJoin::Miter, 2.4, fringe_width);
        }

        fill_paint.inner_color.a *= paint.alpha;
        fill_paint.outer_color.a *= paint.alpha;

        renderer.fill(
            &fill_paint,
            composite_operation,
            path.fill_type,
            &scissor,
            fringe_width,
            cache.bounds,
            &cache.paths,
        )?;

        let mut fill_triangles_count = 0;
        let mut draw_call_count = 0;
        for path in &cache.paths {
            if path.num_fill > 2 {
                fill_triangles_count += path.num_fill - 2;
            }
            if path.num_stroke > 2 {
                fill_triangles_count += path.num_stroke - 2;
            }
            draw_call_count += 2;
        }

        Ok((draw_call_count, fill_triangles_count))
    }

    pub fn draw_path(&mut self, path: &Path, paint: &Paint) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        match paint.style {
            PaintStyle::Stroke => {
                let (draw_call_count, fill_triangles_count) = Self::stroke_path(
                    &mut self.renderer,
                    path,
                    paint,
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
            }
            PaintStyle::Fill => {
                let (draw_call_count, fill_triangles_count) = Self::fill_path(
                    &mut self.renderer,
                    path,
                    paint,
                    self.dist_tol,
                    self.tess_tol,
                    self.fringe_width,
                    state.composite_operation,
                    &state.scissor,
                )?;
                self.draw_call_count += draw_call_count;
                self.fill_triangles_count += fill_triangles_count;
            }
            PaintStyle::StrokeAndFill => {
                let (draw_call_count, fill_triangles_count) = Self::fill_path(
                    &mut self.renderer,
                    path,
                    paint,
                    self.dist_tol,
                    self.tess_tol,
                    self.fringe_width,
                    state.composite_operation,
                    &state.scissor,
                )?;
                self.draw_call_count += draw_call_count;
                self.fill_triangles_count += fill_triangles_count;
                let (draw_call_count, fill_triangles_count) = Self::stroke_path(
                    &mut self.renderer,
                    path,
                    paint,
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
            }
        };
        Ok(())
    }
}
