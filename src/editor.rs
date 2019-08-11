use std::collections::HashMap;
// main rendering crate
use crate::render;
// editor area
use crate::pane;
// editing engine
use crate::editor_core;
// user configs
use crate::config;
// main window
use crate::window::Window;
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

#[derive(Debug, Clone, Copy)]
pub struct CellSize(f32, f32);

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

impl App {
    pub fn new(config: config::Config) -> Result<Self> {
        let event_loop = glutin::EventsLoop::new();

        let window = Window::new(
            event_loop,
            crate::window::Size::from(config.window.width, config.window.height)
            ).map_err(|e| Error::CreationError(e))?;

        // Gl function cannot be loaded until I have a context
        window.init_gl().unwrap();

        let window_dpi = window.window_dpi();

        let mut renderer = render::Renderer::new(&config).map_err(|e| Error::RenderError(e))?;


        let (width, height) = window.dimensions();
        let cache = renderer.prepare_font(window_dpi as f32, &config).map_err(|e| Error::RenderError(e))?;

        let cell_size = Self::compute_cell_size(cache.metrics(), &config.font);

        let (size, pos) = Self::compute_main_pane(width, height, cell_size);

        let main_pane = pane::Pane::new(size, pos, cache.font(), cache.font_size());
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

    // let cell_size = (metrics.average_advance, metrics.line_height);

    // println!("{:?}", cell_size);
    // renderer.text_shader().activate();

    // renderer.text_shader().set_perspective(ortho);
    // // glCheck!();
    // renderer.text_shader().set_cell_size(cell_size);
    // // glCheck!();

    // renderer.text_shader().deactivate();
    // // glCheck!();

    // renderer.rect_shader().activate();

    // renderer.rect_shader().set_perspective(ortho);

    // renderer.rect_shader().set_cell_size(cell_size);

    // renderer.rect_shader().deactivate();
    // glCheck!();

    // unsafe {
    //     gl::Viewport(0, 0, w as i32, h as i32);
    //     gl::Enable(gl::BLEND);
    //     gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
    //     gl::Enable(gl::MULTISAMPLE);
    //     gl::DepthMask(gl::FALSE);
    //     gl::ClearColor(1.0, 1.0, 1.0, 1.0);
    // }
        Ok(())
    }

    pub fn compute_cell_size(metrics: &font::Metrics, font: &config::Font) -> CellSize {
        let width = metrics.average_advance + font.offset.x;
        let height = metrics.line_height + font.offset.y;

        CellSize(width, height)
    }

    pub fn compute_main_pane(width: f32, height: f32, cell_size: CellSize) -> (pane::Size, pane::Loc) {
        let cells_x = width / cell_size.0;
        let cells_y = height / cell_size.1;

        (pane::Size::new(cells_x.ceil() as u32, cells_y.ceil() as u32), pane::Loc::new(0.0, 0.0))
    }
}
