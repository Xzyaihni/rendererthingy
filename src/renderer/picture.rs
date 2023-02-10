use std::{
    io::Write,
    fs::File
};

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
        let mut file = File::create(&self.filename).expect("couldnt create file");

        file.write(b"P6\n").unwrap();

        let size_text = size.0.to_string() + " " + &size.1.to_string() + "\n";
        file.write(size_text.as_bytes()).unwrap();

        file.write(b"255\n").unwrap();

        for color in colors
        {
            let color_byte = |color|
            {
                (color * 256.0) as u8
            };

            let bytes = [color_byte(color.r), color_byte(color.g), color_byte(color.b)];
            file.write(&bytes).unwrap();
        }

        file.flush().unwrap();
    }
}