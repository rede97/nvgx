use crate::{RendererDevice, Transform};
use cache::PathSlice;
use core::f32;
use std::ops::{Deref, DerefMut};

pub(crate) mod cache;
mod commands;
mod transform;

pub use commands::*;


#[derive(Debug, Clone, Default)]
pub(crate) struct DrawInfo {
    fill: Vec<PathSlice>,
    bounds_offset: Option<usize>,
    stroke: Vec<PathSlice>,
}

pub(crate) enum PathInner {
    Commands(PathCommands),
    /// save vertex draw call
    Lock(DrawInfo),
}

#[allow(unused)]
pub struct Path<R: RendererDevice> {
    inner: PathInner,
    vertex_buffer: Option<R::VertexBuffer>,
}

impl<R: RendererDevice> From<PathCommands> for Path<R> {
    fn from(value: PathCommands) -> Self {
        return Self {
            inner: PathInner::Commands(value),
            vertex_buffer: None,
        };
    }
}

impl<R: RendererDevice> Path<R> {
    pub fn new() -> Self {
        return Self::from(PathCommands::default());
    }

    pub fn reset(&mut self) {
        self.inner = PathInner::Commands(PathCommands::default());
    }
}

impl<R: RendererDevice> Deref for Path<R> {
    type Target = PathCommands;
    fn deref(&self) -> &Self::Target {
        match &self.inner {
            PathInner::Commands(path_commands) => {
                return &path_commands;
            }
            PathInner::Lock(_) => {
                panic!("Path must be reset before dereferencing to PathCommands.")
            }
        }
    }
}

impl<R: RendererDevice> DerefMut for Path<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.inner {
            PathInner::Commands(path_commands) => path_commands,
            PathInner::Lock(_) => {
                panic!("Path must be reset before dereferencing to PathCommands.")
            }
        }
    }
}
