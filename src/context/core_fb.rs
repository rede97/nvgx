use crate::{FrameBufferDevice, RenderFramebuffer};

use super::{Context, ImageFlags};

impl<R: RenderFramebuffer> Context<R> {
    #[inline]
    pub fn create_fb(
        &mut self,
        width: u32,
        height: u32,
        flags: ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<R::FB> {
        let image = self.create_image_rgba(
            width,
            height,
            flags | ImageFlags::FLIPY | ImageFlags::PREMULTIPLIED,
            data,
        )?;
        match self.renderer.create_fb(width, height, image) {
            Ok(fb) => {
                return Ok(fb);
            }
            Err(e) => {
                self.renderer.delete_texture(image)?;
                return Err(e);
            }
        }
    }

    #[inline]
    pub fn delete_fb(&mut self, fb: R::FB) -> anyhow::Result<()> {
        let image = fb.image();
        self.renderer.delete_fb(fb)?;
        return self.delete_image(image);
    }

    #[inline]
    pub fn bind(&self, fb: &R::FB) -> anyhow::Result<()> {
        return self.renderer.bind(fb);
    }

    #[inline]
    pub fn unbind(&self) -> anyhow::Result<()> {
        return self.renderer.unbind();
    }
}
