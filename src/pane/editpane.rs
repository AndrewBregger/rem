use super::{PaneID, CellSize, Size, Position, Loc};

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

/*
/// An editable section of the screen.
#[derive(Debug, Clone)]
pub struct EditPane {
    /// The line number at the top of the pane.
    first_line: usize,
    /// offset of the view within the pane.
    view_offset: usize,
}



impl EditPane {
    pub fn new(size: Size, loc: Loc) -> Self {
        let id = PaneID::next();

        Self {
            first_line: 0 as usize,
            view_offset: 0 as usize,
        }
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

    pub fn start(&self) -> usize {
        self.first_line as usize
    }

    pub fn view_offset(&self) -> usize {
        self.view_offset as usize
    }
}
*/
