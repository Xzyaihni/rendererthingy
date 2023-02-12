use std::path::Path;

use image::error::ImageError;

use super::{Point2D, Color};


#[derive(Debug, Clone)]
pub struct Texture
{
    size: (usize, usize),
    colors: Vec<Color>
}

#[allow(dead_code)]
impl Texture
{
    pub fn new(size: (usize, usize), colors: Vec<Color>) -> Self
    {
        Self{size, colors}
    }

    pub fn load(filename: &Path) -> Result<Self, ImageError>
    {
        let image = image::open(filename)?;

        let size = (image.width() as usize, image.height() as usize);

        let colors = image.into_rgb32f().pixels().map(|pixel|
        {
            Color::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64)
        }).collect::<Vec<Color>>();

        Ok(Self{size, colors})
    }

    pub fn pixel(&self, position: Point2D) -> Color
    {
        self.pixel_local(self.to_local(position))
    }

    fn pixel_local(&self, position: Point2D<usize>) -> Color
    {
        self.colors[((self.size.1 - position.y - 1) * self.size.0) + position.x]
    }

    fn to_local(&self, position: Point2D) -> Point2D<usize>
    {
        Point2D{
            x: (((position.x * self.size.0 as f64) as i32).max(0) as usize).min(self.size.0),
            y: (((position.y * self.size.1 as f64) as i32).max(0) as usize).min(self.size.1)
        }
    }
}