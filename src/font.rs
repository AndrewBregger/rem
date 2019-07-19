use super::ft;
use super::PathBuf;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::cmp::Eq;
use std::sync::atomic::{AtomicU16, Ordering::SeqCst};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontSize {
    pub pixel_size: f32,
}

impl Eq for FontSize {
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Style {
    Normal,
    Italics,
    Bold,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    FTError(ft::Error),
    MissingFont,
    InvalidGlyph,
    NoSizeMetrics,
}

// describes a font, eventually this will use font patterns to identify fonts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDesc {
    // pub style: Style,
    pub name: String,
    pub path: PathBuf,
//     pub size: Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontKey {
    pub token: u16,
}

// global metrics of the fonts.
#[derive(Debug, Clone)]
pub struct FullMetrics {
    ft_metrics: ft::ffi::FT_Size_Metrics,
    cell_width: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub average_advance: f32,
    pub line_height: f32,
    pub descent: f32
}

// desciption of a glyph and font.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphKey {
    pub ch: u32,        // to support unicode
    pub font: FontKey, // the uniquly identify this font.
    pub size: FontSize,
}

#[inline]
fn convert_to_ft(size: &FontSize) -> isize {
    (((1 << 6) as f32) * size.pixel_size) as isize
}

#[derive(Clone)]
struct Face {
    pub(crate) font: FontKey,
    pub(crate) face: ft::Face,
    render_mode: ft::RenderMode,
    lcd_mode: i32,
}



// rasterized glyph for a specified size
#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    pub glyph: u32,

    pub width: f32,
    pub height: f32,

    pub top: f32,
    pub left: f32,

    pub bearing_x: f32,
    pub bearing_y: f32,

    pub bitmap: Vec<u8>,
}

pub trait Rasterizer : std::marker::Sized {
    fn new(dpi_factor: f32) -> Result<Self>;

    fn load_glyph(&mut self, glyph: GlyphKey, size: FontSize) -> Result<RasterizedGlyph>;

    fn get_metrics(&self, font: FontKey, size: FontSize) -> Result<Metrics>;

//     fn get_full_metrics(&self, font: FontKey, size: FontSize) -> Result<FullMetrics>;

    fn get_font(&mut self, font: FontDesc) -> Result<FontKey>;
}

pub struct FreeTypeRasterizer {
    library: ft::Library,
    faces: HashMap<FontKey, Face>,
    fonts: HashMap<PathBuf, FontKey>,
    dpi_factor: f64,
}

impl FontKey {
    fn next() -> Self {
        static TOKEN: AtomicU16 = AtomicU16::new(0);

        FontKey {
            token: TOKEN.fetch_add(1, SeqCst),
        }
    }
}

//impl Hash for FontKey {
//    fn hash<H: Hasher>(&self, state: &mut H) {
//        self.token.hash(state);
//    }
//}

impl Hash for GlyphKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ch.hash(state);
        self.font.hash(state);
    }
}

impl Face {
    fn new(library: &ft::Library, path: &PathBuf, font: FontKey) -> Result<Self> {
        Ok(Self {
            font,
            face: library.new_face(path, 0).map_err(|e| Error::FTError(e))?,
            render_mode: ft::RenderMode::Normal,
            lcd_mode: ft::LcdFilter::LcdFilterDefault as i32,
        })
    }

    fn set_size(&self, size: FontSize) {
        self.face.set_char_size(convert_to_ft(&size), 0, 0, 0);
    }

    fn char_index(&self, glyph: &GlyphKey) -> Result<u32> {
        let index = self.face.get_char_index(glyph.ch as usize);
        if index == 0 {
            Err(Error::InvalidGlyph)
        }
        else {
            Ok(index)
        }
    }

    fn render_glyph(&self, glyph: GlyphKey, size: FontSize) -> Result<ft::GlyphSlot> {
        self.set_size(size);
        
        // gets the index of the character
        let index = self.char_index(&glyph)?;
        // loads the glyph into the slot
        self.face.load_glyph(index, ft::face::LoadFlag::DEFAULT);
        
        // gets the glyph
        let glyph = self.face.glyph();

        // renders the glyph into a bitmap
        glyph.render_glyph(self.render_mode);
        
        // returns a copy
        Ok(glyph.clone())
    }
}

impl FreeTypeRasterizer {

