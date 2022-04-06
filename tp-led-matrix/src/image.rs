use crate::gamma;


pub const RED: Color = Color {r: 0xff, g: 0x00, b: 0x00};
pub const BLUE: Color = Color {r: 0x00, g: 0x00, b: 0xff};
pub const GREEN: Color = Color {r: 0x00, g: 0xff, b: 0x00};


#[derive(Clone)]
#[derive(Copy)]
#[derive(Default)]
#[repr(C)]
/// represents an individual RGB pixel
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

#[repr(transparent)]
/// represents a whole 8×8 image made of pixels
pub struct Image(pub [Color; 64]);

impl core::ops::Mul<f32> for Color{
type Output = Self;
fn mul(self, rhs: f32) -> Self {
    let new_r = (self.r as f32)*rhs;
    let new_b = (self.b as f32)*rhs;
    let new_g = (self.g as f32)*rhs;
    assert!(new_r<u8::MAX as f32, "Result can't be stored in a color");
    assert!(new_b<u8::MAX as f32, "Result can't be stored in a color");
    assert!(new_g<u8::MAX as f32, "Result can't be stored in a color");
    return Color {r: new_r as u8, b: new_b as u8, g: new_g as u8};
}
}

impl core::ops::Div<f32> for Color {
type Output = Self;
fn div(self, rhs: f32) -> Self {
    return self*(1.0/rhs);
}
}

impl core::ops::Index<(usize, usize)> for Image{
type Output = Color;
fn index(&self, index: (usize, usize)) -> &Self::Output {
    return &self.0[8*index.0+index.1];
}
}

impl core::ops::IndexMut<(usize, usize)> for Image {
fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
    return &mut self.0[8*index.0+index.1];
}
}

impl Default for Image {
fn default() -> Self {
    Image::new_solid(Color::default())
}
}

impl AsRef<[u8; 192]> for Image {
fn as_ref(&self) -> &[u8; 192] {
    unsafe {return core::mem::transmute::<&Image, &[u8; 192]>(self)}
}
}

impl AsMut<[u8; 192]> for Image {
fn as_mut(&mut self) -> &mut [u8; 192] {
    unsafe {return core::mem::transmute::<&mut Image, &mut [u8; 192]>(self)}
}
}

impl Color {
    /// Applies gamma correction for a pixel.
    pub fn gamma_correct(&self) -> Self {
        return Color {r: gamma::gamma_correct(self.r), g: gamma::gamma_correct(self.g), b: gamma::gamma_correct(self.b)}
    }
}

impl Image {
    /// Creates a new image filled with one unique color.
    pub fn new_solid(color: Color) -> Self {
        return Image([color; 64])
    }

    /// Returns reference on a row in an image.
    pub fn row(&self, row: usize) -> &[Color] {
        assert!(row<8, "There's only 8 rows in the image");
        return &self.0[row*8..(row+1)*8];
    }

    /// Creates a new image filled with a gradient of colors.
    pub fn gradient(color: Color) -> Self {
        let mut new_image: Image = Default::default();
        let mut index = 0;
        for c in new_image.0 {
            let row = index/8;
            let col = index%8;
            new_image.0[index] = c/((1 + row * row + col) as f32);
            index+=1;
        }
        return new_image
    }
}
