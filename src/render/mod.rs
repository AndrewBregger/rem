pub mod caches;
pub mod framebuffer;
pub mod render;

#[macro_use]
pub use render::*;
pub use caches::*;

pub(crate) mod shader;

use crate::font;
use crate::font::GlyphKey;

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
