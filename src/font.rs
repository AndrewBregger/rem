use super::ft;
use super::PathBuf;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Size(u16);

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Style {
    Normal,
    Italics,
    Bold
}

type FResult<T> = Result<T, ft::Error>;

// describes a font, eventually this will use font patterns to identify fonts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDesc {
    pub style: Style,
    pub path: PathBuf,
    pub size: Size,
    pub id: u32, // to uniquly identify this font
}

// global metrics of the fonts.
#[derive(Debug, Clone)]
pub struct FontMetrics {
}

// desciption of a glyph and font.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphDesc {
    pub ch: u32,        // to support unicode
    pub font: FontDesc, // the uniquly identify this font.
}

#[derive(Debug, Clone)]
struct Face {
    pub font: FontDesc,
    pub face: ft::Face,
    pub glyphs: HashMap<GlyphDesc, RasterizedGlyph>,
    pub metrics: FontMetrics,
}

// rasterized glyph for a specified size
#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    pub glyph: u32,
    pub glyph_index: u32,
    pub size: Size,
    pub bitmap: Vec<u8>,
    pub advance_x: f32,
    pub advance_y: f32,
    pub width: f32,
    pub height: f32,
    pub top: f32,
    pub left: f32
}

trait Rasterizer {
    fn load_glyph(&mut self, glyph: GlyphDesc) -> FResult<&RasterizedGlyph>;
     //fn load_font(&self, font: FontDesc) -> FResult<>;
}

struct FTRasterizer {
    library: ft::Library,
    faces: HashMap<FontDesc, Face>,
    fonts: HashMap<PathBuf, FontDesc>,
}

impl Hash for FontDesc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.path.hash(state);
    }
}

impl Hash for GlyphDesc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ch.hash(state);
        self.font.hash(state);
    }
}

impl Face {
    fn new(face: ft::Face, font: FontDesc) -> Self {
        Self {
           font,
           face,
           glyphs: HashMap::new(),
           metrics: FontMetrics {},
        }
    }
}

impl FTRasterizer {
   pub fn new() -> FResult<FTRasterizer> {
       Ok(Self {
           library: ft::library::Library::init()?,
           faces: HashMap::new(),
           fonts: HashMap::new(),
       })
   }
    
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

   fn rasterize_glyph(face: &Face, glyph_index: u32) -> FResult<&ft::GlyphSlot> {
       if glyph_index == 0 {
           Err(ft::Error::InvalidHandle)
       }
       else {
           face.face.load_glyph(glyph_index, ft::face::LoadFlag::RENDER); // get this info somewhere else
           Ok(face.face.glyph())
       }
   }

   fn maybe_insert_font_desc(&mut self, font: &FontDesc) {
       if !self.fonts.contains_key(&font.path) {
           self.fonts.insert(font.path.clone(), font.clone());
       }
   }
}

impl Rasterizer for FTRasterizer {
    //pub fn load_font(&self, font: FontDesc) -> FResult<RasterizedClyph> {
    //}

    fn load_glyph(&mut self, glyph: GlyphDesc) -> FResult<&RasterizedGlyph> {
        unsafe {
            ft::ffi::FT_Library_SetLcdFilter(self.library.raw(), ft::LcdFilter::LcdFilterDefault as u32);
        }

        let mut face = if self.faces.contains_key(&glyph.font) {
            self.faces.get_mut(&glyph.font).unwrap()
        }
        else {
           let pathbuf = glyph.font.path.clone();
           let ft_face = self.library.new_face(pathbuf, glyph.font.style.clone() as isize)?;
           let face = Face::new(ft_face, glyph.font.clone());
           
           self.maybe_insert_font_desc(&glyph.font);
           self.faces.insert(glyph.font.clone(), face);
        
           // we know it is there
           self.faces.get_mut(&glyph.font).unwrap()

        };
    
        let index = face.face.get_char_index(glyph.ch as usize);
        let gl = Self::rasterize_glyph(face, index)?;

        let (w, h, buf) = Self::normalize_buffer(&gl.bitmap());

        let ft::Vector { x , y } = gl.advance();

        let rglyph = RasterizedGlyph {
            glyph: glyph.ch,
            glyph_index: index,
            size: glyph.font.size.clone(),
            bitmap: buf,
            advance_x: x as f32,
            advance_y: y as f32,
            width: w,
            height: h,
            top: gl.bitmap_top() as f32,
            left: gl.bitmap_left() as f32
        };

        face.glyphs.insert(glyph.clone(), rglyph);

        Ok(face.glyphs.get(&glyph).unwrap())
    }
}
