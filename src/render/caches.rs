use gl::types::*;

use std::collections::HashMap;
use std::mem;
use std::ptr;

use super::font::{self, FontDesc, FontKey, FontSize, GlyphKey, RasterizedGlyph, Rasterizer};
use super::Error;
use super::Glyph;
use super::Result;

use crate::config;
use crate::size;

pub trait GlyphLoader {
    /// load a glyph
    fn load_glyph(&mut self, glyph: &RasterizedGlyph) -> Result<super::Glyph>;

    /// clear
    fn clear(&mut self);
}

pub type Size = size::Size<f32>;

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
    pub(crate) size: Size,
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

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
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

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            // allocate an empty texture
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                self.size.x as i32,
                self.size.y as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );

            // glCheck!();

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
            return Err(Error::AtlasFull);
        }

        // the glyph can be added.

        // if the new glyph is vertically largest glyph in the row then set
        // it to the new heightest.
        if glyph.height > self.max_height {
            self.max_height = glyph.height;
        }

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
            // glCheck!();

            //println!("{:?}", glyph);
            //println!("{:?}", self.size);

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                self.x as i32,
                self.base_line as i32,
                glyph.width as i32,
                glyph.height as i32,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                mem::transmute(&glyph.bitmap[0]),
            );
            // glCheck!();

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let old_x = self.x;

        // move the x cursor forward.
        self.x += glyph.width;

        // build the glyph
        Ok(Glyph {
            ch: glyph.glyph,
            atlas: self.texture_id,
            width: glyph.width,
            height: glyph.height,
            uv_x: old_x / self.size.width() as f32,
            uv_y: self.base_line / self.size.y as f32,
            uv_dx: glyph.width / self.size.x as f32,
            uv_dy: glyph.height / self.size.y as f32,
            top: glyph.top,
            left: glyph.left,
            advance_x: glyph.advance_x,
            advance_y: glyph.advance_y,
            bearing_x: glyph.bearing_x,
            bearing_y: glyph.bearing_y,
        })
    }

    // checks whether there is room on the current row for the new glyph
    fn has_space(&self, glyph: &RasterizedGlyph) -> bool {
        self.x + glyph.width < self.size.x as f32
    }

    // advance to the next row of the atlas
    // Errors if there is no more room
    fn advance(&mut self) -> Result<()> {
        println!("Advance!");
        if self.base_line + self.max_height < self.size.y as f32 {
            self.x = 0.0;
            self.base_line += self.max_height;
            self.max_height = 0f32;
            Ok(())
        } else {
            Err(Error::AtlasError("last line of atlas".to_owned()))
        }
    }

    fn clear(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture_id);
        }
    }
}

// should this be a Cache for all glyphs or just the a cache
// for a given font face?
// if all fonts:
//      then there needs to be a method of looking up the glyph
//      of a specific character, font, and size.
//      It is reasonable to assume that multiple fonts will be loaded
//      to render different text on the screen (tabs, file info, gutters, and messages).
#[derive(Debug, Clone)]
pub struct GlyphCache<T> {
    glyphs: HashMap<GlyphKey, Glyph>,
    rasterizer: T,
    font: FontKey,
    font_size: FontSize,
    metrics: font::Metrics,
    proto: CacheMissProto,
}

// What happens when a character and size is requested that doesn't exist.
#[derive(Debug, Clone)]
pub enum CacheMissProto {
    ErrorOnMiss,
    //    RasterizeChar,
    //    Custom(Fn(GlyphKey) -> Glyph)
}
//
//impl CacheMissProto {
//    fn custom<P>(f: P) -> Self
//        where P: Fn(GlyphKey) -> Glyph {
//        CacheMissProto::Custom(f)
//    }
//}

impl<T> GlyphCache<T>
where
    T: Rasterizer,
{
    pub fn new(
        mut rasterizer: T,
        font: config::Font,
        dpi: f32,
        proto: CacheMissProto,
    ) -> Result<Self> {
        let font_size = font.size;
        let font = rasterizer
            .get_font(font.font)
            .map_err(|e| Error::FontError(e))?;
        let metrics = rasterizer
            .get_metrics(font, font_size)
            .map_err(|e| Error::FontError(e))?;

        Ok(Self {
            glyphs: HashMap::new(),
            rasterizer,
            font,
            font_size,
            metrics,
            proto,
        })
    }

    pub fn metrics(&self) -> &font::Metrics {
        &self.metrics
    }

    pub fn get(&self, ch: u32) -> Result<&Glyph> {
        let glyph = GlyphKey {
            ch,
            font: self.font,
            size: self.font_size,
        };

        self.request(&glyph)
    }

    pub fn request(&self, glyph: &GlyphKey) -> Result<&Glyph> {
        match self.glyphs.get(glyph) {
            Some(g) => Ok(g),
            None => Err(Error::CacheMissChar(glyph.clone())),
        }
    }

    pub fn load_glyph<F>(&mut self, glyph: GlyphKey, loader: &mut F)
    where
        F: GlyphLoader,
    {
        let rasterizer = &mut self.rasterizer;

        self.glyphs.entry(glyph.clone()).or_insert_with(|| {
            let size = glyph.size.clone();
            let rglyph = rasterizer.load_glyph(glyph, size).unwrap();

            loader.load_glyph(&rglyph).unwrap()
        });
    }

    pub fn load_glyphs<F>(&mut self, loader: &mut F)
    where
        F: GlyphLoader,
    {
        for c in 33..=126 {
            let g = font::GlyphKey {
                ch: c as u32,
                font: self.font,
                size: self.font_size,
            };
            self.load_glyph(g, loader);
        }

        for c in 161..=256 {
            let g = font::GlyphKey {
                ch: c as u32,
                font: self.font,
                size: self.font_size,
            };

            self.load_glyph(g, loader);
        }
    }

    pub fn font(&self) -> FontKey {
        self.font
    }

    pub fn font_size(&self) -> FontSize {
        self.font_size
    }
}

pub struct LoadApi<'a> {
    atlas: &'a mut Vec<Atlas>,
    last: usize,
}

impl<'a> LoadApi<'a> {
    pub fn new(atlas: &'a mut Vec<Atlas>) -> Self {
        Self { atlas, last: 0 }
    }
}

fn load_glyph(
    atlas: &mut Vec<Atlas>,
    last: &mut usize,
    glyph: &font::RasterizedGlyph,
) -> Result<super::Glyph> {
    match atlas[*last].insert(glyph) {
        Ok(glyph) => Ok(glyph),
        Err(Error::AtlasFull) => {
            let size = atlas[*last].size;
            *last += 1;
            if *last == atlas.len() {
                let t = Atlas::new(size)?;
                atlas.push(t);
                load_glyph(atlas, last, glyph)
            } else {
                panic!("LoadApi failed to manage last atlas");
            }
        }
        Err(e) => Err(e),
    }
}

impl<'a> GlyphLoader for LoadApi<'a> {
    fn load_glyph(&mut self, glyph: &font::RasterizedGlyph) -> Result<super::Glyph> {
        load_glyph(self.atlas, &mut self.last, glyph)
    }

    fn clear(&mut self) {
        for atlas in self.atlas.iter_mut() {
            atlas.clear();
        }

        self.atlas.clear();
        self.last = 0;
    }
}
