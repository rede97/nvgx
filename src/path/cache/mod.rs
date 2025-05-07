use std::fmt::Display;

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

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
}

impl Display for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "x: {}, y: {}, u: {}, v: {}",
            self.x, self.y, self.u, self.v
        )
    }
}

impl Vertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32) -> Vertex {
        Vertex { x, y, u, v }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct VertexSlice {
    pub offset: usize,
    pub count: usize,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct PathInfo {
    pub(crate) first: usize,
    pub(crate) count: usize,
    pub(crate) closed: bool,
    pub(crate) num_bevel: usize,
    pub(crate) windding: PathDir,
    pub(crate) offset: usize,
    pub(crate) num_fill: usize,
    pub(crate) num_stroke: usize,
    pub convex: bool,
}

impl PathInfo {
    pub fn get_fill(&self) -> VertexSlice {
        VertexSlice {
            offset: self.offset,
            count: self.num_fill,
        }
    }

    pub fn get_stroke(&self) -> VertexSlice {
        VertexSlice {
            offset: self.offset + self.num_fill,
            count: self.num_stroke,
        }
    }
}
