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



static BATCH_SIZE: usize = 1024;

#[derive(Debug, Clone, Copy)]
struct InstanceData {
    // cell
    x: f32,
    y: f32,
    
    // glyth info
    width: f32,
    height: f32,
    offset_x: f32,
    offset_y: f32,

    // texture coordinates
    uv_x: f32,
    uv_y: f32,
    uv_dx: f32,
    uv_dy: f32,
    // Mayby this could be used if I move to a texture array of atlases?.
    // texture_id: f32,

    // text metrics offsets for the character

    r: f32,
    g: f32,
    b: f32,

    texture_id: i32
}

struct Batch {
    texture_id: u32,
    instances: Vec<InstanceData>
}

impl Batch {
    fn new() -> Self {
        Self {
            texture_id: 0,
            instances: Vec::with_capacity(BATCH_SIZE)
        }
    }

    fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    fn push(&mut self, data: InstanceData) -> bool {

        if self.is_empty() {
            self.texture_id = data.texture_id as u32
        }

        if self.instances.len() < BATCH_SIZE {
            self.instances.push(data)
        }

        self.instances.len() == BATCH_SIZE
    }

    fn clear(&mut self) {
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
    // batch for this renderer
    batch: Batch
}

impl Renderer {
    pub fn new() -> Result<Self> {
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
            // glCheck!();
            let mut stride = 2;

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3 as i32, gl::FLOAT, gl::FALSE, size as i32, (stride * float_size) as *const _);
            gl::VertexAttribDivisor(1, 1);
        
            stride += 3;

            // color attribute
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 4 as i32, gl::FLOAT, gl::FALSE, size as i32, (stride * float_size) as *const _);
            gl::VertexAttribDivisor(2, 1);
            // glCheck!();

            stride += 4;

            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 4 as i32, gl::FLOAT, gl::FALSE, size as i32, (stride * float_size) as *const _);
            gl::VertexAttribDivisor(3, 1);
            // glCheck!();

            // glCheck!();

            gl::BindVertexArray(0);
        }

        let text_shader = TextShader::new()?;
        let rect_shader = RectShader::new()?;

        Ok(Self {
           vao,
           vbo: bufs[0],
           ibo: bufs[1],
           text_shader,
           rect_shader,
           atlases: Vec::new(),
           batch: Batch::new()
        })
    }

    pub fn text_shader(&self) -> &TextShader {
        &self.text_shader
    }

    pub fn rect_shader(&self) -> &RectShader {
        &self.rect_shader
    }

    pub fn add_atlas(&mut self, atlas: Atlas) -> u32{
        self.atlases.push(atlas);
        (self.atlases.len() - 1) as u32
    }


    pub fn draw(&self) -> Result<()> {

        if !self.batch.is_empty() {
            self.text_shader.activate();


            let atlas = &self.atlases[self.batch.texture_id as usize];

            self.text_shader.set_font_atlas(atlas);
            atlas.bind();

            unsafe {
                gl::BindVertexArray(self.vao);

                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (mem::size_of::<InstanceData>() * BATCH_SIZE) as isize,
                    self.batch.instances.as_ptr() as *const _,
                    gl::STATIC_DRAW
                );

                gl::Enable(gl::BLEND);
                gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), self.batch.instances.len() as i32);

                gl::BindVertexArray(0);
            }

            atlas.unbind();
            self.text_shader.deactivate();
        }
        
        Ok(())
    }
}