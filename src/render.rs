use gl::types::*;

use super::window::Size;
use super::font::{RasterizedGlyph, Rasterizer};

static INDEX_DATA: [u32; 6] = [0, 1, 2, 0, 2, 3];

#[derive(Debug, Clone)]
enum Error {
    FontError(String),
    RenderError(String),
    AtlasError(String),
}
type Result<T> = ::std::result::Result<T, Error>;

pub struct Renderer {
    vbo: u32,
    ibo: u32,
    vao: u32,
    shader: u32,
}

struct Glyph {
    /// Character this glyph represents
    ch: u32,
    /// The id of the atlas this glyph is located
    atlas: u32,
    /// Width of the glyph in pixels
    width: f32,
    /// Height of the glyph in pixels
    height: f32,
    /// The x uv coordinate of the glyph in the atlas
    uv_x: f32,
    /// The y uv coordinate of the glyph in the atlas
    uv_y: f32,
    /// The width in texture coordinate space (delta x)
    uv_dx: f32,
    /// The height in texture coordinate space (delta y)
    uv_dy: f32,
}

/////////////////////////////////////////////////////////
///
///
///
///             current x
///             v
/// +---------------------------------------------------+
/// |                                                   |
/// |                                                   |
/// |                                                   |
/// |                                                   |
/// |                                                   |
/// |---------------------------------------------------|
/// |           |                                       |
/// |     f     |                                       |
/// |           |                                       |
/// |---------------------------------------------------| < Base Line
/// |           |       |       |         |        |    |
/// |     a     |   b   |   c   |    d    |   e    |    | This line is full
/// |           |       |       |         |        |    | the next character
/// +---------------------------------------------------+ would not fit
///
///
///
///
///
///
///
/////////////////////////////////////////////////////////
struct Atlas {
    // the current x within the atlas
    x: f32,
    // the current baseline of the atlas
    base_line: f32,
    // the size of the underline texture
    size: Size,
    // the height of the larget subtexture in the current row.
    max_height: f32,
    // the gl texture handle
    texture_id: u32,
    // the altas id
    id: u32,
}

impl Atlas {
    // the gl texture is not allocated until a subtexture is added.
    fn new(size: Size) -> Result<Self> {
        Ok(Self {
            x: 0.0,
            base_line: 0.0,
            size,
            max_height: 0.0,
            texture_id: 0,
            id: 0 // I dont know how to set this.
        })
    }

    // allocates the gl texture
    fn allocate_texture(&mut self) -> Result<()> {
        unsafe {
            gl::GenTextures(1, &mut self.texture_id);

            if self.texture_id == 0 {
                return Err(Error::AtlasError(
                    format!("Failed to allocate texture of size {:?}", self.size).to_owned(),
                ));
            }

            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_BORDER as GLint,
            );

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_BORDER as GLint,
            );

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as GLint
            );

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as GLint
            );

            // allocate an empty texture
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                self.size.width() as i32,
                self.size.height() as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Ok(())
    }

    fn insert(&mut self, glyph: &RasterizedGlyph) -> Result<Glyph> {
        // move to next row if needed
        if !self.has_space(glyph) {
            self.advance()?;
        }
    
        // check if the glyph will fit vertically fix in the altas.
        if self.base_line + glyph.height > self.size.height() as f32 {
            return Err(Error::AtlasError(format!("Not enough room for glyph of size {} {} in altas", glyph.width, glyph.height).to_owned()));
        }
        
        // the glyph can be added.

        // if the new glyph is vertically largest glyph in the row then set
        // it to the new heightest.
        if glyph.height > self.max_height {
            self.max_height = glyph.height;
        }
    
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);

            gl::TexSubImage2D(gl::TEXTURE_2D,  
                              0,
                              self.x as i32,
                              self.base_line as i32,
                              glyph.width as i32,
                              glyph.height as i32,
                              gl::RGB,
                              gl::UNSIGNED_BYTE,
                              glyph.bitmap.as_ptr() as *const _);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let old_x = self.x;

        // move the x cursor forward.
        self.x += glyph.width;

    
        // build the glyph
        Ok(Glyph {
            ch: glyph.glyph,
            atlas: self.id,
            width: glyph.width,
            height: glyph.height,
            uv_x: old_x / self.size.width() as f32,
            uv_y: self.base_line / self.size.height() as f32,
            uv_dx: glyph.width / self.size.width() as f32,
            uv_dy: glyph.height / self.size.height() as f32,
        })
    }

    // checks whether there is room on the current row for the new glyph
    fn has_space(&self, glyph: &RasterizedGlyph) -> bool {
        self.x + glyph.width < self.size.width() as f32
    }
    
    // advance to the next row of the atlas
    // Errors if there is no more room
    fn advance(&mut self) -> Result<()> {
        if self.base_line + self.max_height < self.size.height() as f32 {
            self.base_line += self.max_height;
            self.max_height = 0f32;
            Ok(())
        }
        else {
            Err(Error::AtlasError("last line of atlas".to_owned()))
        }
    }
}

/*
    let ortho = glm::ortho(0f32, w as f32, h as f32, 0.0, -1f32, 1f32);
    unsafe {
        let un_tex = gl::GetUniformLocation(program, CString::new("text").unwrap().as_ptr());
        gl::Uniform1i(un_tex, tex.id as i32);
        let un_per = gl::GetUniformLocation(program, CString::new("per").unwrap().as_ptr());
        gl::UniformMatrix4fv(un_per, 1, gl::FALSE, ortho.as_ptr());

        let un_clr = gl::GetUniformLocation(program, CString::new("background").unwrap().as_ptr());
        gl::Uniform4f(un_clr, 1.0, 1.0, 1.0, 1.0);

        gl::ClearColor(0.2, 0.2, 0.2, 1.0);
        gl::Viewport(0, 0, w, h);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
        // gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::MULTISAMPLE);
        gl::DepthMask(gl::FALSE);
    }
*/
