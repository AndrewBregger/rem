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

use std::sync::mpsc;
use std::thread::Builder;
use std::result::Result;
use std::mem;
use std::ptr;
use std::str;


use editor_core::Document;
use font::Rasterizer;
use render::{GlyphCache};
use std::collections::HashMap;
use gl::types::*;
use std::path::PathBuf;


struct Loader {
    pub atlas: render::Atlas,
}

impl Loader {
    fn new() -> Self {
        Self {
            atlas: render::Atlas::new(window::Size::from(1024f32, 1024f32)).unwrap(),
        }
    }


}

impl render::GlyphLoader for Loader {
    fn load_glyph(&mut self, glyph: &font::RasterizedGlyph) -> render::Result<render::Glyph> {
        Ok(self.atlas.insert(glyph)?)
    }
}

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

fn main() {
    let event_loop = glutin::EventsLoop::new();    
    let mut window = window::Window::new(event_loop, window::Size::new(1024f64, 726f64)).unwrap();
    
    // How shnt id,
// ould this be handled?
    //
    // Gl function cannot be loaded until I have a context
    window.init_gl().unwrap();

    unsafe {
        gl::DebugMessageCallback(callback, ptr::null());
    }

    let mut vbo = 0;
    let mut vbo2 = 0;
    let mut ibo = 0;
    let mut ibo2 = 0;
    let mut vao = 0;
    let mut vao2 = 0;

    let font = font::FontDesc {
        name: "DroidSansMono".to_string(),
        path: std::path::Path::new("dev/DroidSansMono.ttf").to_path_buf(),
    };

    let mut atlas = render::Atlas::new(window::Size::from(1024f32, 1024f32)).unwrap();

    let mut rasterizer = font::FreeTypeRasterizer::new(window.window_dpi() as f32).unwrap();
    let fontsize = font::FontSize { pixel_size: 20f32 };

    let mut cache = GlyphCache::new(rasterizer, config::Font { font, size: fontsize}, window.window_dpi() as f32, render::CacheMissProto::ErrorOnMiss).unwrap();
    // glCheck!();
    
    println!("Window DPI: {}", window.window_dpi());


    let mut loader = Loader::new();
    cache.load_glyphs(&mut loader);
    let atlas = loader.atlas;

    // glCheck!();

    // let mut shader = render::TextShader::new().unwrap();
    // let mut rect_shader = render::RectShader::new().unwrap();

    // glCheck!();
/*
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenVertexArrays(1, &mut vao2);
        // glCheck!();
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut vbo2);
        gl::GenBuffers(1, &mut ibo);
        gl::GenBuffers(1, &mut ibo2);
        // glCheck!();

        gl::BindVertexArray(vao);
        // glCheck!();

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (mem::size_of::<u32>() * INDEX_DATA.len()) as isize,
            INDEX_DATA.as_ptr() as *const _,
            gl::STATIC_DRAW);
        // glCheck!();

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        // glCheck!();
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<InstanceData>() * BATCH_SIZE) as isize,
            ptr::null(),
            gl::STREAM_DRAW);
        // glCheck!();



        let size = mem::size_of::<InstanceData>() as i32;
        let float_size = mem::size_of::<f32>();

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2 as i32, gl::FLOAT, gl::FALSE, size, ptr::null());
        gl::VertexAttribDivisor(0, 1);
        // glCheck!();
    
        let mut stride = 2;

        // color attribute
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 4 as i32, gl::FLOAT, gl::FALSE, size, (stride * float_size) as *const _);
        gl::VertexAttribDivisor(1, 1);
        // glCheck!();

        stride += 4;

        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 4 as i32, gl::FLOAT, gl::FALSE, size, (stride * float_size) as *const _);
        gl::VertexAttribDivisor(2, 1);
        // glCheck!();

        stride += 4;

        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(3, 3 as i32, gl::FLOAT, gl::FALSE, size, (stride * float_size) as *const _);
        gl::VertexAttribDivisor(3, 1);
        // glCheck!();


        gl::BindVertexArray(0);
        // glCheck!();

        let size = mem::size_of::<RectInstanceData>() as i32;

        gl::BindVertexArray(vao2);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo2);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (mem::size_of::<u32>() * INDEX_DATA.len()) as isize,
            RECT_INDEX_DATA.as_ptr() as *const _,
            gl::STATIC_DRAW);
        // glCheck!();

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo2);
        // glCheck!();
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<RectInstanceData>() * BATCH_SIZE) as isize,
            ptr::null(),
            gl::STREAM_DRAW);
        // glCheck!();

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2 as i32, gl::FLOAT, gl::FALSE, size, ptr::null());
        gl::VertexAttribDivisor(0, 1);
        // glCheck!();
    
        let mut stride = 2;

        // color attribute
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3 as i32, gl::FLOAT, gl::FALSE, size, (stride * float_size) as *const _);
        gl::VertexAttribDivisor(1, 1);
        glCheck!();

        gl::BindVertexArray(0);
        glCheck!();
    }

    let (w, h) = window.dimensions();
    println!("{}, {}", w, h);
    glCheck!();

    let ortho = glm::ortho(0f32, w as f32, h as f32, 0.0, -1f32, 1f32);
    //glCheck!();

    let metrics = cache.metrics();
    let cell_size = (metrics.average_advance, metrics.line_height);

    println!("{:?}", cell_size);
    shader.activate();

    shader.set_perspective(ortho);
    // glCheck!();

    shader.set_font_atlas(&atlas);
    // glCheck!();
    shader.set_cell_size(cell_size);
    // glCheck!();

    shader.deactivate();
    // glCheck!();

    rect_shader.activate();

    rect_shader.set_perspective(ortho);

    rect_shader.set_cell_size(cell_size);

    rect_shader.deactivate();

    // glCheck!();

    unsafe {
        gl::Viewport(0, 0, w as i32, h as i32);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
        gl::Enable(gl::MULTISAMPLE);
        // glCheck!();
        gl::DepthMask(gl::FALSE);
        // glCheck!();
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        // glCheck!();
    }
    
    let msg = "Hello, World"; 

    for c in msg.chars() {
        if c == ' ' {
            continue;
        }

        let glyph = cache.get(c as u32).unwrap(); 
        println!("{}: {:?}", c, glyph);
    }

    
    println!("W: {}, H: {} {:?}", w, h, cell_size);

    let num_rows = w as f32 / cell_size.1;
    let num_cols = h as f32 / cell_size.0;


    println!("{} {}", num_cols, num_rows);

    let num_rows = num_rows.floor() as u32;
    let num_cols = num_cols.floor() as u32;
    println!("{} {}", num_cols, num_rows);


    let mut rect_instance_data = Vec::with_capacity((num_rows * num_cols) as usize);

    for i in 0..num_cols {
        for j in 0..num_rows {
            let instance = RectInstanceData {
                x: i as f32,
                y: j as f32,
                r: 0.8,
                g: 0.2,
                b: 0.5,
            };

            rect_instance_data.push(instance);
        }
    }
    
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo2);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<RectInstanceData>() * rect_instance_data.len()) as isize,
            rect_instance_data.as_ptr() as *const _,
            gl::STREAM_DRAW);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    loop {
        let mut running = true;

        let process = |event| {
            match event {
                // LoopDestroyed => running = false,
                Event::DeviceEvent { .. } => (),
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            if input.state == ElementState::Pressed {
                                println!("GoodBye crual world!");
                                running = false;
                            }
                        }
                    }
                    // Maybe using KeyboardInput and processing that would
                    // give a better using experience instead of using ReceivedCharacter
                    WindowEvent::ReceivedCharacter(_) => (), 
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => running = false,
                    _ => (),
                },
                _ => (),
            }
        };

        window.poll_events(process);

        if running == false {
            break
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        
        atlas.bind();

        unsafe {
            
//            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
//            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    
            // draw_string("Hello, world", &glyphmap); //&font, &atlas);

            let mut cell = (0, 0);
            let mut instance_data = Vec::new();
            for c in msg.chars() {
                if c == ' ' {
                    cell.0 += 1;
                    continue;
                }
                
                let glyph = cache.get(c as u32).unwrap();
                
                let instance = InstanceData {
                    x: cell.0 as f32,
                    y: cell.1 as f32,
                    
                    // text metrics offsets for the character
                    width: glyph.width,
                    height: glyph.height,
                    offset_x: glyph.bearing_x,
                    offset_y: glyph.bearing_y,
                    // texture coordinates
                    uv_x: glyph.uv_x,
                    uv_y: glyph.uv_y,
                    uv_dx: glyph.uv_dx,
                    uv_dy: glyph.uv_dy,
                    // Mayby this could be used if I move to a texture array of atlases?.
                    // texture_id: f32,

                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                };

                instance_data.push(instance);
                cell.0 += 1;
            }
        
            for i in 0..num_cols {
                let c = (i % 10) + '0' as u32;
                let glyph = cache.get(c).unwrap();
                
                let instance = InstanceData {
                    x: i as f32,
                    y: 2.0,
                    
                    // text metrics offsets for the character
                    width: glyph.width,
                    height: glyph.height,
                    offset_x: glyph.bearing_x,
                    offset_y: glyph.bearing_y,
                    // texture coordinates
                    uv_x: glyph.uv_x,
                    uv_y: glyph.uv_y,
                    uv_dx: glyph.uv_dx,
                    uv_dy: glyph.uv_dy,
                    // Mayby this could be used if I move to a texture array of atlases?.
                    // texture_id: f32,

                    r: 0.4,
                    g: 0.1,
                    b: 1.0,
                };

                instance_data.push(instance);

            }

    


            unsafe {
                shader.activate();
    
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (mem::size_of::<InstanceData>() * instance_data.len()) as isize,
                    instance_data.as_ptr() as *const _,
                    gl::STREAM_DRAW);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);

                gl::BindVertexArray(vao);
            
                gl::Enable(gl::BLEND);
                gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), instance_data.len() as i32);

                gl::BindVertexArray(0);
                shader.deactivate();


                rect_shader.activate();
                gl::BindVertexArray(vao2);

                gl::Disable(gl::BLEND);
                gl::DrawElementsInstanced(gl::LINE_LOOP, 4, gl::UNSIGNED_INT, ptr::null(), rect_instance_data.len() as i32);
                gl::BindVertexArray(0);
                rect_shader.deactivate();

                // glCheck!();
            }


            gl::BindTexture(gl::TEXTURE_2D, 0);

//            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
//            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        window.swap_buffers();
    }
    */
}

