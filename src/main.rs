extern crate gl;
extern crate glfw;
extern crate image;

use gl::types::*;
use glfw::{Action, Context, Key};
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;

macro_rules! glCheck {
    () => {{
        //   if cfg!(debug_assertions) {
        let err = gl::GetError();
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
        //  }
    }};
}

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;

    unsafe {
        shader = gl::CreateShader(ty);

        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as GLint;

        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

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
                "{:?} {}",
                ty,
                str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

fn link_shader(vs: GLuint, fs: GLuint) -> GLuint {
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

//fn load_file(file: &str) -> &str {
//
//}

static VERTEX_DATA: [GLfloat; 16] = [
    -0.5,  0.5, 0.0, 0.0, // Top-left
     0.5,  0.5, 1.0, 0.0, // Top-right
     0.5, -0.5, 1.0, 1.0, // Bottom-right
    -0.5, -0.5, 0.0, 1.0  // Bottom-left
];

static INDEX_DATA: [u32; 6] = [0, 1, 2, 0, 2, 3];

fn load_image(path: &str) -> GLuint {
    let mut tex = 0;
    unsafe {
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);

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

        let image = image::open(path).unwrap().to_rgba();

        let (w, h) = image.dimensions();
        let vec = image.into_raw();
        let img_ptr: *const std::ffi::c_void = vec.as_ptr() as *const _ as *const std::ffi::c_void;

        println!("{:?} ({}, {})", img_ptr, w, h);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            w as i32,
            h as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            img_ptr,
        );
    
        gl::BindTexture(gl::TEXTURE_2D, 0);
        glCheck!();
    }
    tex
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 1));

    let (mut window, events) = glfw
        .create_window(300, 300, "REM", glfw::WindowMode::Windowed)
        .expect("Failed to create window.");

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    window.set_key_polling(true);
    window.make_current();

    window.set_floating(true);

    let vs_source = include_str!("../shaders/vs.glsl");
    let fs_source = include_str!("../shaders/fs.glsl");

    let vs = compile_shader(vs_source, gl::VERTEX_SHADER);
    let fs = compile_shader(fs_source, gl::FRAGMENT_SHADER);

    let program = link_shader(vs, fs);

    let mut vbo = 0;
    let mut vao = 0;
    let mut ibo = 0;
    let im: GLuint;

    let vertex_size: GLint = (4 * mem::size_of::<GLfloat>()) as GLint;
    let tex_coord_offset: GLuint = (2 * mem::size_of::<GLfloat>()) as GLuint;

    //let pos_stride: GLint = (2 * mem::size_of::<GLfloat>()) as GLint;
    //let tex_coord_stride: GLint = (2 * mem::size_of::<GLfloat>()) as GLint;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::GenBuffers(1, &mut ibo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (VERTEX_DATA.len() * mem::size_of::<f32>()) as GLsizeiptr,
            mem::transmute(&VERTEX_DATA[0]),
            gl::STATIC_DRAW,
        );

        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (INDEX_DATA.len() * mem::size_of::<u32>()) as GLsizeiptr,
            mem::transmute(&INDEX_DATA[0]),
            gl::STATIC_DRAW,
        );

        gl::UseProgram(program);

        let attr_pos = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr());
        let attr_tex = gl::GetAttribLocation(program, CString::new("texcoords").unwrap().as_ptr());

        println!("{} {}", attr_pos, attr_tex);
        gl::EnableVertexAttribArray(attr_pos as GLuint);
        gl::VertexAttribPointer(
            attr_pos as GLuint,
            2,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            vertex_size,
            ptr::null(),
        );

        gl::EnableVertexAttribArray(attr_tex as GLuint);
        gl::VertexAttribPointer(
            attr_tex as GLuint,
            2,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            vertex_size,
            tex_coord_offset as *const _,
        );
        glCheck!();

        im = load_image("dev/images.png");
        println!("{}", im);

        let un_tex = gl::GetUniformLocation(program, CString::new("text").unwrap().as_ptr());
        glCheck!();
        println!("{}", un_tex);
        gl::Uniform1i(un_tex, im as i32);
        glCheck!();
    }

    // unsafe { gl::ClearColor(0.3f32, 0.5f32, 0.4f32, 1.0f32); }
    unsafe {
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
    }
    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);

            let (w, h) = window.get_size();
            println!("({}, {})", w, h);
            handle_window_event(&mut window, event);
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, im);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            glCheck!();

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        window.swap_buffers();
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        }
        _ => {}
    }
}
