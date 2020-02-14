use winapi::shared::d3d9types::D3DCOLORVALUE;

pub struct Color(pub u32);

impl Color {
    pub fn to_rgb(self) -> (u8, u8, u8)  {
        let r = (self.0 >> 16) as u8;
        let g = (self.0 >> 8) as u8;
        let b = self.0 as u8;
        (r, g, b)
    }
}

impl From<Color> for D3DCOLORVALUE {
    fn from(color: Color) -> Self {
        let rgb = color.to_rgb();
        let r = (rgb.0 as f32) / 255.0;
        let g = (rgb.1 as f32) / 255.0;
        let b = (rgb.2 as f32) / 255.0;

        D3DCOLORVALUE { r, g, b, a: 1.0 }
    }
}

impl From<D3DCOLORVALUE> for Color {
    fn from(color: D3DCOLORVALUE) -> Self {
        let r = (color.r * 255.0) as u32;
        let g = (color.g * 255.0) as u32;
        let b = (color.b * 255.0) as u32;

        Color((b << 16) | (g << 8) | r)
    }
}
