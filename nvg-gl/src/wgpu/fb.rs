use nvg::{Context, Extent, ImageFlags, ImageId};
use wgpu::Extent3d;

use super::Renderer;

pub struct FrameBuffer {
    image: ImageId,
    size: Extent3d,
}

#[allow(unused)]
impl FrameBuffer {
    pub fn new(
        ctx: &mut Context<Renderer>,
        width: u32,
        height: u32,
        flags: ImageFlags,
    ) -> anyhow::Result<Self> {
        let mut fbo = 0;
        let mut rbo = 0;
        let image: ImageId = ctx.create_image_rgba(
            width,
            height,
            flags | ImageFlags::FLIPY | ImageFlags::PREMULTIPLIED,
            None,
        )?;
        let texture = ctx.renderer().texture_manager.get(image).unwrap();
        let size = texture.texture.size();

        return Ok(Self { image, size });
    }

    pub fn size(&self) -> Extent {
        Extent {
            width: self.size.width as f32,
            height: self.size.height as f32,
        }
    }

    
}
