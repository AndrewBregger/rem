use std::collections::HashMap;
// main rendering crate
#[macro_use] use crate::render;
// editor area
use crate::pane;
//use pane::{CellSize, Cells, Cursor, HorizontalLayout, Pane, PaneID, PaneKind, VerticalLayout, Layout};
use render::PaneState;

// editing engine
use crate::editor_core;
// user configs
use crate::config;
// main window
use crate::window::{Window, WindowSize};

use crate::timer::Timer;

// font managment(for now)
use crate::font;

use super::size::Size;
use std::convert::{From, Into};
use std::rc::Rc;

// key bindings
// use crate::bindings;

#[derive(Debug)]
pub enum Error {
    CreationError(crate::window::Error),
    RenderError(render::Error),
    EngineError(editor_core::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub enum EditorMode {
    Insert = 0,
    Normal,
    CommandInput,
    Visual,
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

/// The state around the cursor and other info about the text.
/// I do not know what this is yet.
#[derive(Debug)]
struct EditorState;


/// Main structure of the application
pub struct App {
    /// renders the entire application
    renderer: render::Renderer,
    ///  main editing engine, owns the open documents
    engine: editor_core::Engine,
    /// main windows pane
    main_window: MainWindow,
    /// the current mode of the editor.
    mode: EditorMode,
    /// pane and document association.
    docs: HashMap<pane::PaneID, editor_core::DocID>,
    /// until I know where they should actually go.
    cache: render::GlyphCache<font::FreeTypeRasterizer>,
    /// the current settings of the application
    config: Rc<config::Config>,
    /// 
    timer: Timer,
}


impl App {
    pub fn new(mut config: config::Config) -> Result<Self> {
        let event_loop = glutin::EventsLoop::new();

        println!("{:?}", config);

        let window = Window::new(
            event_loop,
            glutin::dpi::LogicalSize::new(config.window.width as f64, config.window.height as f64),
        )
        .map_err(|e| Error::CreationError(e))?;

        // Gl function cannot be loaded until I have a context
        window.init_gl().unwrap();
        check!();

        let dpf = window.dpi_factor();

        println!("Window DPI: {}", dpf);

        let mut renderer = render::Renderer::new(&config).map_err(|e| Error::RenderError(e))?;

        let cache = renderer
            .prepare_font(dpf as f32, &config)
            .map_err(|e| Error::RenderError(e))?;
        check!();

        let cell_size = Self::compute_cell_size(cache.metrics(), &config.font);
        config.cell_size = cell_size;
        println!("Cell Size: {:?}", cell_size);

        let main_window = MainWindow::new(window, cell_size)?;

        let config = Rc::new(config);

        // if a file to open is not given, then an empty, unnamed file is created.
        let mut engine = editor_core::Engine::new(config.clone());

        // let docid = if let Some(filename) = config.get_initial_file() {
        //  create the file with engine and associate it with the main pane.
        // }
        // else {

        let docid = engine
            .create_empty_document()
            .map_err(|e| Error::EngineError(e))?;

        // }

        let mut app = Self {
            renderer,
            engine,
            main_window,
            /// @TODO change back to normal
            mode: EditorMode::Insert,
            docs: HashMap::new(),
            cache,
            config
        };

        // what if a default layout is allowed and this not an edit pane. @FUTUREPROOF
        app.register_document(app.main_window.pane().id(), docid);

        app.prepare()?;

        app.vertical_split_active_pane(docid);

        Ok(app)
    }

    fn prepare(&self) -> Result<()> {
        let (w, h): (f64, f64) = self.main_window.window().get_physical_size().into();

        println!("Window Size: {} {}", w, h);
        self.renderer.set_view_port(w as f32, h as f32);

        let shader = self.renderer.text_shader();

        shader.activate();
        shader.set_cell_size(self.config.cell_size);
        shader.deactivate();

        self.renderer.prepare();

        Ok(())
    }
    
    /*
	pub fn vertical_split_active_pane(&mut self, new_document: editor_core::DocID) {
		let active_pane = self.main_window.active_pane_mut();
		let id = active_pane.vertical_split();
        if let Some(pane) = MainWindow::find_pane_by_id(self.main_window.pane(), id) {
            let size = pane.size().clone();

            self.register_document(id, new_document);
            self.main_window.create_pane_state(size, id);
        }
        else {
            panic!("Failed to split pane");
        }
	}

    pub fn compute_cell_size(metrics: &font::Metrics, font: &config::Font) -> CellSize {
        let width = metrics.average_advance + font.offset.x;
        let height = metrics.line_height + font.offset.y;

        CellSize::new(width, height)
    }
    */

    /// registers a file to be rendered by an edit pane.
    pub fn register_document(&mut self, pane: PaneID, doc: editor_core::DocID) {
        // mayber there should be some other checks here.
       self.docs
           .entry(pane)
           .and_modify(|e| *e = doc)
           .or_insert(doc); 
    }
    
    
    /*
    pub fn render_panes(&self) -> Result<()> {
        let pane = self.main_window.pane();
        self.render_all_panes(pane)
    }
    
    fn render_all_panes(&self, current_pane: &Pane) -> Result<()> {
        match *current_pane.kind() {
            PaneKind::Edit => {
                if let Some(state) = self.main_window.get_pane_state(current_pane.id()) {
                    self.render_pane(current_pane, state)?
                } else {
                    panic!("Unable to find pane state for valid pane");
                }
            },
            PaneKind::Vert(ref layout) => {
                for pane in layout.iter() {
                    self.render_all_panes(pane)?;
                }
            },
            PaneKind::Hor(ref layout) => {
                for pane in layout.iter() {
                    self.render_all_panes(pane)?;
                }
            },
        }
        Ok(())
    }

    // rewrite this to handle the different layouts
    pub fn render_pane(&self, pane: &Pane, state: &PaneState) -> Result<()> {
        // if the pane doens't need to be redrawn then use the cached pane to blit the screen.
        if !state.dirty {
            return Ok(());
        }

        // Set the view, bind uniforms, and bind the frame buffer.
        //        pane.ready_render(&self.renderer).map_err(|e| Error::RenderError(e))?;
        //        pane.bind_frame_as_write();
        
        let (w, h): (f32, f32) = pane.size().clone().into();
        let loc = pane.loc();

        self.renderer.set_view_port_at(w, h, loc.x as f32, loc.y as f32);

        let shader = self.renderer.text_shader();
        
        let ortho = glm::ortho(0f32, w, h, 0f32, -1f32, 1f32);
        let cell_size = self.config.cell_size;

        shader.activate();
        
        shader.set_perspective(ortho);
        shader.set_cell_size(cell_size);

        shader.deactivate();
        
        state.frame.bind_write();

        //
        // Since the entire frame is to be drawn to, the frame doesnt need to be cleared?.
        // clear the current view, in this case it will be the frame buffer.
        self.renderer.clear_frame(None);

        // get the document associated to the pane.
        let document: &editor_core::Document = match self.docs.get(&pane.id()) {
            Some(doc) => match self.engine.get_document(*doc) {
                Some(doc) => doc,
                None => panic!("invalid document id"),
            },
            _ => panic!("Invalid pane/document association"),
        };

        let mut batch = render::Batch::new();
        let render = &self.renderer;
        let cache = &self.cache;
        let cursor = &state.cursor;

        render.draw_pane_background(&mut batch, pane, cursor);

        let lines =
            document.line_slice(state.start_line, state.start_line + pane.cells().y as usize);

        // this should always be zero? depending on the cutter
        let mut cell = (state.start_line as u32, 0 as u32);

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

                let text_color = if cell.0 == cursor.pos().x && cell.1 == cursor.pos().y {
                    [0.0, 0.0, 0.0]
                } else {
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

        render.draw_batch(&batch).map_err(|e| Error::RenderError(e))?;

        Ok(())
    }
    */

    pub fn process_input(&mut self) -> bool {
        let mut running = true;
        let mut events = Vec::new();

        self.main_window
            .window_mut()
            .poll_events(|event| events.push(event));

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
                }
                // Maybe using KeyboardInput and processing that would
                // give a better using experience instead of using ReceivedCharacter
                WindowEvent::ReceivedCharacter(ch) => {
                    println!("{:?}", event);
                    println!("Character Input: {}", *ch);
                    self.process_character_input(*ch);
                    true
                }
                WindowEvent::CloseRequested | WindowEvent::Destroyed => false,
                _ => true,
            },
            _ => true,
        }
    }
    /*
    pub fn active_pane(&self) -> Option<&pane::Pane> {
        None
    }

    fn get_active_pane(&mut self) -> Option<&mut pane::Pane> {
        None
    }

    */

