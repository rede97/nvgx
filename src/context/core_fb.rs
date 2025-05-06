use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{FrameBufferDevice, RenderFrameBufferDevice};

use super::{Context, ImageFlags};

pub struct FrameBufferContext<'a, 'b, R: RenderFrameBufferDevice> {
    context: &'a mut Context<R>,
    _fb_marker: std::marker::PhantomData<&'b R::FB>,
}

impl<'a, 'b, R: RenderFrameBufferDevice> FrameBufferContext<'a, 'b, R> {
    fn new(context: &'a mut Context<R>, fb: &'b R::FB) -> anyhow::Result<Self> {
        context.renderer.bind(fb)?;
        return Ok(FrameBufferContext {
            context,
            _fb_marker: PhantomData::default(),
        });
    }
}

impl<'a, 'b, R: RenderFrameBufferDevice> Drop for FrameBufferContext<'a, 'b, R> {
    fn drop(&mut self) {
        self.context.renderer.unbind().unwrap();
    }
}

impl<'a, 'b, R: RenderFrameBufferDevice> Deref for FrameBufferContext<'a, 'b, R> {
    type Target = Context<R>;
    fn deref(&self) -> &Self::Target {
        return self.context;
    }
}

impl<'a, 'b, R: RenderFrameBufferDevice> DerefMut for FrameBufferContext<'a, 'b, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return self.context;
    }
}

impl<R: RenderFrameBufferDevice> Context<R> {
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
    pub fn bind<'a, 'b>(
        &'a mut self,
        fb: &'b R::FB,
    ) -> anyhow::Result<FrameBufferContext<'a, 'b, R>> {
        return FrameBufferContext::new(self, fb);
    }
}
