use gl;
use gl::types::*;

use std::mem;
use std::ptr;

use super::glm;
use std::ffi::CString;
use super::shader;
use super::font;
use super::window;
use super::window::Size;
use super::font::{RasterizedGlyph, Rasterizer};

static INDEX_DATA: [u32; 6] = [0, 1, 2, 0, 2, 3];

static VS_SOURCE: &'static str = "shaders/vs.glsl";
static FS_SOURCE: &'static str = "shaders/fs.glsl";

#[derive(Debug, Clone)]
pub enum Error {
    FontError(String),
    RenderError(String),
    AtlasError(String),
}
type Result<T> = ::std::result::Result<T, Error>;

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


pub struct Renderer {
    // vertex buffer object
    vbo: u32,
    // index buffer object
    ibo: u32,
    // vertex attribute object
    vao: u32,
    // the shader for rendering text
    shader: TextShader,
    // all of the vertex atlases
    atlases: Vec<Atlas>,
}

pub struct TextShader {
    program: u32,
    // uniform location
    per_loc: i32,
    // uniform atlas
    atlas_loc: i32,
    // size of each cell
    cell_loc: i32,
    // shader used
    active: bool,
}

impl TextShader {
    pub fn new() -> Result<Self> {
        let vs_src = shader::load_file(VS_SOURCE);
        let fs_src = shader::load_file(FS_SOURCE);

        let vs = shader::compile_shader(vs_src.as_str(), gl::VERTEX_SHADER);
        let fs = shader::compile_shader(fs_src.as_str(), gl::FRAGMENT_SHADER);

        let program = shader::link_shader(vs, fs);

        
        let per_loc = unsafe { gl::GetUniformLocation(program, CString::new("projection").unwrap().as_ptr()) };
        let atlas_loc = unsafe { gl::GetUniformLocation(program, CString::new("atlas").unwrap().as_ptr()) };
        let cell_loc = unsafe { gl::GetUniformLocation(program, CString::new("cell_size").unwrap().as_ptr()) };

        Ok(Self {
            program,
            per_loc,
            atlas_loc,
            cell_loc,
            active: false,
        })
    }

    pub fn set_perspective(&self, per: glm::Mat4) {
        unsafe { gl::UniformMatrix4fv(self.per_loc, 1, gl::FALSE, per.as_ptr()) };
    }

    pub fn set_font_atlas(&self, atlas: &Atlas) {
        unsafe { gl::Uniform1i(self.atlas_loc, atlas.texture_id as i32) };
    }

    pub fn set_cell_size(&self, size: (f32, f32)) {
        unsafe { gl::Uniform2f(self.cell_loc, size.0, size.1) };
    }

    pub fn activate(&mut self) {
        unsafe { gl::UseProgram(self.program) };
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        unsafe { gl::UseProgram(0) };
        self.active = false;
    }
}



impl Renderer {
    pub fn new(dpi: f64) -> Result<Self> {
    
        let mut vbo = 0;
        let mut ibo = 0;
        let mut vao = 0;

        let mut atlas = Atlas::new(Size::from(1024f32, 1024f32))
            .map_err(|e| Error::FontError(format!("{:?}", e)))?;
    
        let font = font::FontDesc {
            style: font::Style::Normal,
            path: std::path::Path::new("dev/DroidSansMono.ttf").to_path_buf(),
            size: font::Size::new(24u16),
            id: 0
        };
        
        let glyph = font::GlyphDesc {
            ch: 'a' as u32,
            font: font.clone(),
        };

        let mut rasterizer = font::FTRasterizer::new(dpi)
            .map_err(|e| Error::FontError(format!("{:?}", e)))?;

        let glyph = rasterizer.load_glyph(glyph)
            .map_err(|e| Error::FontError(format!("{:?}", e)))?;

        let glyph = atlas.insert(glyph).map_err(|e| Error::FontError(format!("{:?}", e)))?;

        struct Vertex {
            pos: [f32; 2],
            uv: [f32; 2]
        }

        fn build_geometry(glyph: &Glyph) -> Vec<Vertex> {
            let mut buf = Vec::new();

            let x = 10f32;
            let y = 10f32;

            // 0.5f,  0.5f, 0.0f,   1.0f, 0.0f, 0.0f,   1.0f, 1.0f,   // top right
            // 0.5f, -0.5f, 0.0f,   0.0f, 1.0f, 0.0f,   1.0f, 0.0f,   // bottom right
            //-0.5f, -0.5f, 0.0f,   0.0f, 0.0f, 1.0f,   0.0f, 0.0f,   // bottom left
            //-0.5f,  0.5f, 0.0f,   1.0f, 1.0f, 0.0f,   0.0f, 1.0f    // top left 
            
            buf.push(Vertex {
                pos: [x + glyph.width, y],
                uv: [glyph.uv_x + glyph.uv_dx, glyph.uv_y],
            }); // top right

            buf.push(Vertex {
                pos: [x + glyph.width, y + glyph.height],
                uv: [glyph.uv_x + glyph.uv_dx, glyph.uv_y + glyph.uv_dy],
            }); // bottom right

            buf.push(Vertex {
                pos: [x, y + glyph.height],
                uv: [glyph.uv_x, glyph.uv_y + glyph.uv_dy],
            }); // bottom left

            buf.push(Vertex {
                pos: [x, y],
                uv: [glyph.uv_x, glyph.uv_y],
            }); // top left

            buf
        }

        let vertices = build_geometry(&glyph);

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ibo);

