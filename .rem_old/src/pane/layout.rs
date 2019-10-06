use core::slice::{IterMut, Iter};
use super::size::Size;
use super::{Pane, PaneID};

pub enum Orientation {
    Vert,
    Hori,
}

pub trait Layout {
    /// Creates a new layout
    fn new() -> Self;

    /// Returns the orientation of the layout.
    fn orientation(&self) -> Orientation;

//    / gets the id of the layout.
//    fn id(&self) -> PaneID;

	fn add_child(&mut self, pane: Pane);
	
	/// the number of direct children of this pane.
	fn num_children(&self) -> usize;
	
	fn iter(&self) -> Iter<Pane>;
	
	fn iter_mut(&mut self) -> IterMut<'_, Pane>;
}

#[derive(Debug, Clone)]
pub struct VerticalLayout {
    // The pane this layout belongs to.
//    id: PaneID,
    elements: Vec<Pane>,
}

#[derive(Debug, Clone)]
pub struct HorizontalLayout {
    // The pane this layout belongs to.
//    id: PaneID,
    elements: Vec<Pane>,
}

impl Layout for VerticalLayout {
    /// Creates a new layout
    fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Returns the orientation of the layout.
    fn orientation(&self) -> Orientation {
        Orientation::Vert
    }

//    /// gets the id of the layout.
//    fn id(&self) -> PaneID {
//        self.id
//    }

	fn add_child(&mut self, pane: Pane) {
		self.elements.push(pane);
	}
	
	fn num_children(&self) -> usize {
		self.elements.len()
	}
	
	fn iter_mut(&mut self) -> IterMut<'_, Pane> {
		self.elements.iter_mut()
	}
	
	fn iter(&self) -> Iter<'_, Pane> {
		self.elements.iter()
	}
}

impl Layout for HorizontalLayout {
    /// Creates a new layout
    fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Returns the orientation of the layout.
    fn orientation(&self) -> Orientation {
        Orientation::Hori
    }

//    /// gets the id of the layout.
//    fn id(&self) -> PaneID {
//        self.id
//    }

	fn add_child(&mut self, pane: Pane) {
		self.elements.push(pane);
	}
	
	fn num_children(&self) -> usize {
		self.elements.len()
	}
	
	fn iter_mut(&mut self) -> IterMut<'_, Pane> {
		self.elements.iter_mut()
	}
	
	fn iter(&self) -> Iter<'_, Pane> {
		self.elements.iter()
	}
}
