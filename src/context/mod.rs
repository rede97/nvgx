use crate::{cache::PathCache, renderer::TextureType, BufferId, PathCommands};

mod composite;
mod core;
mod core_fb;
mod core_font;
mod core_image;
mod core_path;

pub use composite::*;
pub use core::*;
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

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

#[derive(Default)]
pub(crate) struct PathWithCache {
    pub path: PathCommands,
    pub cache: RefCell<PathCache>,
    pub vertex_buffer: BufferId,
}

impl Deref for PathWithCache {
    type Target = PathCommands;
    fn deref(&self) -> &Self::Target {
        return &self.path;
    }
}

impl DerefMut for PathWithCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.path;
    }
}
