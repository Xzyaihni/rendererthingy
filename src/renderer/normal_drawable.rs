use drawable::Drawable;

use crate::renderer::common::{
    Point,
    Color,
    ShaderValue,
    FaceShader
};

pub mod drawable;
mod color_shader;


pub trait DrawableDisplay
{
    fn prepare(&mut self, size: (usize, usize));
    fn display(&self, size: (usize, usize), colors: &[Color]);
}

pub struct NormalDrawable<T>
{
    size: (usize, usize),
    colors: Vec<Color>,
    depths: Vec<f64>,
    display: T
}

impl<T: DrawableDisplay> NormalDrawable<T>
{
    pub fn new(size: (usize, usize), display: T) -> Self
    {
        let mut out = NormalDrawable{
            size,
            colors: Vec::new(),
            depths: Vec::new(),
            display
        };

        out.clear_color(Color::new(0.0, 0.0, 0.0));
        out.clear_depth();

        out
    }

    pub fn clear_color(&mut self, color: Color)
    {
        self.colors = vec![color; self.size.0 * self.size.1];
    }

    pub fn clear_depth(&mut self)
    {
        self.depths = vec![1.0; self.size.0 * self.size.1];
    }

    pub fn display(&mut self)
    {
        self.display.prepare(self.size);
        self.display.display(self.size, &self.colors);

        self.clear_color(Color::new(0.0, 0.0, 0.0));
        self.clear_depth();
    }
}

#[allow(dead_code)]
impl<T> Drawable for NormalDrawable<T>
{
    fn set_pixel_data(&mut self, point: Point<usize>, shader: &FaceShader)
    {
        let index = (self.size.1 - point.y - 1) * self.size.0 + point.x;

        let depth = point.get(ShaderValue::Depth);
        if depth < -1.0 || depth > 1.0
            || point.x >= self.size.0
            || point.y >= self.size.1
        {
            return;
        }

        let pixel_depth = self.depths[index];
        if depth < pixel_depth
        {
            self.colors[index] = color_shader::execute(point, shader);

            self.depths[index] = depth;
        }
    }

    fn to_local(&self, point: Point) -> Point<usize>
    {
        Point{
            x: ((point.x * self.size.0 as f64) as usize),
            y: ((point.y * self.size.1 as f64) as usize),
            interpolated: point.interpolated
        }
    }
}