use std::sync::atomic::{AtomicU32, Ordering::SeqCst};
use std::cmp::Eq;

use super::render::{Glyph, GlyphCache};
use super::font::{GlyphKey};
use super::editor_core;

use crate::font::{FontKey, FontSize};
use crate::config;
use crate::render;
use crate::size;

/*
    A pane is rendered by default rendering everything to a framebuffer attached to the pane.
    This framebuffer is then rendered into the correct location on the main window according
    location of the pane. This is intended as a rendering optimization so that panes
    that un-modified panes do not need to be rerendred each frame. The previously rendered
    pane and be draw using the framebuffer.
*/

/// Size of pane
pub type Size = size::Size<u32>;
/// Cell Size
pub type CellSize = size::Size<f32>;
/// position of the cursor.
pub type Position = size::Size<u32>;
/// 2D location
pub type Loc = glm::Vec2;

#[derive(Debug, Clone)]
pub enum CursorMode {
    /// Full boxy cursor
    Box,
    /// Underline cursor
    Underline,
    /// Single line cursor
    Line
}

/// Represents a cursor.
#[derive(Debug, Clone)]
pub struct Cursor {
    /// current position of cursor
    pos: Position,
    /// owner of this cursor
    pane: PaneID,
    /// mode of the cursor, see CursorMode
    mode: CursorMode,
}

impl Cursor {
    pub fn new(pane: PaneID, mode: CursorMode) -> Self {
        Self {
            pos: Position::new(0, 0),
            pane,
            mode
        }
    }

    pub fn at(self, pos: Position) -> Self {
        Self {
            pos,
            pane: self.pane,
            mode: self.mode
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PaneID(u32);

impl PaneID {
    fn next() -> Self {
        static token: AtomicU32 = AtomicU32::new(0);

        Self { 0: token.fetch_add(1, SeqCst) }
    }
}


/// An editable section of the screen.
#[derive(Debug)]
pub struct Pane {
    /// Size of the pane in number of cells.
    size: Size,
    /// The location of the top left of the pane.
    loc: Loc,
    /// The cursor of this pane
    cursor: Cursor,
    /// is the pane active.
    active: bool,
    /// The line number at the top of the pane.
    first_line: usize,
    /// offset of the view within the pane.
    view_offset: usize,
    /// flag to determine if a redraw is needed
    pub dirty: bool,
    /// Identification of Pane
    pub id: PaneID,
    /// font of the pane
    font: FontKey,
    /// font size of this pane
    font_size: FontSize,
    /// The size of each cell in the pane. This is tied to the font size.
    cell_size: CellSize,
    /// the cached rendered pane
    pub frame: render::framebuffer::FrameBuffer,
}

#[derive(Debug, Clone)]
pub struct Line;

impl Pane {
    pub fn new(size: Size, loc: Loc, font: FontKey, font_size: FontSize, cell_size: CellSize, config: &config::Config) -> Self {
        let id = PaneID::next();
        let frame_size: render::framebuffer::FrameSize = Self::pane_size(&size, &cell_size).into();
        println!("{:?} Pane Size: {:?}", id, frame_size);

        Self {
            size,
            loc,
            cursor: Cursor::new(id, config.cursor.normal.clone()),
            active: true,
            first_line: 0 as usize,
            view_offset: 0 as usize,
            dirty: true,
            id,
            font,
            font_size,
            cell_size,
            frame: render::framebuffer::FrameBuffer::with_size(frame_size).unwrap(),
        }
    }

    pub fn set_font(&mut self, font: FontKey) {
        self.font = font
    }

    pub fn increase_font_size(&mut self, inc: f32) {
        self.font_size.pixel_size += inc;
    }

    pub fn decrease_font_size(&mut self, inc: f32) {
        self.font_size.pixel_size -= inc;
    }

    pub fn ready_render(&self, renderer: &render::Renderer) -> Result<(), render::Error> {
        // the size of the pane in pixels.
        let (w, h) = self.pane_size_in_pixels();

        /// sets the view port of the render to a specific size and location.
        renderer.set_view_port_at(w, h, self.loc.x, self.loc.y);

        let shader = renderer.text_shader();

        shader.activate();
        // set uniforms

        let ortho = glm::ortho(0f32, w, h, 0f32, -1f32, 1f32);

        shader.set_perspective(ortho);
        shader.set_cell_size(self.cell_size);

        shader.deactivate();

        Ok(())
    }

    pub fn pane_size(size: &Size, cell_size: &CellSize) -> (f32, f32) {
        let size = glm::Vec2::new(size.x as f32, size.y as f32);
        let dims = glm::Vec2::new(cell_size.x, cell_size.y);

        (size.x * dims.x,
         size.y * dims.y)
    }

    pub fn start(&self) -> usize {
        self.first_line as usize
    }

    pub fn view_offset(&self) -> usize {
        self.view_offset as usize
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn location(&self) -> &Loc {
        &self.loc
    }

    pub fn pane_size_in_pixels(&self) -> (f32, f32) {
        Self::pane_size(&self.size, &self.cell_size)
    }

    pub fn redraw(&self) -> bool {
        self.dirty
    }

    pub fn bind_frame(&self) {
        self.frame.bind()
    }

    pub fn bind_frame_as_read(&self) {
        self.frame.bind_read();
    }

    pub fn bind_frame_as_write(&self) {
        self.frame.bind_write();
    }

    pub fn unbind_frame(&self) {
        self.frame.unbind()
    }
}
