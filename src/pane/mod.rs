pub mod editpane;
pub mod layout;

pub use editpane::{Cursor, CursorMode};
pub use layout::{HorizontalLayout, VerticalLayout};

use crate::size;
use std::sync::atomic::{AtomicU32, Ordering::SeqCst};

/*
    A pane is rendered by default rendering everything to a framebuffer attached to the pane.
    This framebuffer is then rendered into the correct location on the main window according
    location of the pane. This is intended as a rendering optimization so that panes
    that un-modified panes do not need to be rerendred each frame. The previously rendered
    pane and be draw using the framebuffer.
*/

/// Size of pane
pub type Cells = size::Size<u32>;
/// Cell Size
pub type CellSize = size::Size<f32>;
/// position of the cursor.
pub type Position = size::Size<u32>;
pub type Size = size::Size<f32>;
/// 2D location
pub type Loc = glm::Vec2;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PaneID(u32);

impl Cells {
    pub fn compute_cells(width: f32, height: f32, cell_size: CellSize) -> Self {
        let cells_x = width / cell_size.x;
        let cells_y = height / cell_size.y;

        Self::new(cells_x as u32, cells_y as u32)
    }
}

impl PaneID {
    fn next() -> Self {
        static TOKEN: AtomicU32 = AtomicU32::new(0);

        Self {
            0: TOKEN.fetch_add(1, SeqCst),
        }
    }
}

// should layout panes have ids?
#[derive(Debug, Clone)]
pub enum PaneKind {
    /// The panes are ordered horizontally
    Vert(VerticalLayout),
    /// The panes are ordered horizontally
    Hor(HorizontalLayout),
    /// of the data needed for rending a pane is stored else where.
    Edit,
}

#[derive(Debug, Clone)]
pub struct Pane {
    /// Identification of this pane.
    id: PaneID,
    /// what type of pane is this { Vertical Layout, HorizontalLayout.
    kind: PaneKind,
    /// The size of the pane in pixels
    size: Size,
    /// Size of the pane in number of cells.
    // The number of the parents cell are to be used to render this pane.
    cells: Cells,
    /// The location of the bottom left of the pane.
    loc: Loc,
}

impl Pane {
    pub fn new(kind: PaneKind, size: Size, cells: Cells, loc: Loc) -> Self {
        let id = PaneID::next();

        Self {
            id,
            kind,
            size,
            cells,
            loc,
        }
    }

    pub fn kind(&self) -> &PaneKind {
        &self.kind
    }

    pub fn id(&self) -> PaneID {
        self.id
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn cells(&self) -> &Cells {
        &self.cells
    }

    pub fn loc(&self) -> &Loc {
        &self.loc
    }

    pub fn on_resize(&mut self, size: Size, cells: Cells, cell_size: CellSize) {
        self.size = size;
        self.cells = cells;

        self.update_children(cell_size);
    }

    pub fn on_resize_and_move(&mut self, size: Size, cells: Cells, loc: Loc, cell_size: CellSize) {
        self.size = size;
        self.cells = cells;
        self.loc = loc;

        self.update_children(cell_size);
    }

    pub fn update_children(&mut self, cell_size: CellSize) {}
}
