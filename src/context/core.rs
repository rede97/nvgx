use super::{Align, BasicCompositeOperation, CompositeOperation, CompositeOperationState};
use crate::fonts::{FontId, Fonts, LayoutChar};
use crate::paint::{LineCap, LineJoin, PaintPattern};
use crate::path::Path;
use crate::renderer::Scissor;
use crate::{Color, Paint, PaintStyle, PathFillType};
use crate::{Extent, Point, Rect, Renderer, Transform};
use clamped::Clamp;

#[derive(Clone)]
pub(super) struct State {
    pub(super) composite_operation: CompositeOperationState,
    pub(super) paint: Paint,
    pub(super) fill_type: PathFillType,
    pub(super) xform: Transform,
    pub(super) scissor: Scissor,
    pub(super) font_size: f32,
    pub(super) letter_spacing: f32,
    pub(super) line_height: f32,
    pub(super) text_align: Align,
    pub(super) font_id: FontId,
}

impl Default for State {
    fn default() -> Self {
        State {
            composite_operation: CompositeOperation::Basic(BasicCompositeOperation::SrcOver).into(),
            paint: Default::default(),
            fill_type: PathFillType::Winding,
            xform: Transform::identity(),
            scissor: Scissor {
                xform: Default::default(),
                extent: Extent {
                    width: -1.0,
                    height: -1.0,
                },
            },
            font_size: 16.0,
            letter_spacing: 0.0,
            line_height: 1.0,
            text_align: Align::LEFT | Align::BASELINE,
            font_id: 0,
        }
    }
}

pub struct Context<R: Renderer> {
    pub(super) renderer: R,
    pub(super) path: Path,
    pub(super) states: Vec<State>,
    pub(super) tess_tol: f32,
    pub(super) dist_tol: f32,
    pub(super) fringe_width: f32,
    pub(super) device_pixel_ratio: f32,
    pub(super) fonts: Fonts,
    pub(super) layout_chars: Vec<LayoutChar>,
    pub(super) draw_call_count: usize,
    pub(super) fill_triangles_count: usize,
    pub(super) stroke_triangles_count: usize,
    pub(super) text_triangles_count: usize,
}

impl<R: Renderer> Context<R> {
    pub fn create(mut renderer: R) -> anyhow::Result<Context<R>> {
        let fonts = Fonts::new(&mut renderer)?;
        Ok(Context {
            renderer,
            path: Path::new(),
            states: vec![Default::default()],
            tess_tol: 0.0,
            dist_tol: 0.0,
            fringe_width: 1.0,
            device_pixel_ratio: 0.0,
            fonts,
            layout_chars: Default::default(),
            draw_call_count: 0,
            fill_triangles_count: 0,
            stroke_triangles_count: 0,
            text_triangles_count: 0,
        })
    }

    pub fn renderer(&self) -> &R {
        &self.renderer
    }

    fn set_device_pixel_ratio(&mut self, ratio: f32) {
        self.tess_tol = 0.25 / ratio;
        self.dist_tol = 0.01 / ratio;
        self.device_pixel_ratio = ratio;
    }

    pub fn begin_frame<E: Into<Extent>>(
        &mut self,
        window_extent: E,
        device_pixel_ratio: f32,
    ) -> anyhow::Result<()> {
        self.states.clear();
        self.states.push(Default::default());
        self.set_device_pixel_ratio(device_pixel_ratio);
        self.renderer
            .viewport(window_extent.into(), device_pixel_ratio)?;
        self.draw_call_count = 0;
        self.fill_triangles_count = 0;
        self.stroke_triangles_count = 0;
        self.text_triangles_count = 0;
        Ok(())
    }

    pub fn cancel_frame(&mut self) -> anyhow::Result<()> {
        self.renderer.cancel()?;
        Ok(())
    }

    pub fn end_frame(&mut self) -> anyhow::Result<()> {
        self.renderer.flush()?;
        Ok(())
    }

    pub fn save(&mut self) {
        if let Some(last) = self.states.last() {
            let last = last.clone();
            self.states.push(last);
        }
    }

    pub fn restore(&mut self) {
        if self.states.len() <= 1 {
            return;
        }
        self.states.pop();
    }

    fn state(&mut self) -> &State {
        self.states.last().unwrap()
    }

    pub(super) fn state_mut(&mut self) -> &mut State {
        self.states.last_mut().unwrap()
    }

    pub fn reset(&mut self) {
        *self.state_mut() = Default::default();
    }

    pub fn shape_antialias(&mut self, enabled: bool) {
        self.state_mut().paint.antialias = enabled;
    }

