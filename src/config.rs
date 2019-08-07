use super::font::{FontDesc, FontSize};

pub struct Font {
    pub font: FontDesc,
    pub size: FontSize
}

pub struct Colors {
    bg: [f32; 3],
    fg: [f32; 3],
}
