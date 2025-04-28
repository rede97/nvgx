use nvg::*;
use slab::Slab;

mod renderer;

pub struct Renderer {}

impl Renderer {
    pub fn create(window: &winit::window::Window) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
    }
}
