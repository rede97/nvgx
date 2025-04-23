use super::{LineCap, LineJoin, PaintInfo};

pub struct StrokePaint {
    pub stroke_width: f32,
    pub stroke_paint: PaintInfo,
    pub line_join: LineJoin,
}
