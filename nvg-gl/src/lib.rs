#[macro_use]
#[allow(unused)]
extern crate anyhow;

cfg_if::cfg_if! {
    if #[cfg(feature="ogl-impl")] {
        /// OpenGL implement of NanoVG
        mod ogl;
        pub use ogl::*;
    } else if #[cfg(feature="wgpu-impl")] {
        mod wgpu;
        pub use wgpu::*;
    }
}
