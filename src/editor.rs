use std::collections::HashMap;
// main rendering crate
#[macro_use] use crate::render;
// editor area
use crate::pane;
use pane::CellSize;
use pane::Pane;

// editing engine
use crate::editor_core;
// user configs
use crate::config;
// main window
use crate::window::{Window, WindowSize};
// font managment(for now)
use crate::font;
// key bindings
// use crate::bindings;

#[derive(Debug)]
pub enum Error {
    CreationError(crate::window::Error),
    RenderError(render::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub struct Settings {
}


/// Main structure of the application
pub struct App {
    /// Main window of the application
    window: Window,
    /// renders the entire application
    renderer: render::Renderer,
    ///  main editing engine, owns the open documents
    engine: editor_core::Engine,
    /// main windows pane
    main_pane: pane::Pane,
    /// pane and document association.
    docs: HashMap<pane::PaneID, editor_core::DocID>,
    /// until I know where they should actually go.
    cache: render::GlyphCache<font::FreeTypeRasterizer>,
    /// the current settings of the application
    config: config::Config,
}

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

impl App {
    pub fn new(config: config::Config) -> Result<Self> {
        let event_loop = glutin::EventsLoop::new();

        println!("{:?}", config);

        let window = Window::new(
            event_loop,
            WindowSize::new(config.window.width, config.window.height)
            ).map_err(|e| Error::CreationError(e))?;

        // Gl function cannot be loaded until I have a context
        window.init_gl().unwrap();
        check!();

        let window_dpi = window.window_dpi();

        let mut renderer = render::Renderer::new(&config).map_err(|e| Error::RenderError(e))?;

        let (width, height) = window.dimensions().into();
        check!();

        let cache = renderer.prepare_font(window_dpi as f32, &config).map_err(|e| Error::RenderError(e))?;
        check!();

        let cell_size = Self::compute_cell_size(cache.metrics(), &config.font);
        check!();

        let (size, pos) = Self::compute_main_pane(width, height, cell_size);
        check!();

        println!("Cell Size: {:?}\nPane Size: {:?}\nPan Pos: {:?}", cell_size, size, pos);
        check!();

        let main_pane = pane::Pane::new(
            size,
            pos,
            cache.font(),
            cache.font_size(),
            cell_size,
            &config);

        let engine = editor_core::Engine::new();

        let app = Self {
           window,
           renderer,
           engine,
           main_pane,
           docs: HashMap::new(),
           cache,
           config 
        };

        app.prepare()?;

        Ok(app)
    }

    fn prepare(&self) -> Result<()> {

        let (w, h) = self.window.dimensions().into();
        println!("Window Size: {} {}", w, h);
        self.renderer.set_view_port(w, h);
// this module should have any unsafe code
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
            gl::Enable(gl::MULTISAMPLE);
            gl::DepthMask(gl::FALSE);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        }

        Ok(())
    }

    pub fn compute_cell_size(metrics: &font::Metrics, font: &config::Font) -> CellSize {
        let width = metrics.average_advance + font.offset.x;
        let height = metrics.line_height + font.offset.y;

        CellSize::new(width, height)
    }

    pub fn compute_main_pane(width: f32, height: f32, cell_size: CellSize) -> (pane::Size, pane::Loc) {
        let cells_x = width / cell_size.x;
        let cells_y = height / cell_size.y;
        println!("{}/{} {}/{}", width, cell_size.x, height, cell_size.y);
        println!("{} {}", cells_x, cells_y);
        println!("{} {}", cells_x * cell_size.x, cells_y * cell_size.y);
        (pane::Size::new(cells_x.ceil() as u32, cells_y.ceil() as u32), pane::Loc::new(0.0, 0.0))
    }
    
    /// temporary function to load a file into the main pane until it is actually implemented.
    pub fn test_doc(&mut self) {
        let doc = self.engine.open_document("src/main.rs").unwrap();
        self.docs.insert(self.main_pane.id, doc);
    }

    // does this need to mut the state?
    // rewrite this to handle the different layouts
    pub fn render_panes(&self) -> Result<()> {
        
        // temp
        let pane = &self.main_pane;

        if !pane.redraw() {
            return Ok(());
        }
        // set the view port and set
        pane.ready_render(&self.renderer).map_err(|e| Error::RenderError(e))?;

        // binds the framebuffer for rendering
        pane.bind_frame();
        self.renderer.clear_frame();

        let id = pane.id;

        let document : &editor_core::Document = match self.docs.get(&id) {
            Some(doc) => match self.engine.get_document(doc.clone()) {
                Some(doc) => doc,
                None => panic!("invalid document id"),
            },
            _ => panic!("Invalid pane/document association")
        };

        let mut batch = render::Batch::new();
        let render = &self.renderer;
        let cache = &self.cache;

        let lines = document.line_slice(pane.start(), pane.start() + pane.size().y as usize);

        let mut cell = (pane.start() as u32, 0 as u32);

        for line in lines {
            for c in line.chars() {
                if c == '\n' {
                    continue;
                }

                if c == ' ' {
                    cell.0 += 1;
                    continue;
                }

                if c == '\t' {
                    cell.0 += self.config.tabs.tab_width as u32;
                    continue;
                }

                // @TODO: handle the case when c is not in the cache
                let glyph = cache.get(c as u32).unwrap();

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

                    texture_id: glyph.atlas as i32,
                };

                if batch.push(instance) {
                    render.draw_batch(&batch);
                    batch.clear();
                }
                cell.0 += 1;
            }
            cell.0 = 0; 
            cell.1 += 1;
        }
        render.draw_batch(&batch);


        Ok(())
    }


    pub fn process_input(&mut self) -> bool {
        let mut running = true;
        let process = |event| {
            use glutin::*;
            match event {
                // LoopDestroyed => running = false,
                Event::DeviceEvent { .. } => (),
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            if input.state == ElementState::Pressed {
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

        self.window.poll_events(process);

        running
    }

    pub fn render_window(&self) {
        self.renderer.clear_frame();

        let pane = &self.main_pane; 
        let (w, h) = self.window.dimensions().into(); 

        self.renderer.set_view_port(w, h);
        self.renderer.draw_rendered_pane(&self.window, pane);
    }

    pub fn swap_buffers(&self) {
        self.window.swap_buffers()
    }
}