/*
fn draw_string(msg: &str,  glyphmap: &HashMap<u32, render::Glyph>/*font: &font::FontDesc, atlas: &render::Atlas*/) {
    
    // converting this to a cell based system.
    let mut cell = (0, 0);
    let mut instance_data = Vec::new();
    for c in msg.chars() {
        if c == ' ' {
            cell.0 += 1;
            continue;
        }
        
        let glyph = glyphmap.get(&(c as u32)).unwrap();
        
        let instance = InstanceData {
            x: cell.0 as f32,
            y: cell.1 as f32,
            
            // text metrics offsets for the character
            width: glyph.width,
            height: glyph.height,
            left: glyph.left,
            top: glyph.top,
            // texture coordinates
            uv_x: glyph.uv_x,
            uv_y: glyph.uv_y,
            uv_dx: glyph.uv_dx,
            uv_dy: glyph.uv_dy,
            // Mayby this could be used if I move to a texture array of atlases?.
            // texture_id: f32,

            r: 1.0,
            g: 0.5,
            b: 0.2,
        };

        instance_data.push(instance);
        cell.0 += 1;
    }

    println!("{:?}", instance_data);

    unsafe {
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<InstanceData>() * instance_data.len()) as isize,
            instance_data.as_ptr() as *const _,
            gl::STREAM_DRAW);
    
        gl::DrawElementsInstanced(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null(), instance_data.len() as i32);
        glCheck!();
    }
}
*/
