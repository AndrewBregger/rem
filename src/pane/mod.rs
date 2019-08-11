use super::render::{Glyph, GlyphCache};
use super::font::{GlyphKey};
use std::sync::atomic::{AtomicU32, Ordering::SeqCst};
use super::editor_core;
use std::cmp::Eq;
use crate::font::{FontKey, FontSize};
use crate::config;

#[derive(Debug, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height
        }
    }
}

// #[derive(Debug, Copy, Clone)]
pub type Loc = glm::Vec2;

// type Result<T> = ::std::result::Result<T, Error>;

/// Represents a cursor.
#[derive(Debug)]
pub struct Cursor;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PaneID(u32);

impl PaneID {
    fn next() -> Self {
        static token: AtomicU32 = AtomicU32::new(0);

        Self { 0: token.fetch_add(1, SeqCst) }
    }
}


/// An editable section of the screen.
#[derive(Debug)]
pub struct Pane {
    /// Size of the pane in number of cells.
    size: Size,
    /// The location of the top left of the pane.
    loc: Loc,
    /// The cursor of this pane
    cursor: Cursor,
    /// is the pane active.
    active: bool,
    /// The line number at the top of the pane.
    first_line: usize,
    /// flag to determine if a redraw is needed
    dirty: bool,
    /// Identification of Pane
    id: PaneID,
    /// font of the pane
    font: FontKey,
    /// font size of this pane
    font_size: FontSize,
}

#[derive(Debug, Clone)]
pub struct Line;

impl Pane {
    pub fn new(size: Size, loc: Loc, font: FontKey, font_size: FontSize) -> Self {
        Self {
            size,
            loc,
            cursor: Cursor,
            active: true,
            first_line: 0 as usize,
            dirty: true,
            id: PaneID::next(),
            font,
            font_size,
        }
    }

    pub fn set_font(&mut self, font: FontKey) {
        self.font = font
    }

    pub fn increase_font_size(&mut self, inc: f32) {
        self.font_size.pixel_size += inc;
    }

    pub fn decrease_font_size(&mut self, inc: f32) {
        self.font_size.pixel_size -= inc;
    }
}