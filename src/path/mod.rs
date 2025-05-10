use crate::{RendererDevice, Transform};
use cache::PathSlice;
use core::f32;
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

pub(crate) mod cache;
mod commands;
mod transform;

pub use commands::*;

bitflags! {
    pub struct DrawPathStyle: u32 {
        const FILL = 0x1;
        const STROKE = 0x2;
        #[cfg(feature = "wirelines")]
        const WIRELINES = 0x4;
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DrawSlice {
    pub fill: Vec<PathSlice>,
    pub bounds_offset: Option<usize>,
    pub stroke: Vec<PathSlice>,
    pub lines: Vec<PathSlice>,
}

impl DrawSlice {
    pub fn is_none(&self) -> bool {
        #[cfg(feature = "wirelines")]
        return self.fill.is_empty() && self.stroke.is_empty() && self.lines.is_empty();
        #[cfg(not(feature = "wirelines"))]
        return self.fill.is_empty() && self.stroke.is_empty();
    }

    pub fn clear(&mut self) {
        self.fill.clear();
        self.bounds_offset = None;
        self.stroke.clear();
        self.lines.clear();
    }
}

pub(crate) struct PathMutInner<R: RendererDevice> {
    pub draw_slice: DrawSlice,
    pub vertex_buffer: Option<R::VertexBuffer>,
    pub style: DrawPathStyle,
}

impl<R: RendererDevice> Default for PathMutInner<R> {
    fn default() -> Self {
        PathMutInner {
            draw_slice: DrawSlice::default(),
            vertex_buffer: None,
            style: DrawPathStyle::empty(),
        }
    }
}

#[allow(unused)]
pub struct Path<R: RendererDevice> {
    pub(crate) path_comands: PathCommands,
    pub(crate) inner: RefCell<PathMutInner<R>>,
}

impl<R: RendererDevice> Clone for Path<R> {
    fn clone(&self) -> Self {
        return Path {
            path_comands: self.path_comands.clone(),
            inner: RefCell::new(PathMutInner::default()),
        };
    }
}

impl<R: RendererDevice> From<PathCommands> for Path<R> {
    fn from(path: PathCommands) -> Self {
        return Self {
            path_comands: path,
            inner: RefCell::new(PathMutInner::default()),
        };
    }
}

impl<R: RendererDevice> Path<R> {
    pub fn new() -> Self {
        return Self::from(PathCommands::default());
    }

    pub(crate) fn inner_cached_style(&self) -> DrawPathStyle {
        return self.inner.borrow().style;
    }

    pub fn reset(&mut self) -> &mut PathCommands {
        {
            let mut inner = self.inner.borrow_mut();
            inner.style = DrawPathStyle::empty();
            inner.draw_slice.clear();
            self.path_comands.clear();
        }
        &mut self.path_comands
    }
}

impl<R: RendererDevice> Deref for Path<R> {
    type Target = PathCommands;
    fn deref(&self) -> &Self::Target {
        return &self.path_comands;
    }
}

impl<R: RendererDevice> DerefMut for Path<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.inner.borrow().draw_slice.is_none() {
            return &mut self.path_comands;
        }
        panic!("Path must be reset before Updating")
    }
}
