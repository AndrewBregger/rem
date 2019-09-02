
use super::{Pane, PaneID};
use super::size::Size;

pub enum Orientation {
    Vert,
    Hori,
}

pub trait Layout {
    /// Creates a new layout
    fn new(id: PaneID) -> Self;
    
    /// Returns the orientation of the layout.
    fn orientation(&self) -> Orientation;
    
    /// gets the id of the layout.
    fn id(&self) -> PaneID;
}

#[derive(Debug, Clone)]
pub struct VerticalLayout {
    // The pane this layout belongs to.
    id: PaneID,
    elements: Vec<Pane>,
}

#[derive(Debug, Clone)]
pub struct HorizontalLayout {
    // The pane this layout belongs to.
    id: PaneID,
    elements: Vec<Pane>,
}

impl Layout for VerticalLayout {
    /// Creates a new layout
    fn new(id: PaneID) -> Self {
        Self {
            id,
            elements: Vec::new(),
        }
    }
    
    /// Returns the orientation of the layout.
    fn orientation(&self) -> Orientation {
        Orientation::Vert
    }
    
    /// gets the id of the layout.
    fn id(&self) -> PaneID {
        self.id
    }
}

impl Layout for HorizontalLayout {
    /// Creates a new layout
    fn new(id: PaneID) -> Self {
        Self {
            id,
            elements: Vec::new(),
        }
    }
    
    /// Returns the orientation of the layout.
    fn orientation(&self) -> Orientation {
        Orientation::Hori
    }
    
    /// gets the id of the layout.
    fn id(&self) -> PaneID {
        self.id
    }
}

