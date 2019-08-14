use gl;
use gl::types::*;

use crate::size;

#[derive(Debug)]
pub enum Error {
    Incomplete,
    ToManyBuffers,
}

pub type FrameSize = size::Size<f32>;
pub type Result<T> = ::std::result::Result<T, Error>;

/// Abstraction of a frame buffer to be used when rendering a pane.
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    fbo: u32,
    render_buffer: u32,
    size: FrameSize,
}

impl FrameBuffer {
    pub fn with_size(size: FrameSize) -> Result<Self> {
        let mut fbo = 0;
        let mut render_buffer = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            gl::GenRenderbuffers(1, &mut render_buffer);
            gl::BindRenderbuffer(gl::RENDERBUFFER, render_buffer);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::RGBA, size.x as i32, size.y as i32);
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, render_buffer);

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                return Err(Error::Incomplete);
            }
        }

        Ok(Self {
            fbo,
            render_buffer,
            size
        })
    }


    pub fn clear(&self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo);
            gl::DeleteRenderbuffers(1, &self.render_buffer);
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        }
    }

    pub fn bind_read(&self) {
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo);
        }
    }

    pub fn bind_write(&self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}