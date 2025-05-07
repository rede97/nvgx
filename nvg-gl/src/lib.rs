use nvg::{Color, Transform};

#[macro_use]
#[allow(unused)]
extern crate anyhow;

pub struct RenderConfig {
    antialias: bool,
}

impl RenderConfig {
    pub fn antialias(mut self, antialias: bool) -> Self {
        self.antialias = antialias;
        self
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            antialias: true,
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature="ogl-impl")] {
        /// OpenGL implement of NanoVG
        mod ogl;
        pub use ogl::*;
    } else if #[cfg(feature="wgpu-impl")] {
        /// WGPu implement of NanoVG
        mod wgpu;
        pub use wgpu::*;
    }
}

#[inline]
fn premul_color(color: Color) -> Color {
    Color {
        r: color.r * color.a,
        g: color.g * color.a,
        b: color.b * color.a,
        a: color.a,
    }
}

#[inline]
fn xform_to_3x4(xform: Transform) -> [f32; 12] {
    let mut m = [0f32; 12];
    let t = &xform.0;
    m[0] = t[0];
    m[1] = t[1];
    m[2] = 0.0;
    m[3] = 0.0;
    m[4] = t[2];
    m[5] = t[3];
    m[6] = 0.0;
    m[7] = 0.0;
    m[8] = t[4];
    m[9] = t[5];
    m[10] = 1.0;
    m[11] = 0.0;
    m
}
