use crate::{
    cache::PathCache, renderer::TextureType, LineCap, LineJoin, PathCommands, PathSlice,
};

mod composite;
mod core;
mod core_fb;
mod core_font;
mod core_image;
mod core_path;

pub use composite::*;
pub use core::*;
use std::ops::{Deref, DerefMut};

pub type ImageId = usize;

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

pub(crate) trait FlattenExpandPath {
    fn path_commands(&self) -> &PathCommands;

    fn flatten_paths(&mut self, dist_tol: f32, tess_tol: f32);

    fn expand_stroke(
        &mut self,
        w: f32,
        fringe: f32,
        line_cap: LineCap,
        line_join: LineJoin,
        miter_limit: f32,
        tess_tol: f32,
    );
    fn expand_fill(
        &mut self,
        w: f32,
        line_join: LineJoin,
        miter_limit: f32,
        fringe_width: f32,
    ) -> Option<usize>;
    #[cfg(feature = "wirelines")]
    fn expand_lines(&mut self);

    fn get_fill_slice(&self) -> &[PathSlice];
    fn get_stroke_slice(&self) -> &[PathSlice];
    #[cfg(feature = "wirelines")]
    fn get_lines_slice(&self) -> &[PathSlice];
}

#[derive(Default)]
pub(crate) struct PathCommandsWithCache {
    pub path: PathCommands,
    pub cache: PathCache,
    pub draw_paths_slice: Vec<PathSlice>,
}

impl FlattenExpandPath for PathCommandsWithCache {
    #[inline]
    fn path_commands(&self) -> &PathCommands {
        return &self.path;
    }

    #[inline]
    fn flatten_paths(&mut self, dist_tol: f32, tess_tol: f32) {
        self.cache
            .flatten_paths(&self.path.commands, dist_tol, tess_tol);
    }

    #[inline]
    fn expand_fill(
        &mut self,
        w: f32,
        line_join: LineJoin,
        miter_limit: f32,
        fringe_width: f32,
    ) -> Option<usize> {
        return self.cache.expand_fill(
            w,
            line_join,
            miter_limit,
            fringe_width,
            &mut self.draw_paths_slice,
        );
    }

    #[inline]
    fn expand_stroke(
        &mut self,
        w: f32,
        fringe: f32,
        line_cap: LineCap,
        line_join: LineJoin,
        miter_limit: f32,
        tess_tol: f32,
    ) {
        self.cache.expand_stroke(
            w,
            fringe,
            line_cap,
            line_join,
            miter_limit,
            tess_tol,
            &mut self.draw_paths_slice,
        );
    }

    #[inline]
    #[cfg(feature = "wirelines")]
    fn expand_lines(&mut self) {
        self.cache.expand_lines(&mut self.draw_paths_slice);
    }

    #[inline]
    fn get_fill_slice(&self) -> &[PathSlice] {
        return &self.draw_paths_slice;
    }

    #[inline]
    fn get_stroke_slice(&self) -> &[PathSlice] {
        return &self.draw_paths_slice;
    }

    #[inline]
    #[cfg(feature = "wirelines")]
    fn get_lines_slice(&self) -> &[PathSlice] {
        return &self.draw_paths_slice;
    }
}

impl Deref for PathCommandsWithCache {
    type Target = PathCommands;
    fn deref(&self) -> &Self::Target {
        return &self.path;
    }
}

impl DerefMut for PathCommandsWithCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.path;
    }
}
