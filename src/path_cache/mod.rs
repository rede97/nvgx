use crate::{Bounds, PathDir, Point};

mod cap_join;
mod draw_path;

bitflags! {
    #[derive(Default)]
    struct PointFlags: u32 {
        const PT_CORNER = 0x1;
        const PT_LEFT = 0x2;
        const PT_BEVEL = 0x4;
        const PR_INNERBEVEL	= 0x8;
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct VPoint {
    xy: Point,
    d: Point,
    len: f32,
    dm: Point,
    flags: PointFlags,
}

#[derive(Default)]
pub(crate) struct PathCache {
    pub(crate) points: Vec<VPoint>,
    pub(crate) paths: Vec<PathInfo>,
    pub(crate) vertexes: Vec<Vertex>,
    pub(crate) bounds: Bounds,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
}

impl Vertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32) -> Vertex {
        Vertex { x, y, u, v }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PathInfo {
    pub(crate) first: usize,
    pub(crate) count: usize,
    pub(crate) closed: bool,
    pub(crate) num_bevel: usize,
    pub(crate) windding: PathDir,
    pub(crate) fill: *mut Vertex,
    pub(crate) num_fill: usize,
    pub(crate) stroke: *mut Vertex,
    pub(crate) num_stroke: usize,
    #[cfg(feature = "wirelines")]
    pub(crate) lines: *mut Vertex,
    #[cfg(feature = "wirelines")]
    pub(crate) num_lines: usize,
    pub convex: bool,
}

impl PathInfo {
    pub fn get_fill(&self) -> &[Vertex] {
        if self.fill.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.fill, self.num_fill) }
        }
    }

    pub fn get_stroke(&self) -> &[Vertex] {
        if self.stroke.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.stroke, self.num_stroke) }
        }
    }

    #[cfg(feature = "wirelines")]
    pub fn get_lines(&self) -> &[Vertex] {
        if self.lines.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.lines, self.num_lines) }
        }
    }
}
