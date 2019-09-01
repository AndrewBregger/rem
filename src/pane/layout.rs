
use super::{Pane, PaneID};
use size::Size;

pub enum Orientation {
    Vert,
    Hori,
}

pub trait Layout {
    type Element;
    
    /// Creates a new layout
    fn new(size: Size<f32>) -> <Self as Layout>::Element;
    
    /// Returns the given size.
    fn size(&self) -> Size<f32>;
    
    /// Returns the orientation of the layout.
    fn orientation(&self) -> Orientation;
    
    /// gets the id of the layout.
    fn id(&self) -> PaneID;
}

pub struct VerticalLayout {
    id: PaneID,
    elements: Vec<Pane>,
}

pub struct HorizontalLayout {
    id: PaneID,
    elements: Vec<Pane>,
}
