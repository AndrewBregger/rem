use super::render::{Glyph, GlyphCache};
use super::font::{GlyphKey};

/// Represents a cursor.
pub struct Cursor;

/// An editable section of the screen.
pub struct Pane {
    /// Size of the pane in number of cells.
    size: (i32, i32),
    /// The location of the top left of the pane.
    loc: (f32, f32),
    /// The cursor of this pane
    cursor: Cursor,
    /// is the pane active.
    active: bool,
    // represents the editable document (not implemented)
    // document: Document
    /// all lines of the document
    lines: Vec<Line>,
    // add gutter info, maybe this should be in the line?
}

#[derive(Debug, Clone)]
pub struct Line;
