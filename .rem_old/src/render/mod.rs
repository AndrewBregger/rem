pub mod caches;
pub mod framebuffer;
pub mod render;

pub use caches::*;
pub(crate) mod shader;

use crate::font;
use crate::font::GlyphKey;

use gl;
use gl::types::*;
use nalgebra_glm as glm;

use std::collections::HashMap;
use std::ffi::CString;
use std::mem;
use std::ptr;

use crate::config;
use crate::size;
use crate::window::Window;

use font::{self, FontDesc, FontKey, FontSize, GlyphKey, RasterizedGlyph, Rasterizer};
use framebuffer::{self, FrameBuffer};
use shader::{RectShader, TextShader};

#[derive(Debug, Clone)]
pub enum Error {
    FontError(font::Error),
    AtlasFull,
    RenderError(String),
    AtlasError(String),
    CacheMissChar(GlyphKey),
    FrameBufferError(framebuffer::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;

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
                trace!("{}:{} error {}", file!(), line!(), err_str);
                panic!();
            }
        }
    }};
}

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    /// Character this glyph represents
    pub ch: u32,
    /// The id of the atlas this glyph is located
    pub atlas: u32,
    /// Width of the glyph in pixels
    pub width: f32,
    /// Height of the glyph in pixels
    pub height: f32,
    /// The x uv coordinate of the glyph in the atlas
    pub uv_x: f32,
    /// The y uv coordinate of the glyph in the atlas
    pub uv_y: f32,
    /// The width in texture coordinate space (delta x)
    pub uv_dx: f32,
    /// The height in texture coordinate space (delta y)
    pub uv_dy: f32,
    /// top of glyph
    pub top: f32,
    /// left of glyph
    pub left: f32,
    /// advance x
    pub advance_x: f32,
    /// advance y
    pub advance_y: f32,
    /// x bearing of the character
    pub bearing_x: f32,
    /// y bearing of the character
    pub bearing_y: f32,
}


// static mut CURRENT_TIME: std::time::Instant = std::time::Instant::new(0, 0); // = std::time::Instant::now();

static BATCH_SIZE: usize = 1024;

#[derive(Debug, Clone, Copy, Default)]
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
    pub ba: f32,

    pub texture_id: i32,
}

pub struct Batch {
    texture_id: u32,
    instances: Vec<InstanceData>,
}

impl Batch {
    pub fn new() -> Self {
        Self {
            texture_id: 0,
            instances: Vec::with_capacity(BATCH_SIZE),
        }
    }

