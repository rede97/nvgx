#[macro_use]
extern crate bitflags;

mod color;
mod context;
mod fonts;
mod math;
mod paint;
mod path;
mod renderer;

pub use color::*;
pub use context::*;
pub use fonts::*;
pub use math::*;
pub use paint::*;
pub use path::*;
pub use renderer::*;

pub mod utils {
    use crate::{Color, Transform};

    #[inline]
    pub fn xform_to_3x4(xform: Transform) -> [f32; 12] {
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

    #[inline]
    pub fn premul_color(color: Color) -> Color {
        Color {
            r: color.r * color.a,
            g: color.g * color.a,
            b: color.b * color.a,
            a: color.a,
        }
    }
}
