use std::collections::HashMap;

use crate::color::Color;
use crate::editor_core::DocID;

#[derive(Debug)]
pub enum Error {
}

type Result<T> = ::std::result::Result<T, Error>;

/// A single token to be rendered.
/// It knows its value and display color.
pub struct Atom<'a> {
    /// the string this an atom of.
    value: &'a str,
    /// where this atom resides in the line.
    index: usize, // this is to allow for quick look ups.
    /// the render color of this text.
    color: Color
}

impl<'a> Atom<'a> {
}

/// A line that was has previously been rendered.
pub struct Line<'a> {
    line_number: u32, 
    /// A sorted list of atoms. The atoms are sorted by their start index.
    atoms: Vec<Atom<'a>>
}

impl<'a> Line<'a> {
    pub fn emtpy() -> Self {
        Self {
            line_number: 0,
            atoms: Vec::new(),
        }
    }
    
    /// inserts the atom into the line at the appropriate location in the vector.
    pub fn insert(&mut self, atom: Atom<'a>) -> Result<()> {
    }
    
    pub fn remove(&mut self, index: usize) -> Result<()> {
    }
}

/// A cache of renderable lines.
/// Theses lines have been processed before and have not changed since the last time they
/// were viewed.
pub struct LineCache<'a> {
    /// the lines of a view that were rendered indexed by their line.
    lines: HashMap<u32, Line<'a>>,
    /// The document these line are associated with.
    document: DocID,
}

impl<'a> LineCache<'a> {
    pub fn new(document: DocID) -> Self {
        Self {
            lines: HashMap::new(),
            document
        }
    }
}
