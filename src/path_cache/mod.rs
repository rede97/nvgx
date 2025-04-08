use crate::context::{Path, Vertex};
use crate::{Bounds, Point};

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
    pub(crate) paths: Vec<Path>,
    pub(crate) vertexes: Vec<Vertex>,
    pub(crate) bounds: Bounds,
}
