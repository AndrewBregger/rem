extern crate freetype as ft;
extern crate gl;
extern crate glutin;
extern crate image;
extern crate nalgebra_glm as glm;

//mod shader;
mod font;
mod render;
mod window;

//use gl::types::*;
//use std::ffi::CString;
//use std::mem;
//use std::ptr;
//use std::str;
use std::path::PathBuf;
//use glutin::dpi::LogicalSize;
use glutin::*;
use std::sync::mpsc;
use std::thread::Builder;
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

struct App {
    window: window::Window,
    renderer: render::Renderer,
}

fn main() {
    let event_loop = window::EventsLoop::new();

    let mut window = window::Window::new(event_loop, window::Size::from(1078.0, 428.0)).unwrap();

    window.run();

    //let (trans, recv) = mpsc::channel();
    //let worker_thread = Builder::new().name("renderer".to_string());

    //let render_done = worker_thread.spawn(move || {
    //    let window = window::Window::new(event_loop, window::Size::new(1078.0, 426.0)).unwrap();
    //    render(window_context, recv);
    //});
}

/*
fn run(event_loop: &window::EventLoop) {

    trans.send(()).ok().expect("Failed to close render thread");
    let _ = render_done;
}
*/

/*
fn render(context: glutin::ContextWrapper<glutin::NotCurrent, glutin::window::Window>, recv: mpsc::Receiver<()>) {
    // get the gl_context to render from
    let render_context = unsafe { context.make_current().unwrap() };

    // loads all of the functions on this thread.
    gl::load_with(|s| render_context.get_proc_address(s) as *const _);

    // render shit!!
    loop {
        if recv.try_recv() == Ok(()) { break };

        unsafe {
            gl::ClearColor(0.2, 0.4, 0.4, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        render_context.swap_buffers().unwrap();
    }
}
*/
