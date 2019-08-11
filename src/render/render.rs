use gl;
use gl::types::*;
use nalgebra_glm as glm;

use std::mem;
use std::ptr;
use std::collections::HashMap;
use std::ffi::CString;

use crate::pane::Pane;
use crate::config;
use super::font::{self, RasterizedGlyph, Rasterizer, FontKey, GlyphKey, FontSize, FontDesc};

use super::shader::{TextShader, RectShader};
use super::{Error, Result, Glyph, GlyphCache};
use super::caches::Atlas;

#[macro_export]
macro_rules! glCheck {
    () => {{
        if cfg!(debug_assertions) {
            let err = unsafe { gl::GetError() };
            // println!("Error {:?}", err);
            if err != gl::NO_ERROR {
                let err_str = match err {
                    gl::INVALID_ENUM => "GL_INVALID_ENUM",
                    gl::INVALID_VALUE => "GL_INVALID_VALUE",
                    gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
                    gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
                    gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
                    gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
                    gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
                    _ => "unknown error",
                };
                println!("{}:{} error {}", file!(), line!(), err_str);
            }
        }
    }};
}



static BATCH_SIZE: usize = 1024;

#[derive(Debug, Clone, Copy)]
pub struct InstanceData {
    // cell
    pub x: f32,
    pub y: f32,
    
    // glyth info
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,

    // texture coordinates
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_dx: f32,
    pub uv_dy: f32,

    pub tr: f32,
    pub tg: f32,
    pub tb: f32,
    
    pub br: f32,
    pub bg: f32,
    pub bb: f32,

    pub texture_id: i32
}

pub struct Batch {
    texture_id: u32,
    instances: Vec<InstanceData>
}

impl Batch {
    pub fn new() -> Self {
        Self {
            texture_id: 0,
            instances: Vec::with_capacity(BATCH_SIZE)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn push(&mut self, data: InstanceData) -> bool {

        if self.is_empty() {
            self.texture_id = data.texture_id as u32
        }

        if self.instances.len() < BATCH_SIZE {
            self.instances.push(data)
        }

        self.instances.len() == BATCH_SIZE
    }

    pub fn clear(&mut self) {
        self.instances.clear()
    }

}

pub struct Renderer {
    // vertex buffer object
    vbo: u32,
    // index buffer object
    ibo: u32,
    // vertex attribute object
    vao: u32,
    // the shader for rendering text
    text_shader: TextShader,
    // the shader for rendering text
    rect_shader: RectShader,
    // all of the vertex atlases
    atlases: Vec<Atlas>,
}

impl Renderer {
    pub fn new(config: &config::Config) -> Result<Self> {
        let mut vao = 0;

        let mut bufs = [0, 0];
        // bufs[0] is vbo
        // bufs[1] is ibo

        let index_data = [0, 1, 2, 0, 2, 3];

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            // generate both with a single call
            gl::GenBuffers(2, bufs.as_mut_ptr());

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, bufs[1]);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mem::size_of::<u32>() * index_data.len()) as isize,
                index_data.as_ptr() as *const _,
                gl::STATIC_DRAW);

            gl::BindBuffer(gl::ARRAY_BUFFER, bufs[0]);

            let size = mem::size_of::<InstanceData>() as usize;

            gl::BufferData(gl::ARRAY_BUFFER,
                (size * BATCH_SIZE) as isize,
                ptr::null(),
                gl::STATIC_DRAW
                );

            let float_size = mem::size_of::<f32>();

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2 as i32, gl::FLOAT, gl::FALSE, size as i32, ptr::null());
            gl::VertexAttribDivisor(0, 1);
            glCheck!();

            let mut stride = 2;

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 4 as i32, gl::FLOAT, gl::FALSE, size as i32, (stride * float_size) as *const _);
            gl::VertexAttribDivisor(1, 1);
            glCheck!();
        
            stride += 4;

            // color attribute
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 4 as i32, gl::FLOAT, gl::FALSE, size as i32, (stride * float_size) as *const _);
            gl::VertexAttribDivisor(2, 1);
            glCheck!();

            stride += 4;

            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 3 as i32, gl::FLOAT, gl::FALSE, size as i32, (stride * float_size) as *const _);
            gl::VertexAttribDivisor(3, 1);

            stride += 3;

            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(4, 3 as i32, gl::FLOAT, gl::FALSE, size as i32, (stride * float_size) as *const _);
            gl::VertexAttribDivisor(4, 1);

            glCheck!();

            gl::BindVertexArray(0);
        }

        let text_shader = TextShader::new()?;
        let rect_shader = RectShader::new()?;

        let atlas_size = crate::window::Size::from(config.atlas.size, config.atlas.size);
        let atlas = Atlas::new(atlas_size)?;

        Ok(Self {
           vao,
           vbo: bufs[0],
           ibo: bufs[1],
           text_shader,
           rect_shader,
           atlases: vec![atlas]
        })
    }

    pub fn text_shader(&self) -> &TextShader {
        &self.text_shader
    }

    pub fn rect_shader(&self) -> &RectShader {
        &self.rect_shader
    }

    pub fn push_atlas(&mut self, atlas: Atlas) -> u32{
        self.atlases.push(atlas);
        (self.atlases.len() - 1) as u32
    }


    pub fn draw_batch(&self, batch: &Batch) -> Result<()> {
        if !batch.is_empty() {
            let atlas = &self.atlases[batch.texture_id as usize];
            atlas.bind();

            self.text_shader.activate();
            self.text_shader.set_font_atlas(atlas);

            unsafe {
                gl::BindVertexArray(self.vao);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

                glCheck!();

                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (mem::size_of::<InstanceData>() * BATCH_SIZE) as isize,
                    batch.instances.as_ptr() as *const _,
                    gl::STREAM_DRAW
                );
                glCheck!();

                self.text_shader.set_background_pass(0);
                gl::Enable(gl::BLEND);
                gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), batch.instances.len() as i32);
                gl::Disable(gl::BLEND);

                // self.text_shader.set_background_pass(1);
                // gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), batch.instances.len() as i32);

                glCheck!();

                gl::BindVertexArray(0);
            }
            atlas.unbind();
            self.text_shader.deactivate();
        }
        
        Ok(())
    }

    pub fn prepare_font(&mut self, dpi: f32, config: &config::Config) -> Result<super::GlyphCache<font::FreeTypeRasterizer>>  {
        let mut rasterizer = font::FreeTypeRasterizer::new(dpi).map_err(|e| Error::FontError(e))?;

        let mut cache = GlyphCache::new(
            rasterizer, 
            config.font.clone(),
            dpi,
            super::CacheMissProto::ErrorOnMiss
            )?;
        
        let mut loader = super::LoadApi::new(&mut self.atlases);
        cache.load_glyphs(&mut loader);

        Ok(cache)
    }
}