    pub fn editor_mode(&self) -> EditorMode {
        self.mode
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
            }
            EditorMode::CommandInput => self.process_command_input(ch),
            EditorMode::Insert => self.process_character_insert(ch),
            _ => (),
        }
    }

    fn process_command_input(&mut self, ch: char) {}

    fn process_character_insert(&mut self, ch: char) {
        let id = self.main_window.active_pane_mut().id();

        if let Some(state) = self.main_window.get_pane_state_mut(id) {
            let start_index = state.start_line;
            let (x, y) = state.cursor.pos().clone().into();
            state.cursor.advance(if ch == '\t' {
                self.config.tabs.tab_width as u32
            }
            else {
                1
            });

            if let Some(doc_id) = self.get_pane_document_id(id) {
                let op = editor_core::Operation::insert(doc_id.clone(), start_index, x, y, ch);
                match self.engine.execute_on(op) {
                    Err(_) => {
                        // handle the error here
                    },
                    _ => {},
                }
                // this function needs document context information. No, because this is just
                // moving the cursor. The context is needed when inserting an new line, to know
                // if and how many tab need to be inserted.
            }
            else {
                panic!("Corrupted pane and document association");
            }
        }
    }
    
    /*
    pub fn render_window(&self) {
        let pane = self.main_window.pane();
        let (w, h): (f32, f32) = pane.size().clone().into();

        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }

        self.renderer.clear_frame(None);
        self.renderer.set_view_port(w as f32, h as f32);
        
        self.blit_panes(pane);
    }

    fn blit_panes(&self, pane: &Pane) {
        match pane.kind() {
            PaneKind::Edit => {
                let state = self.main_window.get_pane_state(pane.id()).unwrap();
                self.renderer.draw_rendered_pane(self.main_window.window(), pane, state);
            }
            PaneKind::Vert(ref layout) => {
                for pane in layout.iter() {
                    let state = self.main_window.get_pane_state(pane.id()).unwrap();
                    self.renderer.draw_rendered_pane(self.main_window.window(), pane, state);
                }
            },
            PaneKind::Hor(ref layout) => {
                for pane in layout.iter() {
                    let state = self.main_window.get_pane_state(pane.id()).unwrap();
                    self.renderer.draw_rendered_pane(self.main_window.window(), pane, state);
                }
            }
        }
    }
    */

    pub fn swap_buffers(&self) {
        self.main_window.window().swap_buffers()
    }
}
