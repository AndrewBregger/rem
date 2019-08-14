use glm::Vec2;
use super::font::{FontDesc, FontSize};
use std::default::Default;
use crate::pane;

#[derive(Debug, Clone)]
pub struct Font {
    pub font: FontDesc,
    pub size: FontSize,
    pub offset: Vec2,
}

#[derive(Debug, Clone)]
pub struct Colors {
    pub bg: [f32; 3],
    pub fg: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct Window {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub tab_width: u8,
    // smart tabbing?
}

#[derive(Debug, Clone)]
pub struct Atlas {
    pub size: f32,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    /// the cursor mode in insert mode
    pub insert: pane::CursorMode,
    /// the cursor mode in normal mode
    pub normal: pane::CursorMode,
}
// ColorScheme? Theme? Theses could be file names and the settings struct handles how they interact

#[derive(Debug, Clone)]
pub struct Config {
    pub font: Font,
    pub colors: Colors,
    pub window: Window,
    pub tabs: Tab,
    pub atlas: Atlas,
    pub cursor: Cursor,
}


// implement config file to be loaded.

impl Default for Config {
    fn default() -> Self {
        Self {
            font: Font {
                font: FontDesc {
                    name: "DroidSansMono".to_string(),
                    path: std::path::Path::new("dev/DroidSansMono.ttf").to_path_buf(),
                },
                size: FontSize { pixel_size: 14.0 },
                offset: Vec2::new(0.0, 0.0)
            },
            colors: Colors {
                bg: [0.0, 0.0, 0.0],
                fg: [0.0, 0.0, 0.0],
            },
            window: Window {
                width: 1024f32,
                height: 864f32,
            },
            tabs: Tab {
                tab_width: 2,
            },
            atlas: Atlas {
                size: 1024f32,
            },
            cursor: Cursor {
                insert: pane::CursorMode::Line,
                normal: pane::CursorMode::Box,
            }
        }
    }
}
