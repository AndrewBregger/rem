// Needed glutin modules and structures
pub use glutin::{
    DeviceEvent, ElementState, Event, EventsLoop, NotCurrent, PossiblyCurrent, VirtualKeyCode,
    WindowEvent, WindowedContext,
};

// needed for error handling
pub use glutin::{ContextError, CreationError};

// needed for size
use crate::size;
use glutin::dpi::{LogicalSize, PhysicalSize};
use std::convert::Into;

#[derive(Debug)]
pub enum Error {
    NoContext(ContextError),
    NoWindow(CreationError),
}

// Uses the entire Result path so result is not redeclared in this scope.
pub type Result<T> = ::std::result::Result<T, Error>;
pub type WindowSize = size::Size<f32>;

impl WindowSize {
    pub fn width(&self) -> f32 {
        self.x
    }

    pub fn height(&self) -> f32 {
        self.y
    }
}

impl Into<LogicalSize> for WindowSize {
    fn into(self) -> LogicalSize {
        LogicalSize::new(self.x.into(), self.y.into())
    }
}

#[derive(Debug)]
pub struct Window {
    // the window event loop. This should be the only event loop we need.
    // Unless other input devices are implemented later. E.G integrated terminal
    event_loop: EventsLoop,

    // The main display window. Eventually this window could be owned by an other window.
    // This will allow for integration of working project between multiple windows.
    context: WindowedContext<PossiblyCurrent>,

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
    pub fn new(event_loop: EventsLoop, size: LogicalSize) -> Result<Self> {
        let context = Self::build_window(&event_loop, size)?;

        Ok(Self {
            event_loop,
            context,
            is_focus: true,
        })
    }

    // gets the dpi of the window, this can be be changed by user action
    // such as when the window is moved to a different monitor.
    // This is needed for font rendering.
    pub fn dpi_factor(&self) -> f64 {
        self.window().get_hidpi_factor()
    }

    pub fn get_inner_size(&self) -> Option<LogicalSize> {
        self.window().get_inner_size()
    }

    pub fn get_physical_size(&self) -> PhysicalSize {
        if let Some(size) = self.get_inner_size() {
            let dpi_factor = self.dpi_factor();

            size.to_physical(dpi_factor)
        } else {
            panic!("Attempting to get size of invalid window");
        }
    }

    pub fn window(&self) -> &glutin::Window {
        self.context.window()
    }

    pub fn set_title(&self, title: &str) {
        self.window().set_title(title)
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn build_window(
        event_loop: &EventsLoop,
        size: LogicalSize,
    ) -> Result<WindowedContext<PossiblyCurrent>> {
        use glutin::{ContextBuilder, WindowBuilder};

        let window = WindowBuilder::new()
            .with_title("REM Editor")
            .with_dimensions(size)
            .with_resizable(true)
            // for now
            .with_decorations(true);
        // test right now
        // .with_transparency(true);
        // @TODO: Window icon.
        // .window_window_icon(???)

        let context = ContextBuilder::new()
            // these should be checked or passed an not assumed. {
            .with_gl_debug_flag(true)
            .with_gl_robustness(glutin::Robustness::TryRobustLoseContextOnReset)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 0)))
            .with_double_buffer(Some(true))
            .with_srgb(true)
            // .with_vsync(true)
            // }
            .build_windowed(window, event_loop)
            .map_err(|e| Error::NoWindow(e))?;

        Ok(unsafe {
            context
                .make_current()
                .map_err(|(_, e)| Error::NoContext(e))?
        })
    }

    #[cfg(target_os = "macos")]
    fn build_window(
        event_loop: &EventsLoop,
        size: LogicalSize,
    ) -> Result<WindowedContext<PossiblyCurrent>> {
        use super::glutin::os::macos::WindowBuilderExt;
        use super::glutin::{ContextBuilder, WindowBuilder};

        let windowbuilder = WindowBuilder::new()
            .with_title("REM Editor")
            .with_dimensions(size)
            .with_resizable(true)
            // for now
            .with_decorations(true);
        // test right now
        // .with_transparency(true);

        let context = ContextBuilder::new()
            // these should be checked or passed an not assumed. {
            .with_gl_debug_flag(true)
            .with_gl_robustness(glutin::Robustness::TryRobustLoseContextOnReset)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 0)))
            .with_double_buffer(Some(true))
            .with_srgb(true)
            // .with_vsync(true)
            // }
            .build_windowed(windowbuilder, event_loop)
            .map_err(|e| Error::NoWindow(e))?;

        Ok(unsafe {
            context
                .make_current()
                .map_err(|(_, e)| Error::NoContext(e))?
        })
    }

    pub fn poll_events<F: FnMut(Event)>(&mut self, f: F) {
        self.event_loop.poll_events(f);
    }

    pub fn swap_buffers(&self) {
        self.context.swap_buffers().unwrap();
    }

    // this should never fail.
    pub fn init_gl(&self) -> Result<()> {
        gl::load_with(|s| self.context.get_proc_address(s) as *const _);
        Ok(())
    }
}
