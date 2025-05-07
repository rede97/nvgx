use nvg::{Context, RendererDevice};
use winit;

cfg_if::cfg_if! {
    if #[cfg(feature="ogl-impl")] {
        mod ogl;
        pub use ogl::run;
    } else if #[cfg(feature="wgpu-impl")] {
        mod wgpu;
        pub use wgpu::run;
    }
}

pub trait Demo<R: RendererDevice> {
    fn init(&mut self, ctx: &mut Context<R>, _scale_factor: f32) -> anyhow::Result<()> {
        ctx.create_font_from_file("roboto", "nvg-gl/examples/Roboto-Bold.ttf")?;
        Ok(())
    }

    fn before_frame(&mut self, _ctx: &mut Context<R>) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, _width: f32, _height: f32, _ctx: &mut Context<R>) -> anyhow::Result<()> {
        Ok(())
    }

    fn cursor_moved(&mut self, _x: f32, _y: f32) {}

    fn mouse_event(&mut self, _btn: winit::event::MouseButton, _state: winit::event::ElementState) {
    }

    fn key_event(
        &mut self,
        _key: winit::keyboard::KeyCode,
        _state: winit::event::ElementState,
    ) {
    }

    fn mouse_wheel(&mut self, _delta: winit::event::MouseScrollDelta) {}
}
