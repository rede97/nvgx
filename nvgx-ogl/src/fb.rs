use super::Renderer;
use nvgx::{Extent, FrameBufferDevice, ImageId, RenderFrameBufferDevice};


#[allow(unused)]
#[derive(Debug, Copy, Clone)]
pub struct FrameBuffer {
    fbo: gl::types::GLuint,
    rbo: gl::types::GLuint,
    image: ImageId,
    width: u32,
    height: u32,
}

impl FrameBufferDevice for FrameBuffer {
    fn size(&self) -> Extent {
        Extent {
            width: self.width as f32,
            height: self.height as f32,
        }
    }

    fn image(&self) -> ImageId {
        return self.image;
    }
}

impl RenderFrameBufferDevice for Renderer {
    type FB = FrameBuffer;

    fn create_fb(&mut self, width: u32, height: u32, image: ImageId) -> anyhow::Result<Self::FB> {
        let mut fbo = 0;
        let mut rbo = 0;
        let texture = self.textures.get(image).unwrap().tex;

        unsafe {
            // framebuffer object
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            // renderbuffer object
            gl::GenRenderbuffers(1, &mut rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::STENCIL_INDEX8,
                width as i32,
                height as i32,
            );

            // Attach the texture to the framebuffer
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture,
                0,
            );
            // Attach the renderbuffer to the framebuffer
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                rbo,
            );

            let default_fbo = self.default_fbo.clone();

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                gl::BindFramebuffer(gl::FRAMEBUFFER, default_fbo.fbo as gl::types::GLuint);
                gl::BindRenderbuffer(gl::RENDERBUFFER, default_fbo.rbo as gl::types::GLuint);

                gl::DeleteFramebuffers(1, &mut fbo);
                gl::DeleteRenderbuffers(1, &mut rbo);
                return Err(anyhow!("Failed to create framebuffer"));
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, default_fbo.fbo as gl::types::GLuint);
            gl::BindRenderbuffer(gl::RENDERBUFFER, default_fbo.rbo as gl::types::GLuint);
            return Ok(Self::FB {
                fbo,
                rbo,
                image,
                width: width as u32,
                height: height as u32,
            });
        }
    }

    fn delete_fb(&mut self, mut fb: Self::FB) -> anyhow::Result<()> {
        unsafe {
            gl::DeleteFramebuffers(1, &mut fb.fbo);
            gl::DeleteRenderbuffers(1, &mut fb.rbo);
        }
        return Ok(());
    }

    fn bind(&mut self, fb: &Self::FB) -> anyhow::Result<()> {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fb.fbo);
        }
        Ok(())
    }

    fn unbind(&mut self) -> anyhow::Result<()> {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.default_fbo.fbo as gl::types::GLuint);
        }
        Ok(())
    }
}
