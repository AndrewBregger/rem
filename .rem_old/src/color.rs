/// A color reprsentation in standard 255 different values for each channel.

use std::str::FromStr;

pub struct Color {
    channels: [u8; 3]
}


impl FromStr for Color {
    type Err = std::num::ParseIntError;

    fn from_str(hex_string: &str) -> Result<Self, Self::Err> {
        assert!(hex_string.chars().nth(0) == Some('#'));

        let r : u8 = u8::from_str_radix(&hex_string[1..3], 16)?;
        let g : u8 = u8::from_str_radix(&hex_string[3..5], 16)?;
        let b : u8 = u8::from_str_radix(&hex_string[5..7], 16)?;

        Ok(Self::channels(r, g, b))
    }
}

impl Color {
    /// create a color from opengl color values [0 .. 1.0]
    /// If a give value is greater than 1 or lesss than 0 it will be rounded to the closes valid
    /// value.
    pub fn from_ogl_color(r: f32, g: f32, b: f32) -> Self {
        
        // copy of clamp: https://doc.rust-lang.org/src/std/f32.rs.html#986-992
        fn clamp(mut x: f32, min: f32, max: f32) -> f32 {
            assert!(min <= max);
            if x < min { x = min; }
            if x > max { x = max; }
            x
        }

        let r = clamp(r, 0.0, 1.0) * 255.0;
        let g = clamp(g, 0.0, 1.0) * 255.0;
        let b = clamp(b, 0.0, 1.0) * 255.0;
        
        Self::channels(r as u8, g as u8, b as u8)
    }

    pub fn channels(r: u8, g: u8, b: u8) -> Self {
        Self {
            channels: [r, g, b]
        }
    }

    pub fn r(&self) -> u8 {
        self.channels[0]
    }

    pub fn b(&self) -> u8 {
        self.channels[1]
    }

    pub fn g(&self) -> u8  {
        self.channels[2]
    }

    pub fn gl_r(&self) -> f32 {
        self.channels[0] as f32 / 255.0
    }

    pub fn gl_b(&self) -> f32 {
        self.channels[1] as f32 / 255.0
    }

    pub fn gl_g(&self) -> f32  {
        self.channels[2] as f32 / 255.0
    }
        
    
    pub fn white() -> Self {
        Self::from_str("#ffffff").unwrap()
    }

    pub fn silver() -> Self {
        Self::from_str("#c0c0c0").unwrap()
    }

    pub fn gray() -> Self {
        Self::from_str("#808080").unwrap()
    }

    pub fn black() -> Self {
        Self::from_str("#000000").unwrap()
    }

    pub fn red() -> Self {
        Self::from_str("#ff0000").unwrap()
    }

    pub fn maroon() -> Self {
        Self::from_str("#800000").unwrap()
    }

    pub fn yellow() -> Self {
        Self::from_str("#ffff00").unwrap()
    }

    pub fn olive() -> Self {
        Self::from_str("#808000").unwrap()
    }

    pub fn lime() -> Self {
        Self::from_str("#00ff00").unwrap()
    }

    pub fn green() -> Self {
        Self::from_str("#008000").unwrap()
    }

    pub fn aqua() -> Self {
        Self::from_str("#00ffff").unwrap()
    }

    pub fn teal() -> Self {
        Self::from_str("#008080").unwrap()
    }

    pub fn blue() -> Self {
        Self::from_str("#0000ff").unwrap()
    }

    pub fn navy() -> Self {
        Self::from_str("#000080").unwrap()
    }

    pub fn fuchsia() -> Self {
        Self::from_str("#ff00ff").unwrap()
    }

    pub fn purple() -> Self {
        Self::from_str("#800080").unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_str_test() {
        let red = Color::red();
        assert!(red.r() == 255);
    }
}
