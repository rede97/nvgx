use nvg::{Extent, FrameBufferDevice, ImageId, RenderFrameBufferDevice};

use super::{texture::StencilTexture, Renderer};

pub struct FrameBuffer {
    stencil: StencilTexture,
    image: ImageId,
    width: u32,
    height: u32,
}

impl FrameBufferDevice for FrameBuffer {
    fn image(&self) -> ImageId {
        return self.image;
    }

    fn size(&self) -> Extent {
        Extent {
            width: self.width as f32,
            height: self.height as f32,
        }
    }
}

impl RenderFrameBufferDevice for Renderer {
    type FB = FrameBuffer;

    fn create_fb(&mut self, width: u32, height: u32, image: ImageId) -> anyhow::Result<Self::FB> {
        let stencil = StencilTexture::new(&self.device, width, height);
        Ok(FrameBuffer {
            stencil,
            image,
            width,
            height,
        })
    }

    fn delete_fb(&mut self, _fb: Self::FB) -> anyhow::Result<()> {
        Ok(())
    }

    fn bind(&mut self, fb: &Self::FB) -> anyhow::Result<()> {
        self.target_fb = Some((fb.image(), fb.stencil.view.clone()));
        Ok(())
    }

    fn unbind(&mut self) -> anyhow::Result<()> {
        self.target_fb = None;
        Ok(())
    }
}
