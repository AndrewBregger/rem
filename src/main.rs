extern crate gl;
extern crate glutin;
extern crate image;
extern crate freetype as ft;
extern crate nalgebra_glm as glm;

//mod shader;
//mod font;
//mod renderer;
//mod window;

use gl::types::*;
// use std::thread::Builder;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;
use std::path::PathBuf;
use glutin::dpi::LogicalSize;
use glutin::{event_loop::EventLoop, window::WindowBuilder, event::Event};
// use std::sync::mpsc;
//
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

/*
struct Texture {
    pub id: u32,
    data: Vec<u8>,
    w: i32,
    h: i32,
    format: PixelFormat,
}

struct EmptyTexture {
    w: i32,
    h: i32,
    format: PixelFormat,
}

impl EmptyTexture {
    pub fn new(w: i32, h: i32, format: PixelFormat) -> Self {
        Self {
            w,
            h,
            format
        }
    }

    pub fn to_texture(self, data: Vec<u8>) -> Texture {
        Texture::new(data, self.w, self.h, self.format)
    }
}

impl Texture {
    pub fn new(data: Vec<u8>, w: i32, h: i32, format: PixelFormat) -> Self {
        Self {
            id: 0,
            data,
            w,
            h,
            format
        }
    }

    pub fn gl_format(&self) -> GLenum {
        match &self.format {
            PixelFormat::RGB => gl::RGB,
            PixelFormat::RGBA => gl::RGBA,
            PixelFormat::Red => gl::RED,
            PixelFormat::Alpha => gl::ALPHA,
        }
    }

    pub fn width(&self) -> f32 {
        self.w as f32
    }

    pub fn height(&self) -> f32 {
        self.h as f32
    }


    pub fn activate(&self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + self.id);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn deactivate(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn buffer(&self) -> &Vec<u8> {
        &self.data
    }
    
    // maybe add a options: TextureOptions
    pub fn init(&mut self) {
        self.allocate();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);

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

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                self.gl_format() as i32,
                self.w,
                self.h,
                0,
                self.gl_format(),
                gl::UNSIGNED_BYTE,
                self.data.as_ptr() as *const _,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl GLResource for Texture {
    fn allocate(&mut self)  {
        unsafe {
            gl::GenTextures(1, &mut self.id);
        }
    }

    fn destroy(&self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
*/


fn main() {
    
    let el = EventLoop::new(); 

    let wb = WindowBuilder::new()
        .with_title("REM")
        .with_inner_size(LogicalSize::new(1024.0, 720.0));

    let contextBuilder = glutin::ContextBuilder::new().build_windowed(wb, &el).unwrap();

    
    el.run(|event, &_, control_flow| {
        match event {
            Event::DeviceEvent {
                event,
                ..
            } => {
                match event {
                    event::Event::DeviceEvent::Key(k) => {
                        match k.virtual_keycode {
                            Some(glutin::event::VirtualKey::Escape) => glutin::event_loop::ControlFlow::Exit,
                            _ => glutin::event_loop::ControlFlow::Poll,
                        }
                    },
                    _ =>  glutin::event_loop::ControlFlow::Poll,
                }
            },
            _ => glutin::event_loop::ControlFlow::Poll,
            Event::LoopDestroyed => glutin::event_loop::ControlFlow::Exit,
        }
    });

    //let (trans, recv) = mpsc::channel();
    //let worker_thread = Builder::new().name("renderer".to_string());

    //let render_done = worker_thread.spawn(move || {
    //    render(render_context, recv);
    //});



    //trans.send(()).ok().expect("Failed to close render thread");
    //let _ = render_done;
}

/*
fn render(mut context: glfw::RenderContext, recv: mpsc::Receiver<()>) {
    context.make_current();


    let (_vao, _vbo, _ibo, im) = setup();

    unsafe {
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
    }

    loop {
        if recv.try_recv() == Ok(()) { break };

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0 + im);
            gl::BindTexture(gl::TEXTURE_2D, im);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            glCheck!();

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        context.swap_buffers();
    }
}
*/
