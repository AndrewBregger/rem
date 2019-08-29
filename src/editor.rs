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

#[derive(Debug, Clone, Copy)]
pub enum EditorMode {
    Insert = 0,
    Normal,
    CommandInput,
    Visual,
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
    /// the current mode of the editor.
    mode: EditorMode,
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
            glutin::dpi::LogicalSize::new(config.window.width as f64, config.window.height as f64)
            ).map_err(|e| Error::CreationError(e))?;

        // Gl function cannot be loaded until I have a context
        window.init_gl().unwrap();
        check!();

        let dpf = window.dpi_factor();

        println!("Window DPI: {}", dpf);

        let mut renderer = render::Renderer::new(&config).map_err(|e| Error::RenderError(e))?;

        let (width, height) = if let Some(s) = window.get_inner_size() {
            let s = s.to_physical(window.dpi_factor());
            (s.width as f32, s.height as f32)
        }
        else {
            unreachable!();
        };

        let cache = renderer.prepare_font(dpf as f32, &config).map_err(|e| Error::RenderError(e))?;
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
           /// @TODO change back to normal
           mode: EditorMode::Insert,
           docs: HashMap::new(),
           cache,
           config 
        };

        app.prepare()?;

        Ok(app)
    }

    fn prepare(&self) -> Result<()> {

        let (w, h) = if let Some(s) = self.window.get_inner_size() {
            let s = s.to_physical(self.window.dpi_factor());

            (s.width as f32, s.height as f32)
        }
        else {
            unreachable!();
        };

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
    
    /// TEMPORARY function to load a file into the main pane until it is actually implemented.
    pub fn test_doc(&mut self) {
        let doc = self.engine.open_document("src/main.rs").unwrap();
        self.docs.insert(self.main_pane.id, doc);
    }

    /// TEMPORARY making sure the caching works when rendering.
    pub fn pane_rendered(&mut self) {
        self.main_pane.dirty = false;
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
        pane.bind_frame_as_write();

        self.renderer.clear_frame(None);

        // Since the entire frame is to be drawn to, the frame doesnt need to be cleared?.

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
        let cursor = pane.cursor();

        render.draw_pane_background(&mut batch, pane);

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

                let text_color = if cell.0 == cursor.pos().x &&
                   cell.1 == cursor.pos().y  {
                    [0.0, 0.0, 0.0]
                }
                else {
                    [1.0, 1.0, 1.0]
                };

                let instance = render::InstanceData {
                    x: cell.0 as f32,
                    y: cell.1 as f32,
                    
                    // text metrics offsets for the character
                    width: glyph.width,
                    height: glyph.height,
                    offset_x: glyph.bearing_x, // - 1.0,
                    offset_y: glyph.bearing_y + 2.0,

                    // texture coordinates
                    uv_x: glyph.uv_x,
                    uv_y: glyph.uv_y,
                    uv_dx: glyph.uv_dx,
                    uv_dy: glyph.uv_dy,

                    tr: text_color[0],
                    tg: text_color[1],
                    tb: text_color[2],

                    br: 0.0,
                    bg: 0.0,
                    bb: 0.0,
                    ba: 1.0,

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
        let mut events = Vec::new();

        self.window.poll_events(|event| {
            events.push(event)
        });
        
        if events.is_empty() {
            return true;
        }

        for event in events {
            running = self.process_event(&event);
        }

        running
    }

    pub fn process_event(&mut self, event: &glutin::Event) -> bool {
        use glutin::*;
        match *event {
            // LoopDestroyed => running = false,
            Event::DeviceEvent { .. } => true,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::KeyboardInput { ref input, .. } => {
                    println!("{:?}", input);
                    true
                },
                // Maybe using KeyboardInput and processing that would
                // give a better using experience instead of using ReceivedCharacter
                WindowEvent::ReceivedCharacter(ch) => {
                    println!("Character Input: {}", *ch);
                    self.process_character_input(*ch);
                    true
                }, 
                WindowEvent::CloseRequested | WindowEvent::Destroyed => false,
                _ => true,
            },
            _ => true,
        }
    }

    pub fn active_pane(&self) -> Option<&pane::Pane> {
        if self.main_pane.active() {
            Some(&self.main_pane)
        }
        else {
            None
        }
    }

    fn get_active_pane(&mut self) -> Option<&mut pane::Pane> {
        if self.main_pane.active() {
            Some(&mut self.main_pane)
        }
        else {
            None
        }
    }

    pub fn editor_mode(&self) -> EditorMode {
        self.mode
    }

    fn get_pane_document(&self, pane: pane::PaneID) -> Option<&editor_core::Document> {
        unimplemented!();
    }

    fn get_pane_document_id(&self, pane_id: pane::PaneID) -> Option<&editor_core::DocID> {
        self.docs.get(&pane_id)
    }

    fn process_character_input(&mut self, ch: char) {
        match self.editor_mode() {
            EditorMode::Normal => {
                if ch == ':' {
                    self.mode = EditorMode::CommandInput;
                }
                // otherwise the input is ignored.
            },
            EditorMode::CommandInput => self.process_command_input(ch),
            EditorMode::Insert => self.process_character_insert(ch),
            _ => ()
        }
    }

    fn process_command_input(&mut self, ch: char) {
    }

    fn process_character_insert(&mut self, ch: char) {
        if let Some(pane) = self.get_active_pane() {
            // location of the cursor.
            let cursor = pane.cursor();
            let x = cursor.pos().x;
            let y = cursor.pos().y;
            // the first visable line in the pane.
            let start_index = pane.start();
            let pane_id = pane.id;
            pane.advance_cursor();
            pane.set_dirty();
            println!("{:?}", pane.cursor());
            // for the drop of pane reference
            drop(pane);

            if let Some(doc_id) = self.get_pane_document_id(pane_id) {
                let op = editor_core::Operation::insert(doc_id.clone(), start_index, x, y, ch);
                self.engine.execute_on(op);
            }
            else {
                panic!("Corrupted pane and document association");
            }

        } 
    }

    pub fn render_window(&self) {
        let pane = &self.main_pane; 
        let (w, h) = if let Some(s) = self.window.get_inner_size() {
            let s = s.to_physical(self.window.dpi_factor());

            (s.width as f32, s.height as f32)
        }
        else {
            unreachable!();
        };

        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
        self.renderer.clear_frame(None);

        self.renderer.set_view_port(w, h);
        self.renderer.draw_rendered_pane(&self.window, pane);
    }

    pub fn swap_buffers(&self) {
        self.window.swap_buffers()
    }
}
