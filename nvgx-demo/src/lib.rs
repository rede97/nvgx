use nvgx::{Context, RendererDevice};
use winit;

mod perf;

#[macro_use]
#[allow(unused)]
extern crate anyhow;

const DEFAULT_SIZE: (i32, i32) = (1280, 720);
pub const FONT_PATH: &str = "nvgx-demo/Roboto-Bold.ttf";
pub const IMG_PATH: &str = "nvgx-demo/lenna.png";

cfg_if::cfg_if! {
    if #[cfg(feature="ogl")] {
        mod ogl;
        pub use ogl::run;
        pub use nvgx_ogl as nvgx_impl;
    } else if #[cfg(feature="wgpu")] {
        mod wgpu;
        pub use wgpu::run;
        pub use nvgx_wgpu as nvgx_impl;
    }
}

pub trait Demo<R: RendererDevice> {
    fn init(&mut self, ctx: &mut Context<R>, _scale_factor: f32) -> anyhow::Result<()> {
        ctx.create_font_from_file("roboto", FONT_PATH)?;
        ctx.create_font_from_file("notocjk", "nvgx-demo/NotoSansCJK-Regular.otf")?;
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

    fn key_event(&mut self, _key: winit::keyboard::KeyCode, _state: winit::event::ElementState) {}

    fn mouse_wheel(&mut self, _delta: winit::event::MouseScrollDelta) {}
}
