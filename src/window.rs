// Needed glutin modules and structures
pub use super::glutin::{
    ContextBuilder, DeviceEvent, ElementState, Event, EventsLoop, NotCurrent, PossiblyCurrent,
    VirtualKeyCode, WindowBuilder, WindowEvent, WindowedContext,
};

// needed for error handling
pub use super::glutin::{ContextError, CreationError};

// needed for size
use super::glutin::dpi::LogicalSize;
use std::convert::Into;

#[derive(Debug)]
pub enum Error {
    NoContext(ContextError),
    NoWindow(CreationError),
}

// Uses the entire Result path so result is not redeclared in this scope.
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub struct Size(f64, f64);

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Size(width, height)
    }

    pub fn from(width: f32, height: f32) -> Self {
        Size::new(width as f64, height as f64)
    }

    pub fn width(&self) -> f64 {
        self.0
    }

    pub fn height(&self) -> f64 {
        self.1
    }
}

impl Into<LogicalSize> for Size {
    fn into(self) -> LogicalSize {
        LogicalSize::new(self.0, self.1)
    }
}

#[derive(Debug)]
pub struct Window {
    // the window event loop. This should be the only event loop we need.
    // Unless other input devices are implemented later. E.G integrated terminal
    event_loop: EventsLoop,

    // The main display window. Eventually this window could be owned by an other window.
    // This will allow for integration of working project between multiple windows.
    window: WindowedContext<PossiblyCurrent>,

    // the current size of the window.
    // When a window size is updated this value will be updated as well.
    size: Size,

    // is the mouse visible
    // @NTOE: Maybe this should be associated with a pane. But
    //          does it make since for one pane to have a visible mouse and
    //          another not?
    mouse_visible: bool,

    // Is this window the active window of the user.
    is_focus: bool,
}

// @TODO: extend window building to platform specific window builders to
//        allow for control over window properties.
//        I.E. Use glutin::os::unit::WindowBuilderExt to build linxu/unix windows
//              It gives access to other properties such as gtk themes.
//        e.t.c
//
//        For now I am only using the generic window builder.
impl Window {
    // Change size of a WindowConfig or Config object so the window
    // is created to some properties specified by the user.
    // A config created by loading some config file (rem.config or whatever)
    pub fn new(event_loop: EventsLoop, size: Size) -> Result<Self> {
        let window = Self::build_window(&event_loop, size)?;
        gl::load_with(|s| window.get_proc_address(s) as *const _);

        Ok(Self {
            event_loop,
            window,
            size,
            mouse_visible: true,
            is_focus: true,
        })
    }

    // gets the dpi of the window, this can be be changed by user action
    // such as when the window is moved to a different monitor.
    // This is needed for font rendering.
    pub fn window_dpi(&self) -> f64 {
        self.window.window().get_hidpi_factor()
    }

    fn build_window(
        event_loop: &EventsLoop,
        size: Size,
    ) -> Result<WindowedContext<PossiblyCurrent>> {
        let window = WindowBuilder::new()
            .with_title("REM Editor")
            .with_dimensions(size.into())
            .with_resizable(true)
            // for now
            .with_decorations(true)
            // test right now
            .with_transparency(true);
        // @TODO: Window icon.
        // .window_window_icon(???)

        let context = ContextBuilder::new()
            // these should be checked or passed an not assumed. {
            .with_srgb(true)
            .with_vsync(true)
            // }
            .build_windowed(window, event_loop)
            .map_err(|e| Error::NoWindow(e))?;

        Ok(unsafe {
            context
                .make_current()
                .map_err(|(_, e)| Error::NoContext(e))?
        })
    }

    pub fn run(&mut self) {
        let mut running = true;
        while running {
            self.event_loop.poll_events(|event| {
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
                };
            });
            unsafe {
                gl::ClearColor(0.2, 0.4, 0.4, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            self.window.swap_buffers().unwrap();
        }
    }
}