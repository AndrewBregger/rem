mod layout;
use layout::{VerticalLayout, HorizontalLayout};

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

    pub fn pos(&self) -> &Position {
        &self.pos
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

// should layout panes have ids?
pub enum Pane {
    Vert(VerticalLayout),
    Hor(HorizontalLayout),
    Edit(EditPane),
}


/// An editable section of the screen.
#[derive(Debug)]
pub struct EditPane {
    /// Size of the pane in number of cells.
    size: Size,
    /// The location of the top left of the pane.
    loc: Loc,
    /// The line number at the top of the pane.
    first_line: usize,
    /// offset of the view within the pane.
    view_offset: usize,
    /// Identification of Pane
    pub id: PaneID,
    /// The size of each cell in the pane. This is tied to the font size.
    cell_size: CellSize,
}



impl EditPane {
    pub fn new(size: Size, loc: Loc, cell_size: CellSize) -> Self {
        let id = PaneID::next();

        Self {
            size,
            loc,
            first_line: 0 as usize,
            view_offset: 0 as usize,
            dirty: true,
            id,
            cell_size,
        }
    }

    pub fn id(&self) -> PaneID {
        self.id
    }

/*
    pub fn ready_render(&self, renderer: &render::Renderer) -> Result<(), render::Error> {
        // the size of the pane in pixels.
        let (w, h) = self.compute_render_size();
        
        /// sets the view port of the render to a specific size and location.
        renderer.set_view_port_at(w, h, self.loc.x, self.loc.y);

        let shader = renderer.text_shader();

        shader.activate();
        // set uniforms

        let ortho = glm::ortho(0f32, w, h, 0f32, -1f32, 1f32);
        // let ortho = glm::ortho(0f32, w, 0f32, h, -1f32, 1f33);

        shader.set_perspective(ortho);
        shader.set_cell_size(self.cell_size);

        shader.deactivate();

        Ok(())
    }
*/

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
/*
    pub fn redraw(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self) {
        self.dirty = true;
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

    pub fn advance_cursor(&mut self) {
        // what is the actual logic of advancing the cursor.
        self.cursor.pos.x += 1; 
    }
*/
}
