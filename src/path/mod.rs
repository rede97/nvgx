use crate::Point;

pub mod cache;
pub mod path;

pub const KAPPA90: f32 = 0.5522847493;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PathDir {
    CCW,
    CW,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FillType {
    Winding,
    EvenOdd,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WindingSolidity {
    Solid,
    Hole,
}

impl Into<PathDir> for WindingSolidity {
    fn into(self) -> PathDir {
        match self {
            WindingSolidity::Solid => PathDir::CCW,
            WindingSolidity::Hole => PathDir::CW,
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

#[derive(Debug)]
pub(crate) enum Command {
    MoveTo(Point),
    LineTo(Point),
    BezierTo(Point, Point, Point),
    Close,
    Winding(PathDir),
}
