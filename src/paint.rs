use super::ImageId;
use crate::{Color, Extent, Point, Rect, Transform};

#[derive(Debug, Copy, Clone)]
pub struct PaintPattern {
    pub xform: Transform,
    pub extent: Extent,
    pub radius: f32,
    pub feather: f32,
    pub inner_color: Color,
    pub outer_color: Color,
    pub image: Option<ImageId>,
}

#[derive(Debug, Copy, Clone)]
pub enum Gradient {
    Linear {
        start: Point,
        end: Point,
        start_color: Color,
        end_color: Color,
    },
    Radial {
        center: Point,
        in_radius: f32,
        out_radius: f32,
        inner_color: Color,
        outer_color: Color,
    },
    Box {
        rect: Rect,
        radius: f32,
        feather: f32,
        inner_color: Color,
        outer_color: Color,
    },
}

#[derive(Debug, Copy, Clone)]
pub struct ImagePattern {
    pub center: Point,
    pub size: Extent,
    pub angle: f32,
    pub img: ImageId,
    pub alpha: f32,
}

impl From<Gradient> for PaintPattern {
    fn from(grad: Gradient) -> Self {
        match grad {
            Gradient::Linear {
                start,
                end,
                start_color: inner_color,
                end_color: outer_color,
            } => {
                const LARGE: f32 = 1e5;

                let mut dx = end.x - start.x;
                let mut dy = end.y - start.y;
                let d = (dx * dx + dy * dy).sqrt();

                if d > 0.0001 {
                    dx /= d;
                    dy /= d;
                } else {
                    dx = 0.0;
                    dy = 1.0;
                }

                PaintPattern {
                    xform: Transform([dy, -dx, dx, dy, start.x - dx * LARGE, start.y - dy * LARGE]),
                    extent: Extent {
                        width: LARGE,
                        height: LARGE + d * 0.5,
                    },
                    radius: 0.0,
                    feather: d.max(1.0),
                    inner_color,
                    outer_color,
                    image: None,
                }
            }
            Gradient::Radial {
                center,
                in_radius,
                out_radius,
                inner_color,
                outer_color,
            } => {
                let r = (in_radius + out_radius) * 0.5;
                let f = out_radius - in_radius;
                PaintPattern {
                    xform: Transform([1.0, 0.0, 0.0, 1.0, center.x, center.y]),
                    extent: Extent {
                        width: r,
                        height: r,
                    },
                    radius: r,
                    feather: f.max(1.0),
                    inner_color,
                    outer_color,
                    image: None,
                }
            }
            Gradient::Box {
                rect,
                radius,
                feather,
                inner_color,
                outer_color,
            } => {
                let Rect { xy, size } = rect;
                PaintPattern {
                    xform: Transform([
                        1.0,
                        0.0,
                        0.0,
                        1.0,
                        xy.x + size.width * 0.5,
                        xy.y + size.height * 0.5,
                    ]),
                    extent: Extent::new(size.width * 0.5, size.height * 0.5),
                    radius,
                    feather: feather.max(1.0),
                    inner_color,
                    outer_color,
                    image: None,
                }
            }
        }
    }
}

impl From<ImagePattern> for PaintPattern {
    fn from(pat: ImagePattern) -> Self {
        let mut xform = Transform::rotate(pat.angle);
        xform.0[4] = pat.center.x;
        xform.0[5] = pat.center.y;
        PaintPattern {
            xform,
            extent: pat.size,
            radius: 0.0,
            feather: 0.0,
            inner_color: Color::rgba(1.0, 1.0, 1.0, pat.alpha),
            outer_color: Color::rgba(1.0, 1.0, 1.0, pat.alpha),
            image: Some(pat.img),
        }
    }
}

impl<T: Into<Color> + Clone> From<T> for PaintPattern {
    fn from(color: T) -> Self {
        PaintPattern {
            xform: Transform::identity(),
            extent: Default::default(),
            radius: 0.0,
            feather: 1.0,
            inner_color: color.clone().into(),
            outer_color: color.into(),
            image: None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone)]
pub struct Paint {
    pub antialias: bool,
    pub alpha: f32,
    pub stroke: PaintPattern,
    pub fill: PaintPattern,
    pub stroke_width: f32,
    pub line_join: LineJoin,
    pub line_cap: LineCap,
    pub miter_limit: f32,
}

impl Paint {
    pub fn new() -> Self {
        return Self::default();
    }
}

impl Default for Paint {
    fn default() -> Self {
        return Self {
            antialias: true,
            alpha: 1.0,
            stroke: Color::rgb(0.0, 0.0, 0.0).into(),
            stroke_width: 1.0,
            line_join: LineJoin::Miter,
            line_cap: LineCap::Butt,
            miter_limit: 10.0,
            fill: Color::rgb(1.0, 1.0, 1.0).into(),
        };
    }
}
