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

#[derive(Debug, Clone, Default)]
pub(crate) struct DrawInfo {
    pub fill: Vec<PathSlice>,
    pub bounds_offset: Option<usize>,
    pub stroke: Vec<PathSlice>,
    pub lines: Vec<PathSlice>,
}

pub(crate) struct PathMutInner<R: RendererDevice> {
    pub draw_info: Option<DrawInfo>,
    pub vertex_buffer: Option<R>,
}

#[allow(unused)]
pub struct Path<R: RendererDevice> {
    pub(crate) path: PathCommands,
    pub(crate) inner: RefCell<PathMutInner<R>>,
}

impl<R: RendererDevice> Clone for Path<R> {
    fn clone(&self) -> Self {
        return Path {
            path: self.path.clone(),
            inner: RefCell::new(PathMutInner {
                draw_info: None,
                vertex_buffer: None,
            }),
        };
    }
}

impl<R: RendererDevice> From<PathCommands> for Path<R> {
    fn from(path: PathCommands) -> Self {
        return Self {
            path,
            inner: RefCell::new(PathMutInner {
                draw_info: None,
                vertex_buffer: None,
            }),
        };
    }
}

impl<R: RendererDevice> Path<R> {
    pub fn new() -> Self {
        return Self::from(PathCommands::default());
    }

    pub fn reset(&mut self) {
        self.inner.borrow_mut().draw_info = None;
        self.path.clear();
    }
}

impl<R: RendererDevice> Deref for Path<R> {
    type Target = PathCommands;
    fn deref(&self) -> &Self::Target {
        return &self.path;
    }
}

impl<R: RendererDevice> DerefMut for Path<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.inner.borrow().draw_info.is_none() {
            return &mut self.path;
        }
        panic!("Path must be reset before Updating")
    }
}
