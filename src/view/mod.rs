pub mod cells;
pub mod line_cache;
pub mod view_status;

use crate::size::Size;

pub use cells::*;
pub use line_cache::*;
pub use view_status::*;

use crate::render::framebuffer::{FrameBuffer, FrameSize};

/// A 'view' into a file. It allows the user to see the content of a file within a given range.
// Is it the right way to have this view take a reference to the buffer.
pub struct View<'a> {
    /// the the physical size of a view in pixels.
    size: Size<f32>,
    // The renderable area of this view.
    // cell: CellGrid,
    /// A cache of the previously views lines.
    line_cache: LineCache<'a>,
    /// 
    frame_cache: FrameBuffer,
}

