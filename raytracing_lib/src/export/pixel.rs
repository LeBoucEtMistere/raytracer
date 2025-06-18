use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct RGBPixel(pub u8, pub u8, pub u8);

impl Display for RGBPixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.0, self.1, self.2)
    }
}
