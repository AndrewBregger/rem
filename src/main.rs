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

use std::sync::mpsc;
use std::thread::Builder;
use std::result::Result;
use std::mem;
use std::ptr;
use std::str;


use editor_core::Document;
use font::Rasterizer;
use render::{GlyphCache, Renderer};
use std::collections::HashMap;
use gl::types::*;
use std::path::PathBuf;
use glutin::*;

use editor::App;

static INDEX_DATA: [u32; 6] = [0, 1, 2, 0, 2, 3];
static RECT_INDEX_DATA: [u32; 4] = [0, 1, 2, 3];

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
        gl::DEBUG_SEVERITY_NOTIFICATION => return,
        _ => "UNKNOWN",
    };
    
    let message = unsafe { std::slice::from_raw_parts(message as *const u8, length as usize) };
    println!("{}: {} of {} severity, raised from {}: {}", id, ty, ser, sor, str::from_utf8(message).unwrap());
}

fn main() -> Result<(), editor::Error>{
    let config = config::Config::default();

    let app = App::new(config);

    unsafe {
        gl::DebugMessageCallback(callback, ptr::null());
    }


    // glCheck!();


    let document = Document::from_path("src/main.rs").unwrap();

    // let mut running = true;
    // while running { 
    //     let process = |event| {
    //         match event {
    //             // LoopDestroyed => running = false,
    //             Event::DeviceEvent { .. } => (),
    //             Event::WindowEvent { ref event, .. } => match event {
    //                 WindowEvent::KeyboardInput { ref input, .. } => {
    //                     if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
    //                         if input.state == ElementState::Pressed {
    //                             println!("GoodBye crual world!");
    //                             running = false;
    //                         }
    //                     }
    //                 }
    //                 // Maybe using KeyboardInput and processing that would
    //                 // give a better using experience instead of using ReceivedCharacter
    //                 WindowEvent::ReceivedCharacter(_) => (), 
    //                 WindowEvent::CloseRequested | WindowEvent::Destroyed => running = false,
    //                 _ => (),
    //             },
    //             _ => (),
    //         }
    //     };

    //     window.poll_events(process);

    //     unsafe {
    //         gl::Clear(gl::COLOR_BUFFER_BIT);
    //     }
    //     let content = document.as_str();
    //     draw_text(content.as_ref(), 0, 0, &renderer, &mut cache).unwrap();

    //     window.swap_buffers();
    // }
    Ok(())
}

fn draw_text<T>(msg: &str, x: i32, y: i32, render: &Renderer, cache: &mut GlyphCache<T>) -> render::Result<()>
    where T: Rasterizer {
    let mut batch = render::Batch::new();
    const TAB_SIZE: i32 = 2;

    let mut cell = (x, y);

    for c in msg.chars() {
        if c == ' ' {
            cell.0 += 1;
            continue;
        }

        if c == '\n' {
            cell.0 = 0;
            cell.1 += 1;
            continue;
        }

        if c == '\t' {
            cell.0 += TAB_SIZE;
            continue;
        }

        let glyph = 
            match cache.get(c as u32) {
                Ok(glyph) => glyph,
                Err(_) => {
                    panic!(format!("Missing Glyph for character {}", c));
                    /* This is a possible process to handle missing characters in the cache.
                    // The missing character is requested
                    match cache.handle_miss(c, loader) {
                        // if it exists then return the new glyph
                        Ok(glyph) => glyphu,
                        // if it doenst exists, the return the square or whatever to signify a missing character in the font.
                        Err(_) => cache.get_missing_char_glyph()?,
                    }
                    */
                }
            };

        let instance = render::InstanceData {
            x: cell.0 as f32,
            y: cell.1 as f32,
            
            // text metrics offsets for the character
            width: glyph.width,
            height: glyph.height,
            offset_x: glyph.bearing_x + 1.0,
            offset_y: glyph.bearing_y,

            // texture coordinates
            uv_x: glyph.uv_x,
            uv_y: glyph.uv_y,
            uv_dx: glyph.uv_dx,
            uv_dy: glyph.uv_dy,

            tr: 0.7,
            tg: 0.4,
            tb: 0.7,

            br: 0.0,
            bg: 0.0,
            bb: 0.0,

            texture_id: 0,
        };

        // maybe add a is_full function to Batch
        // that would seperate this operation.
        if batch.push(instance) {
            render.draw_batch(&batch)?;
            batch.clear();
        }

        cell.0 += 1;
    }

    render.draw_batch(&batch)?;

    Ok(())
}