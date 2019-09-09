pub mod editpane;
pub mod layout;

pub use editpane::{Cursor, CursorMode};
pub use layout::{HorizontalLayout, VerticalLayout, Layout, Orientation};

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

    pub fn kind_mut(&mut self) -> &mut PaneKind {
        &mut self.kind
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

	pub fn vertical_split(&mut self) -> PaneID {
		match *self.kind() {
			PaneKind::Edit => {
				self.make_into_vertical_layout()
//				let new_parent = layout::VerticalLayout::new();
			},
			_ => {
				unimplemented!();
			}
		}
	}
	
	fn make_into_vertical_layout(&mut self) -> PaneID {
		let size = self.size();
		let cells = self.cells();
		let loc = self.loc();
		
		let vertical_layout = layout::VerticalLayout::new();
		
		let mut parent_pane = Self::new(PaneKind::Vert(vertical_layout), size.clone(), cells.clone(), loc.clone());
		
		let mut right_pane = Self::new(PaneKind::Edit, size.clone(), cells.clone(), loc.clone());
		let new_id = right_pane.id();
		
		parent_pane.add_child_pane(self.clone());
		parent_pane.add_child_pane(right_pane);
		
		parent_pane.resize_children();
		
		*self = parent_pane;
	    
        new_id
	}
	
	fn resize_children(&mut self) {
		let psize = self.size().clone();
		let pcells = self.cells().clone();
		let mut new_loc = self.loc().clone();
		
		match self.kind {
			PaneKind::Vert(ref mut layout) => {
//				layout as dyn Layout
				let num = layout.num_children();
				let new_size = Size::new(psize.x, psize.y / num as f32);
				let new_cells = Cells::new(pcells.x, pcells.y / num as u32);
				
				for mut pane in layout.iter_mut() {
					pane.size = new_size;
					pane.cells = new_cells;
					pane.loc = new_loc;
					
					new_loc.x += new_size.x;
					
					pane.resize_children();
				}
			},
			PaneKind::Hor(ref mut layout) => {
//				layout as dyn Layout
				let num = layout.num_children();
				
				let new_size = Size::new(psize.x / num as f32, psize.y);
				let new_cells = Cells::new(pcells.x / num as u32, pcells.y);
				
				for mut pane in layout.iter_mut() {
					pane.size = new_size;
					pane.cells = new_cells;
					pane.loc = new_loc;
					
					new_loc.y += new_size.y;
					
					pane.resize_children();
				}
			},
			_ => return,
		};	
	}
	
	fn add_child_pane(&mut self, pane: Pane) {
		match self.kind {
			PaneKind::Vert(ref mut layout) => {
				layout.add_child(pane);
			},
			PaneKind::Hor(ref mut layout) => {
				layout.add_child(pane);
			},
			_ => {
				panic!("Attempting to add child pane to non-layout pane");
			}
		}
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
