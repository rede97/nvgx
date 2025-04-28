#[macro_use]
extern crate anyhow;

cfg_if::cfg_if! {
    if #[cfg(feature="ogl")] {
        /// OpenGL implement of NanoVG
        mod ogl;
        pub use ogl::*;
    }
}
