use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use crate::{RendererDevice, Transform};

pub(crate) struct InstancesInner<R: RendererDevice> {
    pub(crate) vertex_buffer: Option<R::VertexBuffer>,
}

pub struct Instances<R: RendererDevice> {
    pub(crate) transforms: Vec<Transform>,
    pub(crate) inner: RefCell<InstancesInner<R>>,
}

impl<R: RendererDevice> Instances<R> {
    pub fn new(transforms: Vec<Transform>) -> Self {
        return Self {
            transforms,
            inner: RefCell::new(InstancesInner {
                vertex_buffer: None,
            }),
        };
    }
}

impl<R: RendererDevice> Deref for Instances<R> {
    type Target = Vec<Transform>;
    fn deref(&self) -> &Self::Target {
        return &self.transforms;
    }
}

impl<R: RendererDevice> DerefMut for Instances<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.transforms;
    }
}
