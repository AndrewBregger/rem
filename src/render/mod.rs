pub mod render;

#[macro_use] pub use render::*;

pub(crate) mod shader;
pub(crate) mod window;

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