            gl::BindBuffer(gl::VERTEX_ARRAY, vbo);
            gl::BufferData(
                gl::VERTEX_ARRAY,
                (mem::size_of::<Vertex>() * vertices.len()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mem::size_of::<u32>() * INDEX_DATA.len()) as isize,
                INDEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW);

            let stride = 2 * mem::size_of::<f32>();

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2 as i32, gl::FLOAT, gl::FALSE, mem::size_of::<Vertex>() as i32, ptr::null());

            // color attribute
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2 as i32, gl::FLOAT, gl::FALSE, mem::size_of::<Vertex>() as i32, stride as *const _);

            gl::BindVertexArray(0);
        }



        Ok(Self {
            vbo,
            ibo,
            vao,
            shader: TextShader::new()?,
            atlases: vec![atlas]
        })
    }

    pub fn draw(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }
    }

    pub fn setup(&self, window: &window::Window) {
        let (w, h) = window.dimensions();
        let ortho = glm::ortho(0f32, w as f32, h as f32, 0.0, -1f32, 1f32);


        self.shader.set_perspective(ortho);
        self.shader.set_font_atlas(self.atlases.first().unwrap());

        unsafe {
            gl::Viewport(0, 0, w as i32, h as i32);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
            // gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::MULTISAMPLE);
            gl::DepthMask(gl::FALSE);

            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        }
    }
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
/////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct Atlas {
    // the current x within the atlas
    x: f32,
    // the current baseline of the atlas
    base_line: f32,
    // the size of the underline texture
    size: Size,
    // the height of the larget subtexture in the current row.
    max_height: f32,
    // the gl texture handle
    pub(crate) texture_id: u32,
    // the altas id
    id: u32,
}

impl Atlas {
    // the gl texture is not allocated until a subtexture is added.
    pub fn new(size: Size) -> Result<Self> {
        let mut atlas = Self {
            x: 0.0,
            base_line: 0.0,
            size,
            max_height: 0.0,
            texture_id: 0,
            id: 0, // I dont know how to set this.
        };

        atlas.allocate_texture()?;

        Ok(atlas)
    }

    pub fn bind(&self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + self.texture_id);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
        }
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
        
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
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

            glCheck!();

            gl::BindTexture(gl::TEXTURE_2D, 0);

        }
        Ok(())
    }

    pub fn insert(&mut self, glyph: &RasterizedGlyph) -> Result<Glyph> {
        // move to next row if needed
        //println!("{:?}", self.size);
        //println!("\tW: {}, H: {}", glyph.width, glyph.height);
        //println!("\tX: {}, y: {}", self.x, self.base_line);
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
            glCheck!();

            //println!("{:?}", glyph);
            //println!("{:?}", self.size);

            gl::TexSubImage2D(gl::TEXTURE_2D,  
                              0,
                              self.x as i32,
                              self.base_line as i32,
                              glyph.width as i32,
                              glyph.height as i32,
                              gl::RGB,
                              gl::UNSIGNED_BYTE,
                              mem::transmute(&glyph.bitmap[0]));
            glCheck!();

            gl::BindTexture(gl::TEXTURE_2D, 0);
            glCheck!();
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
            top: glyph.top,
            left: glyph.left,
            advance_x: glyph.advance_x,
            advance_y: glyph.advance_y,
            bearing_x: glyph.bearing_x,
            bearing_y: glyph.bearing_y
        })
    }

    // checks whether there is room on the current row for the new glyph
    fn has_space(&self, glyph: &RasterizedGlyph) -> bool {
        self.x + glyph.width < self.size.width() as f32
    }
    
    // advance to the next row of the atlas
    // Errors if there is no more room
    fn advance(&mut self) -> Result<()> {
        println!("Advance!");
        if self.base_line + self.max_height < self.size.height() as f32 {
            self.x = 0.0;
            self.base_line += self.max_height;
            self.max_height = 0f32;
            Ok(())
        }
        else {
            Err(Error::AtlasError("last line of atlas".to_owned()))
        }
    }
}

