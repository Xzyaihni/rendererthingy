use image::{Rgb, ImageBuffer};

use crate::renderer::common::Color;
use crate::renderer::normal_drawable::DrawableDisplay;


pub struct Picture
{
    filename: String
}

#[allow(dead_code)]
impl Picture
{
    pub fn new(filename: String) -> Self
    {
        Picture{filename}
    }
}

#[allow(dead_code)]
impl DrawableDisplay for Picture
{
    fn prepare(&mut self, _: (usize, usize)) {}
    fn display(&self, size: (usize, usize), colors: &[Color])
    {
        let mut image = ImageBuffer::new(size.0 as u32, size.1 as u32);

        for (pixel, color) in image.pixels_mut().zip(colors.iter())
        {
            let convert = |color| (color * 255.0) as u8;

            *pixel = Rgb([convert(color.r), convert(color.g), convert(color.b)]);
        }

        image.save(self.filename.clone()).unwrap();
    }
}