    pub fn stroke_width(&mut self, width: f32) {
        self.state_mut().paint.stroke_width = width;
    }

    pub fn miter_limit(&mut self, limit: f32) {
        self.state_mut().paint.miter_limit = limit;
    }

    pub fn line_cap(&mut self, cap: LineCap) {
        self.state_mut().paint.line_cap = cap;
    }

    pub fn line_join(&mut self, join: LineJoin) {
        self.state_mut().paint.line_join = join;
    }

    pub fn global_alpha(&mut self, alpha: f32) {
        self.state_mut().paint.alpha = alpha;
    }

    pub fn transform(&mut self, xform: Transform) {
        let state = self.state_mut();
        state.xform = xform * state.xform;
    }

    pub fn reset_transform(&mut self) {
        self.state_mut().xform = Transform::identity();
    }

    pub fn translate(&mut self, tx: f32, ty: f32) {
        self.transform(Transform::translate(tx, ty));
    }

    pub fn rotate(&mut self, angle: f32) {
        self.transform(Transform::rotate(angle));
    }

    pub fn skew_x(&mut self, angle: f32) {
        self.transform(Transform::skew_x(angle));
    }

    pub fn skew_y(&mut self, angle: f32) {
        self.transform(Transform::skew_y(angle));
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.transform(Transform::scale(sx, sy));
    }

    pub fn current_transform(&mut self) -> Transform {
        self.state().xform
    }

    pub fn stroke_paint<T: Into<PaintPattern>>(&mut self, paint: T) {
        let mut paint = paint.into();
        paint.xform *= self.state().xform;
        self.state_mut().paint.stroke = paint;
    }

    pub fn fill_paint<T: Into<PaintPattern>>(&mut self, paint: T) {
        let mut paint = paint.into();
        paint.xform *= self.state().xform;
        self.state_mut().paint.fill = paint;
    }

    pub fn fill_type(&mut self, fill_type: PathFillType) {
        self.state_mut().fill_type = fill_type;
    }

    pub fn scissor<T: Into<Rect>>(&mut self, rect: T) {
        let rect = rect.into();
        let state = self.state_mut();
        let x = rect.xy.x;
        let y = rect.xy.y;
        let width = rect.size.width.max(0.0);
        let height = rect.size.height.max(0.0);
        state.scissor.xform = Transform::identity();
        state.scissor.xform.0[4] = x + width * 0.5;
        state.scissor.xform.0[5] = y + height * 0.5;
        state.scissor.xform *= state.xform;
        state.scissor.extent.width = width * 0.5;
        state.scissor.extent.height = height * 0.5;
    }

    pub fn intersect_scissor<T: Into<Rect>>(&mut self, rect: T) {
        let rect = rect.into();
        let state = self.state_mut();

        if state.scissor.extent.width < 0.0 {
            self.scissor(rect);
            return;
        }

        let Extent {
            width: ex,
            height: ey,
        } = state.scissor.extent;
        let invxorm = state.xform.inverse();
        let pxform = state.scissor.xform * invxorm;
        let tex = ex * pxform.0[0].abs() + ey * pxform.0[2].abs();
        let tey = ex * pxform.0[1].abs() + ey * pxform.0[3].abs();
        self.scissor(
            Rect::new(
                Point::new(pxform.0[4] - tex, pxform.0[5] - tey),
                Extent::new(tex * 2.0, tey * 2.0),
            )
            .intersect(rect),
        );
    }

    pub fn reset_scissor(&mut self) {
        let state = self.state_mut();
        state.scissor.xform = Transform::default();
        state.scissor.extent.width = -1.0;
        state.scissor.extent.height = -1.0;
    }

    pub fn global_composite_operation(&mut self, op: CompositeOperation) {
        self.state_mut().composite_operation = op.into();
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

    #[cfg(feature = "wireframe")]
    pub fn wireframe(&mut self, enable: bool) -> anyhow::Result<()> {
        return self.renderer.wireframe(enable);
    }

    #[cfg(feature = "wirelines")]
    pub fn wirelines(&mut self) -> anyhow::Result<()> {
        let state = self.states.last_mut().unwrap();

        let mut cache = self.path.cache.borrow_mut();
        cache.flatten_paths(&self.path.commands, self.dist_tol, self.tess_tol);
        cache.expand_lines();

        self.renderer.wirelines(
            &state.paint.stroke,
            state.composite_operation,
            &state.scissor,
            &cache.paths,
        )?;
        for _path in &cache.paths {
            self.draw_call_count += 1;
        }
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
