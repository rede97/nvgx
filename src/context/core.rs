use super::{Align, BasicCompositeOperation, CompositeOperation, CompositeOperationState};
use crate::fonts::{FontId, Fonts, LayoutChar};
use crate::paint::{LineCap, LineJoin, PaintPattern};
use crate::renderer::Scissor;
use crate::{Extent, Paint, PathFillType, PathWithCache, Point, Rect, RendererDevice, Transform};

pub(super) const INIT_VERTEX_BUFF_SIZE: usize = 10 * 1024;

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

pub struct Context<R: RendererDevice> {
    pub(super) renderer: R,
    pub(super) path: PathWithCache<R::VertexBuffer>,
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

impl<R: RendererDevice> Context<R> {
    pub fn create(mut renderer: R) -> anyhow::Result<Context<R>> {
        let fonts = Fonts::new(&mut renderer)?;
        Ok(Context {
            path: PathWithCache::new(renderer.create_vertex_buffer(INIT_VERTEX_BUFF_SIZE)?),
            renderer,
            states: vec![Default::default()],
            tess_tol: 0.0,
            dist_tol: 0.0,
            fringe_width: 1.0,
            device_pixel_ratio: 1.0,
            fonts,
            layout_chars: Default::default(),
            draw_call_count: 0,
            fill_triangles_count: 0,
            stroke_triangles_count: 0,
            text_triangles_count: 0,
        })
    }

    #[inline]
    pub fn renderer(&self) -> &R {
        &self.renderer
    }

    #[inline]
    pub fn renderer_mut(&mut self) -> &mut R {
        &mut self.renderer
    }

    pub fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        self.renderer.resize(width, height)
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
        let mut cache = self.path.cache.borrow_mut();
        self.renderer
            .update_vertex_buffer(&mut self.path.vertex_buffer, &cache.vertexes)?;
        self.renderer.flush()?;
        cache.reset();
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
}
