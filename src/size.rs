use std::convert::{From, Into};

#[derive(Debug, Clone, Copy)]
pub struct Size<T> {
    pub x: T,
    pub y: T,
}

impl<T> Size<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T, D> From<(D, D)> for Size<T>
where
    D: Into<T>,
{
    fn from(other: (D, D)) -> Self {
        Self::new(other.0.into(), other.1.into())
    }
}

impl<T, D> Into<(D, D)> for Size<T>
where
    D: From<T>,
{
    fn into(self) -> (D, D) {
        (self.x.into(), self.y.into())
    }
}
