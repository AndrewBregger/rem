pub mod render;
pub mod caches;

#[macro_use] pub use render::*;
pub use caches::*;

pub(crate) mod shader;


use crate::font;
use crate::font::GlyphKey;

#[derive(Debug, Clone)]
pub enum Error {
    FontError(font::Error),
    RenderError(String),
    AtlasError(String),
    CacheMissChar(GlyphKey)
}

pub type Result<T> = ::std::result::Result<T, Error>;

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