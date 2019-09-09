use std::collections::HashMap;
use crate::color::Color;

/// A single token to be rendered.
/// It knows its value and display color.
pub struct Atom<'a> {
    value: &'a str,
    color: Color
}

/// A line that was has previously been rendered.
pub struct Line<'a> {
    line_number: u32, 
    atoms: Vec<Atom<'a>>
}

pub struct LineCache<'a> {
    /// the lines of a view that were rendered indexed by their line.
    lines: HashMap<u32, Line<'a>>,
}
