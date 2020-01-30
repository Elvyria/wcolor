use std::str::FromStr;
use std::io::{Error, ErrorKind};

pub enum ColorFormat {
    HEX,
    RGB,
}

impl FromStr for ColorFormat {
    type Err = Error;

    fn from_str(format: &str) -> Result<Self, Self::Err> {
        match format.to_uppercase().as_ref() {
            "HEX" => Ok(ColorFormat::HEX),
            "RGB" => Ok(ColorFormat::RGB),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Color format is not supported")),
        }
    }
}

pub fn to_rgb(color: u32) -> (u8, u8, u8) {
    let r = (color >> 16) as u8;
    let g = (color >> 8) as u8;
    let b = color as u8;
    (r, g, b)
}
