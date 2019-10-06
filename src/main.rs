extern crate fern;
extern crate chrono;
extern crate font_kit;
extern crate gl;
extern crate euclid;
extern crate nalgebra_glm as glm;


mod window;

use euclid::{Point2D};
use std::convert::AsRef;
use std::ffi::CString;
use std::fs;
use std::ptr;
use std::path::PathBuf;
use std::str;
use std::mem;

use gl::{types::*};
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::hinting::HintingOptions;
use font_kit::loader::FontTransform;
use font_kit::source::SystemSource;
use glutin::{WindowEvent, Event};

use window::{Window, EventsLoop, LogicalSize};
pub use log::{info, trace, warn};

macro_rules! glCheck {
    () => {{
        if cfg!(debug_assertions) {
            let err = gl::GetError() ;
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

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("logs/rem.log")?)
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()?;
    Ok(()) 
}

fn create_texture(image: &Vec<u8>, x: i32, y: i32) -> u32 {
    let mut id = 0;
    unsafe {
        gl::GenTextures(1, &mut id);

        if id == 0 {
            warn!("Invalid texture id");
        }

        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::BindTexture(gl::TEXTURE_2D, id);

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
            x,
            y,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            image.as_ptr() as *const _
        );

        glCheck!();

        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    id
}

pub fn load_file<P: AsRef<std::path::Path>>(filename: P) -> String {
    let content = fs::read_to_string(&filename).expect(
        format!(
            "Failed to load file: '{}'",
            filename.as_ref().to_str().unwrap()
        )
        .as_str(),
    );
    content
}

pub fn compile_shader(path: &str, ty: GLenum) -> GLuint {
    let shader;

    let src = load_file(path);
    let src = src.as_str();

    unsafe {
        shader = gl::CreateShader(ty);

        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as GLint;

        if gl::GetShaderiv::is_loaded() {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        } else {
            println!("GetShaderiv is not loaded");
        }
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!(
                "{} {}",
                path,
                str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

pub fn link_shader(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);

        gl::LinkProgram(program);

        let mut status = gl::FALSE as GLint;

        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            //if len == 0 {
            //    println!("Status {} but len is 0", status);
            //    return program;
            //}
            buf.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("LinkStatus not valid utf8")
            );
        }
        program
    }
}

extern "system" fn callback(
    source: GLenum,
    gltype: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    userParam: *mut std::ffi::c_void,
) {
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
    println!(
        "{}: {} of {} severity, raised from {}: {}",
        id,
        ty,
        ser,
        sor,
        str::from_utf8(message).unwrap()
    );
}

fn main() {
    setup_logger().unwrap();

    let el = EventsLoop::new();
    let mut window = Window::new(el, LogicalSize::new(400f64, 400f64)).unwrap();
    trace!("Built main window: {:?}", window.get_size());

    let font_name = "FiraCode-Retina";

    let transform = FontTransform::identity();

    let hinting = HintingOptions::None;
    let format = Format::A8;
    let rast_options = RasterizationOptions::GrayscaleAa;

    let font = SystemSource::new().select_by_postscript_name(&font_name).unwrap().load().unwrap();
    println!("Glyph Count: {}", font.glyph_count());
    let char_idx = font.glyph_for_char('A').unwrap();

    let size = 32f32;

    let raster_rec = font.raster_bounds(char_idx, size, &transform, &Point2D::zero(), hinting, rast_options).unwrap();

    println!("Character Size: {:?}", raster_rec);

    let mut canvas = Canvas::new(&raster_rec.size.to_u32(), format);

    let origin = Point2D::new(
            -raster_rec.origin.x,
            raster_rec.size.height + raster_rec.origin.y,
        )
        .to_f32();


    font.rasterize_glyph(&mut canvas, char_idx, size, &transform, &origin, hinting, rast_options).unwrap();

    println!("{:?}", canvas);
    println!("Postscripot Name: {}", font.postscript_name().unwrap());
    println!("Font Name: {}", font.full_name());
    println!("Family Name: {}", font.family_name());
    println!("IsMono: {}", font.is_monospace());
    // println!("{:?}", canvas.pixels);

    window.init_gl();

    unsafe {
        gl::DebugMessageCallback(callback, ptr::null());
    }

    let mut image = Vec::new();

    for y in 0..(raster_rec.size.height as usize) {
        let (row_start, row_end) = (y * canvas.stride, (y + 1) * canvas.stride);
        let row = &canvas.pixels[row_start..row_end];

        for x in 0..(raster_rec.size.width as usize) {
            match canvas.format {
                Format::A8 => {
                    image.push(row[x]);
                    image.push(row[x]);
                    image.push(row[x]);
                }
                Format::Rgb24 => {
                    image.push(row[x * 3 + 0]);
                    image.push(row[x * 3 + 1]);
                    image.push(row[x * 3 + 2]);
                }
                _ => unimplemented!(),
            }
        }
    } 

    // print!("{:?}", image);

    let image = create_texture(&image, canvas.size.width as i32, canvas.size.height as i32);
    println!("Character: {}", image);

    let vs = compile_shader("./res/text_vs.glsl", gl::VERTEX_SHADER);
    let fs = compile_shader("./res/text_fs.glsl", gl::FRAGMENT_SHADER);

    let program = link_shader(vs, fs);

    let mut vao = 0;
    let mut vbo = 0;
    let mut ibo = 0;
    let text_loc;

    let index_data = [0, 1, 2, 0, 2, 3];

    struct Vertex {
        pos: [f32; 3],
        tex: [f32; 2],
        color: [f32; 3]
    }

    let geometry = vec![
        Vertex {
            pos: [100f32, 100f32, 0f32],
            tex: [0.0, 0.0],
            color: [1.0, 1.0, 1.0]
        },
        Vertex {
            pos: [100f32 + raster_rec.size.width as f32, 100f32, 0f32],
            tex: [1.0, 0.0],
            color: [1.0, 1.0, 1.0]
        },
        Vertex {
            pos: [100f32 + raster_rec.size.width as f32, 100f32 + raster_rec.size.height as f32, 0f32],
            tex: [1.0, 1.0],
            color: [1.0, 1.0, 1.0]
        },
        Vertex {
            pos: [100f32, 100f32 + raster_rec.size.height as f32, 0f32],
            tex: [0.0, 1.0],
            color: [1.0, 1.0, 1.0]
        },
    ];

    unsafe {
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        glCheck!();

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);

        gl::Disable(gl::DEPTH_TEST);

        gl::GenVertexArrays(1, &mut vao);
        // generate both with a single call
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ibo);

        gl::UseProgram(program);
        glCheck!();

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (mem::size_of::<u32>() * index_data.len()) as isize,
            index_data.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        let size = mem::size_of::<Vertex>() as usize;

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (size * 4) as isize,
            geometry.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        let float_size = mem::size_of::<f32>();

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3 as i32, gl::FLOAT, gl::FALSE, size as i32, ptr::null());
        glCheck!();

        let mut stride = 3;

        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            2 as i32,
            gl::FLOAT,
            gl::FALSE,
            size as i32,
            (stride * float_size) as *const _,
        );

        stride += 2;

        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(
            2,
            3 as i32,
            gl::FLOAT,
            gl::FALSE,
            size as i32,
            (stride * float_size) as *const _,
        );
        glCheck!();



        let (w, h) = window.get_size();
        let proj = glm::ortho(0f32, w, h, 0f32, -1f32, 1f32);
        let proj_loc = gl::GetUniformLocation(program, CString::new("projection").unwrap().as_ptr());
        gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, proj.as_ptr());
        glCheck!();
         
        text_loc = gl::GetUniformLocation(program, CString::new("atlas").unwrap().as_ptr());
        gl::UseProgram(0);
    }

    let mut running = true;

    while running {
        let process = |e| {
            match e {
                Event::WindowEvent { ref event, window_id: _ } => {
                    match *event {
                        WindowEvent::CloseRequested |
                        WindowEvent::Destroyed => {
                            running = false
                        }
                        _ => {},
                    }
                }
                _ => {},
            }
        };

        window.poll_events(process);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program);
            glCheck!();

            glCheck!();

            gl::ActiveTexture(gl::TEXTURE0 + image);
            gl::BindTexture(gl::TEXTURE_2D, image);

            gl::Uniform1i(text_loc, image as i32);
            glCheck!();

            gl::BindVertexArray(vao);
            glCheck!();

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            glCheck!();

            gl::BindTexture(gl::TEXTURE_2D, 0);

            gl::UseProgram(0);

        }
        
        window.swap_buffers();
    }
}