    pub fn is_full(&self) -> bool {
        self.instances.len() == BATCH_SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn push(&mut self, data: InstanceData)  {
        if self.is_empty() {
            self.texture_id = data.texture_id as u32
        }

        if self.instances.len() < BATCH_SIZE {
            self.instances.push(data)
        }
    }

    pub fn push_background_pass_data(&mut self, x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) {
        let mut data = InstanceData::default();
        data.x = x;
        data.y = y;
        data.br = r;
        data.bg = g;
        data.bb = b;
        data.ba = a;

        self.instances.push(data);
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
    // instance batch
    batch: Batch
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
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, bufs[0]);

            let size = mem::size_of::<InstanceData>() as usize;

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size * BATCH_SIZE) as isize,
                ptr::null(),
                gl::STATIC_DRAW,
            );

            let float_size = mem::size_of::<f32>();

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2 as i32, gl::FLOAT, gl::FALSE, size as i32, ptr::null());
            gl::VertexAttribDivisor(0, 1);
            glCheck!();

            let mut stride = 2;

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                4 as i32,
                gl::FLOAT,
                gl::FALSE,
                size as i32,
                (stride * float_size) as *const _,
            );
            gl::VertexAttribDivisor(1, 1);
            glCheck!();

            stride += 4;

            // color attribute
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                4 as i32,
                gl::FLOAT,
                gl::FALSE,
                size as i32,
                (stride * float_size) as *const _,
            );
            gl::VertexAttribDivisor(2, 1);
            glCheck!();

            stride += 4;

            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(
                3,
                3 as i32,
                gl::FLOAT,
                gl::FALSE,
                size as i32,
                (stride * float_size) as *const _,
            );
            gl::VertexAttribDivisor(3, 1);
            glCheck!();

            stride += 3;

            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(
                4,
                4 as i32,
                gl::FLOAT,
                gl::FALSE,
                size as i32,
                (stride * float_size) as *const _,
            );
            gl::VertexAttribDivisor(4, 1);

            glCheck!();
        }

        let text_shader = TextShader::new()?;
        let rect_shader = RectShader::new()?;

        let atlas_size = size::Size::new(config.atlas.size, config.atlas.size);
        let atlas = Atlas::new(atlas_size)?;

        Ok(Self {
            vao,
            vbo: bufs[0],
            ibo: bufs[1],
            text_shader,
            rect_shader,
            atlases: vec![atlas],
        })
    }

    pub fn push_instance(&mut self, data: InstanceData) {
        if self.batch.is_full() {
            self.
        }
        self.batch.push(data);
    }

    pub fn prepare(&self) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
            gl::Enable(gl::MULTISAMPLE);
            gl::DepthMask(gl::FALSE);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        }
    }

    pub fn text_shader(&self) -> &TextShader {
        &self.text_shader
    }

    pub fn rect_shader(&self) -> &RectShader {
        &self.rect_shader
    }

    pub fn push_atlas(&mut self, atlas: Atlas) -> u32 {
        self.atlases.push(atlas);
        (self.atlases.len() - 1) as u32
    }

    pub fn set_view_port(&self, width: f32, height: f32) {
        self.set_view_port_at(width, height, 0.0, 0.0);
    }

    pub fn set_view_port_at(&self, width: f32, height: f32, x: f32, y: f32) {
        unsafe {
            gl::Viewport(x as i32, y as i32, width as i32, height as i32);
        }
    }

    pub fn draw_batch(&mut self) -> Result<()> {
        let batch = &self.batch;

        if !batch.is_empty() {
            glCheck!();
            self.text_shader.activate();
            glCheck!();

            self.text_shader
                .set_font_atlas_texture(batch.texture_id as i32);
            glCheck!();

            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + batch.texture_id);
                glCheck!();
                gl::BindTexture(gl::TEXTURE_2D, batch.texture_id);
                glCheck!();

                gl::BindVertexArray(self.vao);
                glCheck!();

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
                glCheck!();

                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                glCheck!();

                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (mem::size_of::<InstanceData>() * BATCH_SIZE) as isize,
                    batch.instances.as_ptr() as *const _,
                    gl::STREAM_DRAW,
                );
                glCheck!();

                //let mut fbo = 0;
                //gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut fbo);

                self.text_shader.set_background_pass(0);
                gl::DrawElementsInstanced(
                    gl::TRIANGLES,
                    6,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                    batch.instances.len() as i32,
                );

                // self.text_shader.set_background_pass(1);
                // gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), batch.instances.len() as i32);

                glCheck!();

                gl::BindVertexArray(0);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            self.text_shader.deactivate();
            self.batch.clear();
        }

        Ok(())
    }

    pub fn render_background_pass(&mut self) -> Result<()> {
        self.text_shader.activate();
        let batch = &self.batch;

        unsafe {
            gl::BindVertexArray(self.vao);
            glCheck!();

            // gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            // glCheck!();
            // gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            // glCheck!();

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mem::size_of::<InstanceData>() * BATCH_SIZE) as isize,
                batch.instances.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );
            glCheck!();
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                ptr::null(),
                batch.instances.len() as i32,
            );
        }
        self.text_shader.deactivate();
        self.batch.clear();

        Ok(())
    }

    pub fn prepare_font(
        &mut self,
        dpi: f32,
        config: &config::Config,
    ) -> Result<super::GlyphCache<font::FreeTypeRasterizer>> {
        let mut rasterizer = font::FreeTypeRasterizer::new(dpi).map_err(|e| Error::FontError(e))?;

        let mut cache = GlyphCache::new(
            rasterizer,
            config.font.clone(),
            dpi,
            super::CacheMissProto::ErrorOnMiss,
        )?;

        let mut loader = super::LoadApi::new(&mut self.atlases);
        cache.load_glyphs(&mut loader);

        Ok(cache)
    }

    /// maybe give this function a framebuffer object that is should clear.
    /// This is to explicity giving the function a specific buffer to clear
    /// not just the currently one bound.
    pub fn clear_frame(&self, frame: Option<&FrameBuffer>) {
        let mut saved_fbo = 0;
        unsafe {
            // this is so the state can be restored after this call.
            gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut saved_fbo);
        }

        match frame {
            Some(frame) => {
                frame.bind_write();
            }
            _ => { /* clear currently active draw buffer*/ }
        }
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, saved_fbo as u32);
        }
    }

    /// draws the background of a pane.
    /// I am passing in batch to reduce the number of allocations
    pub fn draw_pane_background(&mut self, pane: &Pane, cursor: &Cursor) {
        // temporary background color
        const R: f32 = 33f32 / 255f32;
        const G: f32 = 33f32 / 255f32;
        const B: f32 = 33f32 / 255f32;

        let pane_size = pane.cells();

        self.text_shader.activate();
        self.text_shader.set_background_pass(1);
        self.text_shader.deactivate();

        let batch = &mut self.batch;

        for i in 0..=pane_size.x {
            for j in 0..=pane_size.y {
                batch.push_background_pass_data(i as f32, j as f32, R, G, B, 1.0);

                if batch.is_full() {
                    self.render_background_pass(&batch);
                    batch.clear();
                }
            }
        }

        // if it is full then draw it.
        if batch.is_full() {
            self.render_background_pass();
            batch.clear();
        }

        // if duration.as_millis() >= CURSOR_TIME.as_millis() {
        batch.push_background_pass_data(
            cursor.pos().x as f32,
            cursor.pos().y as f32,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        // }

        self.render_background_pass();
        batch.clear();
    }

    /// assumes the frame buffer has been rendered to and ready to be drawn.
    /*
    pub fn draw_rendered_pane(&self, window: &Window, pane: &Pane, state: &PaneState) {
        let (w, h): (i32, i32) = if let Some(s) = window.get_inner_size() {
            let s = s.to_physical(window.dpi_factor());
            (s.width as i32, s.height as i32)
        } else {
            unreachable!();
        };

        // println!("{} {}", w, h);
        let pane_size = pane.size();
        // println!("{:?}", pane_size);
        let pane_loc = pane.loc();

        state.frame.bind_read();

        unsafe {
            // bind the window as the target draw.
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);

            gl::BlitFramebuffer(
                pane_loc.x as i32,
                pane_loc.y as i32,
                w,
                h,
                0,
                0,
                pane_size.x as i32,
                pane_size.y as i32,
                gl::COLOR_BUFFER_BIT,
                gl::LINEAR,
            );
            // reset the state.
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
    */
}