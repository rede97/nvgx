use crate::renderer::TextureType;
use crate::Point;

mod composite;
mod core;
mod core_font;
mod core_image;
mod core_path;
mod paint;
mod path;

pub use composite::*;
pub use core::*;
pub use paint::*;

pub type ImageId = usize;

const KAPPA90: f32 = 0.5522847493;

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

bitflags! {
    pub struct Align: u32 {
        const LEFT = 0x1;
        const CENTER = 0x2;
        const RIGHT = 0x4;
        const TOP = 0x8;
        const MIDDLE = 0x10;
        const BOTTOM = 0x20;
        const BASELINE = 0x40;
    }
}

bitflags! {
    pub struct ImageFlags: u32 {
        /// Generate mipmaps during creation of the image.
        const GENERATE_MIPMAPS = 0x1;
        /// Repeat image in X direction.
        const REPEATX = 0x2;
        /// Repeat image in Y direction.
        const REPEATY = 0x4;
        /// Flips (inverses) image in Y direction when rendered.
        const FLIPY	= 0x8;
        /// Image data has premultiplied alpha.
        const PREMULTIPLIED = 0x10;
        /// Image interpolation is Nearest instead Linear
        const NEAREST = 0x20;
    }
}


#[derive(Copy, Clone)]
pub struct TextMetrics {
    pub ascender: f32,
    pub descender: f32,
    pub line_gap: f32,
}

impl TextMetrics {
    pub fn line_height(&self) -> f32 {
        self.ascender - self.descender + self.line_gap
    }
}

#[derive(Debug)]
pub(crate) enum Command {
    MoveTo(Point),
    LineTo(Point),
    BezierTo(Point, Point, Point),
    Close,
    Winding(PathDir),
}
