use super::Renderer;
use nvg::{Context, Extent, ImageFlags, ImageId};

#[allow(unused)]
#[derive(Debug, Copy, Clone, Default)]
pub struct FrameBuffer {
    default_fbo: gl::types::GLint,
    pub fbo: gl::types::GLuint,
    pub rbo: gl::types::GLuint,
    pub texture: gl::types::GLuint,
    pub image: ImageId,
    pub width: u32,
    pub height: u32,
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
        let texture = ctx.renderer().textures.get(image).unwrap().tex;

        unsafe {
            let mut default_fbo: gl::types::GLint = 0;
            let mut default_rbo: gl::types::GLint = 0;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut default_fbo as *mut _);
            gl::GetIntegerv(gl::RENDERBUFFER_BINDING, &mut default_rbo as *mut _);

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

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                gl::BindFramebuffer(gl::FRAMEBUFFER, default_fbo as gl::types::GLuint);
                gl::BindRenderbuffer(gl::RENDERBUFFER, default_rbo as gl::types::GLuint);
                ctx.delete_image(image)?;
                gl::DeleteFramebuffers(1, &mut fbo);
                gl::DeleteRenderbuffers(1, &mut rbo);
                return Err(anyhow!("Failed to create framebuffer"));
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, default_fbo as gl::types::GLuint);
            gl::BindRenderbuffer(gl::RENDERBUFFER, default_rbo as gl::types::GLuint);
            return Ok(Self {
                default_fbo,
                fbo,
                rbo,
                texture,
                image,
                width: width as u32,
                height: height as u32,
            });
        }
    }

    pub fn size(&self) -> Extent {
        Extent {
            width: self.width as f32,
            height: self.height as f32,
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.default_fbo as gl::types::GLuint);
        }
    }

    pub fn delete(&mut self, ctx: &mut Context<Renderer>) -> anyhow::Result<()> {
        unsafe {
            gl::DeleteFramebuffers(1, &mut self.fbo);
            gl::DeleteRenderbuffers(1, &mut self.rbo);
        }
        ctx.delete_image(self.image)?;
        Ok(())
    }
}
