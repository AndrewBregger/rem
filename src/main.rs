extern crate freetype as ft;
extern crate gl;
extern crate glutin;
extern crate image;
extern crate nalgebra_glm as glm;

mod shader;
mod font;
mod render;
mod window;

use font::Rasterizer;
use std::path::PathBuf;
use glutin::*;
use std::sync::mpsc;
use std::thread::Builder;
use std::mem;
use std::ptr;
//
struct App {
    window: window::Window,
    renderer: render::Renderer,
    pub running: bool,
}

static INDEX_DATA: [u32; 6] = [0, 1, 2, 0, 2, 3];

impl App {
    fn new() -> Self {
        let event_loop = window::EventsLoop::new();
        let window = window::Window::new(event_loop, window::Size::from(1078.0, 428.0)).unwrap();
        let dpi = window.window_dpi();

        gl::load_with(|s| window.window.get_proc_address(s) as *const _);
    

        Self {
            window: window,
            renderer: render::Renderer::new(dpi).unwrap(),
            running: true,
        }
    }

    fn setup(&self) {
        
        self.renderer.setup(&self.window);
    }

    fn process_events(&mut self) {
        let mut running = true;
        let process = |event| {
            match event {
                // LoopDestroyed => running = false,
                Event::DeviceEvent { ref event, .. } => (),
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            if input.state == ElementState::Pressed {
                                println!("GoodBye crual world!");
                                running = false;
                            }
                        }
                    }
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => running = false,
                    _ => (),
                },
                _ => (),
            }
        };


        self.window.poll_events(process); 
        self.running = running; 
        
        self.renderer.draw();

        self.window.swap_buffers();
    }
}


struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2]
}

fn build_geometry(glyph: &render::Glyph) -> Vec<Vertex> {
    let mut buf = Vec::new();

    let x = 10f32;
    let y = 10f32;

    // 0.5f,  0.5f, 0.0f,   1.0f, 0.0f, 0.0f,   1.0f, 1.0f,   // top right
    // 0.5f, -0.5f, 0.0f,   0.0f, 1.0f, 0.0f,   1.0f, 0.0f,   // bottom right
    //-0.5f, -0.5f, 0.0f,   0.0f, 0.0f, 1.0f,   0.0f, 0.0f,   // bottom left
    //-0.5f,  0.5f, 0.0f,   1.0f, 1.0f, 0.0f,   0.0f, 1.0f    // top left 
    
    buf.push(Vertex {
        pos: [x + glyph.width, y],
        uv: [glyph.uv_x + glyph.uv_dx, glyph.uv_y],
    }); // top right

    buf.push(Vertex {
        pos: [x + glyph.width, y + glyph.height],
        uv: [glyph.uv_x + glyph.uv_dx, glyph.uv_y + glyph.uv_dy],
    }); // bottom right

    buf.push(Vertex {
        pos: [x, y + glyph.height],
        uv: [glyph.uv_x, glyph.uv_y + glyph.uv_dy],
    }); // bottom left

    buf.push(Vertex {
        pos: [x, y],
        uv: [glyph.uv_x, glyph.uv_y],
    }); // top left

    buf
}

fn main() {
    let event_loop = glutin::EventsLoop::new();    
    let mut window = window::Window::new(event_loop, window::Size::new(1024f64, 726f64)).unwrap();
    
    // How should this be handled?
    //
    // Gl function cannot be loaded until I have a context
    window.init_gl().unwrap();

    let mut vbo = 0;
    let mut ibo = 0;
    let mut vao = 0;

    let mut atlas = render::Atlas::new(window::Size::from(1024f32, 1024f32)).unwrap();

    let font = font::FontDesc {
        style: font::Style::Normal,
        path: std::path::Path::new("dev/DroidSansMono.ttf").to_path_buf(),
        size: font::Size::new(24u16),
        id: 0
    };
    
    let glyph = font::GlyphDesc {
        ch: 'a' as u32,
        font: font.clone(),
    };

    let mut rasterizer = font::FTRasterizer::new(window.window_dpi()).unwrap();

    let glyph = rasterizer.load_glyph(glyph).unwrap();

    let glyph = atlas.insert(glyph).unwrap();

    println!("{:?}", glyph);

    let shader = render::TextShader::new().unwrap();

    let vertices = build_geometry(&glyph);

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ibo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (mem::size_of::<u32>() * INDEX_DATA.len()) as isize,
            INDEX_DATA.as_ptr() as *const _,
            gl::STATIC_DRAW);

        gl::BindBuffer(gl::VERTEX_ARRAY, vbo);
        gl::BufferData(
            gl::VERTEX_ARRAY,
            (mem::size_of::<Vertex>() * vertices.len()) as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW);


        let stride = 2 * mem::size_of::<f32>();

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2 as i32, gl::FLOAT, gl::FALSE, mem::size_of::<Vertex>() as i32, ptr::null());

        // color attribute
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2 as i32, gl::FLOAT, gl::FALSE, mem::size_of::<Vertex>() as i32, stride as *const _);

        gl::BindVertexArray(0);
    }

    let (w, h) = window.dimensions();

    let ortho = glm::ortho(0f32, w as f32, h as f32, 0.0, -1f32, 1f32);


    shader.set_perspective(ortho);
    shader.set_font_atlas(&atlas);

    unsafe {
        gl::Viewport(0, 0, w as i32, h as i32);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
        // gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::MULTISAMPLE);
        gl::DepthMask(gl::FALSE);

        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
    }

    let mut running = true;
    while running {
        let process = |event| {
            match event {
                // LoopDestroyed => running = false,
                Event::DeviceEvent { ref event, .. } => (),
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            if input.state == ElementState::Pressed {
                                println!("GoodBye crual world!");
                                running = false;
                            }
                        }
                    }
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => running = false,
                    _ => (),
                },
                _ => (),
            }
        };
        window.poll_events(process);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        
        atlas.bind();

        unsafe {
            
            gl::BindVertexArray(vao);
            //gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            //gl::BindBuffer(gl::VERTEX_ARRAY, vbo);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());

            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindBuffer(gl::VERTEX_ARRAY, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        
        window.swap_buffers();
    }

}

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
