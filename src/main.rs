extern crate freetype as ft;
extern crate gl;
extern crate ropey;
extern crate glutin;
extern crate image;
extern crate nalgebra_glm as glm;

mod font;
#[macro_use] mod render;

mod pane;
mod config;
mod editor_core;
mod window;
mod editor;
mod size;
use std::result::Result;
use std::ptr;
use std::str;
use font::Rasterizer;
use gl::types::*;
use std::path::PathBuf;
use editor::App;

static INDEX_DATA: [u32; 6] = [0, 1, 2, 0, 2, 3];
static RECT_INDEX_DATA: [u32; 4] = [0, 1, 2, 3];

macro_rules! check {
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
                panic!();
            }
        }
    }};
}

extern "system" fn callback(source: GLenum, gltype: GLenum, id: GLuint, severity: GLenum,
                            length: GLsizei, message: *const GLchar, userParam: *mut std::ffi::c_void) {
    let sor = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "WINDOW SYSTEM",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "SHADER COMPILER",
        gl::DEBUG_SOURCE_THIRD_PARTY => "THIRD PARTY",
        gl::DEBUG_SOURCE_APPLICATION => "APPLICATION",
        gl::DEBUG_SOURCE_OTHER => "UNKNOWN",
        _ => "UNKNOWN",
    };

    let ty = match gltype {
        gl::DEBUG_TYPE_ERROR => "ERROR",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED BEHAVIOR",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "UDEFINED BEHAVIOR",
        gl::DEBUG_TYPE_PORTABILITY => "PORTABILITY",
        gl::DEBUG_TYPE_PERFORMANCE => "PERFORMANCE",
        gl::DEBUG_TYPE_OTHER => "OTHER",
        gl::DEBUG_TYPE_MARKER => "MARKER",
        _ => "UNKNOWN",
    };

    let ser = match severity {
        gl::DEBUG_SEVERITY_HIGH => "HIGH",
        gl::DEBUG_SEVERITY_MEDIUM => "MEDIUM",
        gl::DEBUG_SEVERITY_LOW => "LOW",
        gl::DEBUG_SEVERITY_NOTIFICATION => "NOTICE",
        _ => "UNKNOWN",
    };
    
    let message = unsafe { std::slice::from_raw_parts(message as *const u8, length as usize) };
    println!("{}: {} of {} severity, raised from {}: {}", id, ty, ser, sor, str::from_utf8(message).unwrap());
}

fn main() -> Result<(), editor::Error>{
    let config = config::Config::default();

    let mut app = App::new(config)?;
    check!();

    unsafe {
        gl::DebugMessageCallback(callback, ptr::null());
    }

    let mut running = true;
    let mut _iter = 0;

    while app.process_input() {
        app.render_panes()?;

        app.render_window();

        app.swap_buffers();
        //iter += 1;
    }

    // app.clean();
    Ok(())
}

//  Ok(glyph) => glyph,
//  Err(_) => {
//      panic!(format!("Missing Glyph for character {}", c));
//      /* This is a possible process to handle missing characters in the cache.
//      // The missing character is requested
//      match cache.handle_miss(c, loader) {
//          // if it exists then return the new glyph
//          Ok(glyph) => glyphu,
//          // if it doenst exists, the return the square or whatever to signify a missing character in the font.
//          Err(_) => cache.get_missing_char_glyph()?,
//      }
//      */
//  }
//let instance = render::InstanceData {
//    x: cell.0 as f32,
//    y: cell.1 as f32,
//    
//    // text metrics offsets for the character
//    width: glyph.width,
//    height: glyph.height,
//    offset_x: glyph.bearing_x + 1.0,
//    offset_y: glyph.bearing_y,

//    // texture coordinates
//    uv_x: glyph.uv_x,
//    uv_y: glyph.uv_y,
//    uv_dx: glyph.uv_dx,
//    uv_dy: glyph.uv_dy,

//    tr: 0.7,
//    tg: 0.4,
//    tb: 0.7,

//    br: 0.0,
//    bg: 0.0,
//    bb: 0.0,

//    texture_id: 0,
//};
