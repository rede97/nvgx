use nvgx::{Context, RendererDevice};
use winit;

#[macro_use]
#[allow(unused)]
extern crate anyhow;

const DEFAULT_SIZE: (i32, i32) = (640, 480);
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


#[cfg(feature = "save-fps")]
#[derive(Default)]
struct SaveFPS {
    pub name: String,
    pub data: Vec<f32>,
    pub idx: usize,
}

#[cfg(feature = "save-fps")]
impl SaveFPS {
    fn push(&mut self, fps: f32) {
        if self.idx < 1024 {
            self.data.push(fps);
        } else {
            self.data[self.idx % 1024] = fps;
        }
        self.idx += 1;
    }
}

#[cfg(feature = "save-fps")]
impl Drop for SaveFPS {
    fn drop(&mut self) {
        use std::io::Write;
        let mut f = std::fs::File::create(format!("{}.csv", self.name)).unwrap();
        for fps in self.data.iter() {
            writeln!(f, "{}", fps).unwrap();
        }
    }
}
