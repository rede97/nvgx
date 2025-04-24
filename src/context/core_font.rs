use super::Context;
use super::{Align, TextMetrics};
use crate::fonts::FontId;
use crate::path::cache::Vertex;
use crate::{Extent, Point, Renderer};

impl<R: Renderer> Context<R> {
    pub fn create_font_from_file<N: Into<String>, P: AsRef<std::path::Path>>(
        &mut self,
        name: N,
        path: P,
    ) -> anyhow::Result<FontId> {
        self.create_font(name, std::fs::read(path)?)
    }

    pub fn create_font<N: Into<String>, D: Into<Vec<u8>>>(
        &mut self,
        name: N,
        data: D,
    ) -> anyhow::Result<FontId> {
        self.fonts.add_font(name, data)
    }

    pub fn find_font<N: AsRef<str>>(&self, name: N) -> Option<FontId> {
        self.fonts.find(name.as_ref())
    }

    pub fn add_fallback_fontid(&mut self, base: FontId, fallback: FontId) {
        self.fonts.add_fallback(base, fallback);
    }

    pub fn add_fallback_font<N1: AsRef<str>, N2: AsRef<str>>(&mut self, base: N1, fallback: N2) {
        if let (Some(base), Some(fallback)) = (self.find_font(base), self.find_font(fallback)) {
            self.fonts.add_fallback(base, fallback);
        }
    }

    pub fn font_size(&mut self, size: f32) {
        self.state_mut().font_size = size;
    }

    pub fn text_letter_spacing(&mut self, spacing: f32) {
        self.state_mut().letter_spacing = spacing;
    }

    pub fn text_line_height(&mut self, line_height: f32) {
        self.state_mut().line_height = line_height;
    }

    pub fn text_align(&mut self, align: Align) {
        self.state_mut().text_align = align;
    }

    pub fn fontid(&mut self, id: FontId) {
        self.state_mut().font_id = id;
    }

    pub fn font<N: AsRef<str>>(&mut self, name: N) {
        if let Some(id) = self.find_font(name) {
            self.state_mut().font_id = id;
        }
    }

    pub fn text<S: AsRef<str>, P: Into<Point>>(&mut self, pt: P, text: S) -> anyhow::Result<()> {
        let state = self.states.last().unwrap();
        let mut cache = self.path.cache.borrow_mut();
        let scale = state.xform.font_scale() * self.device_pixel_ratio;
        let xform = &state.xform;
        let invscale = 1.0 / scale;
        let pt = pt.into();

        self.fonts.layout_text(
            &mut self.renderer,
            text.as_ref(),
            state.font_id,
            (pt.x * scale, pt.y * scale).into(),
            state.font_size * scale,
            state.text_align,
            state.letter_spacing * scale,
            true,
            &mut self.layout_chars,
        )?;

        cache.vertexes.clear();

        for lc in &self.layout_chars {
            let lt = xform.transform_point(Point::new(
                lc.bounds.min.x * invscale,
                lc.bounds.min.y * invscale,
            ));
            let rt = xform.transform_point(Point::new(
                lc.bounds.max.x * invscale,
                lc.bounds.min.y * invscale,
            ));
            let lb = xform.transform_point(Point::new(
                lc.bounds.min.x * invscale,
                lc.bounds.max.y * invscale,
            ));
            let rb = xform.transform_point(Point::new(
                lc.bounds.max.x * invscale,
                lc.bounds.max.y * invscale,
            ));

            cache
                .vertexes
                .push(Vertex::new(lt.x, lt.y, lc.uv.min.x, lc.uv.min.y));
            cache
                .vertexes
                .push(Vertex::new(rb.x, rb.y, lc.uv.max.x, lc.uv.max.y));
            cache
                .vertexes
                .push(Vertex::new(rt.x, rt.y, lc.uv.max.x, lc.uv.min.y));

            cache
                .vertexes
                .push(Vertex::new(lt.x, lt.y, lc.uv.min.x, lc.uv.min.y));
            cache
                .vertexes
                .push(Vertex::new(lb.x, lb.y, lc.uv.min.x, lc.uv.max.y));
            cache
                .vertexes
                .push(Vertex::new(rb.x, rb.y, lc.uv.max.x, lc.uv.max.y));
        }

        let mut paint = state.paint.fill.clone();
        paint.image = Some(self.fonts.img.clone());
        paint.inner_color.a *= state.paint.alpha;
        paint.outer_color.a *= state.paint.alpha;

        self.renderer.triangles(
            &paint,
            state.composite_operation,
            &state.scissor,
            &cache.vertexes,
        )?;
        Ok(())
    }

    pub fn text_metrics(&self) -> TextMetrics {
        let state = self.states.last().unwrap();
        let scale = state.xform.font_scale() * self.device_pixel_ratio;
        self.fonts
            .text_metrics(state.font_id, state.font_size * scale)
    }

    pub fn text_size<S: AsRef<str>>(&self, text: S) -> Extent {
        let state = self.states.last().unwrap();
        let scale = state.xform.font_scale() * self.device_pixel_ratio;
        self.fonts.text_size(
            text.as_ref(),
            state.font_id,
            state.font_size * scale,
            state.letter_spacing * scale,
        )
    }
}