    fn normalize_buffer(bitmap: &ft::Bitmap) -> (f32, f32, Vec<u8>) {
        let mut data = Vec::new();
        let buf = bitmap.buffer();
        match bitmap.pixel_mode().unwrap() {
            ft::bitmap::PixelMode::Mono => {
                data.reserve(buf.len() * 8 * 3);
                fn expand_pixel(buf: &mut Vec<u8>, byte: u8, mut bit_count: u8) {
                    let mut offset = 7;
                    while bit_count != 0 {
                        let value: u8 = ((byte >> offset) & 1) * 255;

                        buf.push(value);
                        buf.push(value);
                        buf.push(value);

                        offset -= 1;
                        bit_count -= 1;
                    }
                }

                for i in 0..(bitmap.rows() as usize) {
                    let start = i * bitmap.pitch().abs() as usize;
                    let mut column = bitmap.width();
                    let mut byte = 0;

                    while column != 0 {
                        let bit_count = std::cmp::min(8, column);
                        expand_pixel(&mut data, buf[start + byte], bit_count as u8);

                        column -= bit_count;
                        byte += 1;
                    }
                }
                (bitmap.width() as f32, bitmap.rows() as f32, data)
            }
            ft::bitmap::PixelMode::Gray => {
                data.reserve(buf.len() * 3);
                for i in 0..bitmap.rows() {
                    let start = (i * bitmap.pitch()) as usize;
                    let stop = start + bitmap.width() as usize;
                    for byte in &buf[start..stop] {
                        data.push(*byte);
                        data.push(*byte);
                        data.push(*byte);
                    }
                }
                (bitmap.width() as f32, bitmap.rows() as f32, data)
            }
            ft::bitmap::PixelMode::Lcd => {
                data.reserve(buf.len());
                for i in 0..bitmap.rows() {
                    let start = (i as usize) * bitmap.pitch() as usize;
                    let stop = start + bitmap.width() as usize;
                    data.extend_from_slice(&buf[start..stop]);
                }
                (bitmap.rows() as f32, (bitmap.width() as f32 / 3.0), data)
            }
            ft::bitmap::PixelMode::LcdV => {
                panic!("Not implemented");
            }
            _ => {
                panic!("Not implemented");
            }
        }
    }

    fn find_font(&self, font: FontKey) -> Result<&Face> {
        self.faces.get(&font).ok_or(Error::MissingFont)
    }

    fn get_rendered_glyph(&mut self, glyph: GlyphKey, size: FontSize) -> Result<RasterizedGlyph> {
        let font = glyph.font; 
        
        let face = self.find_font(font)?;
    
        let bitmap = face.render_glyph(glyph.clone(), size)?;

        let (w, h, buffer) = Self::normalize_buffer(&bitmap.bitmap());

        Ok( RasterizedGlyph {
            glyph: glyph.ch,
            width: w,
            height: h,
            top: bitmap.bitmap_top() as f32,
            left: bitmap.bitmap_left() as f32,
            bearing_x: 0f32,
            bearing_y: 0f32,
            bitmap: buffer,
        })
    }

    fn get_full_metrics(&self, font: FontKey, size: FontSize) -> Result<FullMetrics> {
        let face = self.find_font(font)?;

        face.set_size(size);

        let metrics = face.face.size_metrics().ok_or(Error::NoSizeMetrics)?;

        let cell_width =
            match face.face.load_glyph('0' as u32, ft::face::LoadFlag::RENDER) {
                Ok(_) => (face.face.glyph().metrics().horiAdvance / 64) as f32,
                Err(_) => (metrics.max_advance / 64) as f32,
            };

        Ok( FullMetrics {
            ft_metrics: metrics,
            cell_width
        })
    }
}

impl Rasterizer for FreeTypeRasterizer {
    fn new(dpi_factor: f32) -> Result<Self> {
        let library = ft::Library::init().map_err(|e| Error::FTError(e))?;

        Ok(Self {
            library,
            faces: HashMap::new(),
            fonts: HashMap::new(),
            dpi_factor: dpi_factor as f64
        })
    }

    fn load_glyph(&mut self, glyph: GlyphKey, size: FontSize) -> Result<RasterizedGlyph> {
        self.get_rendered_glyph(glyph, size)
    }

    fn get_metrics(&self, font: FontKey, size: FontSize) -> Result<Metrics> {
        let _face = self.find_font(font)?;

        let full = self.get_full_metrics(font, size)?;

        let height = (full.ft_metrics.height / 64) as f32;
        let descent = (full.ft_metrics.descender / 64) as f32;

        Ok( Metrics {
            average_advance: full.cell_width,
            line_height: height,
            descent
        })
    }


    fn get_font(&mut self, font: FontDesc) -> Result<FontKey> {
        match self.fonts.get(&font.path) {
            Some(key) => Ok(key.clone()),
            None => {
                // build the face if it hasnt been seen yet.
                self.fonts.insert(font.path.clone(), FontKey::next());
                let key = self.fonts[&font.path];
                self.faces.insert(key, Face::new(&self.library, &font.path, key)?);

                return Ok(key);
            }
        }
    }
}
