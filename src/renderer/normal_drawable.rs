use drawable::Drawable;

use crate::renderer::common::{
    Point,
    Color,
    ShaderValue,
    FaceShader,
    PixelInfo,
    INTERPOLATED_ZEROS
};

pub mod drawable;
mod color_shader;


pub trait DrawableDisplay
{
    fn prepare(&mut self, size: (usize, usize));
    fn display(&self, size: (usize, usize), colors: &[Color]);
}

pub trait DrawSurface<'a>: Drawable<'a>
{
    fn display(self);
}

pub trait DrawableNormal
{
    type SurfaceType<'a>: DrawSurface<'a> where Self: 'a;
    fn surface<'b>(&'b mut self) -> Self::SurfaceType<'b>;
}

pub struct NormalDrawable<T>
{
    size: (usize, usize),
    display: T
}

impl<T> NormalDrawable<T>
{
    pub fn new(size: (usize, usize), display: T) -> Self
    {
        Self{
            size,
            display
        }
    }
}

impl<T: DrawableDisplay> DrawableNormal for &mut NormalDrawable<T>
{
    type SurfaceType<'a> = NormalSurface<'a, T> where Self: 'a;

    fn surface<'b>(&'b mut self) -> Self::SurfaceType<'b>
    {
        let total_size = self.size.0 * self.size.1;

        NormalSurface{
            size: self.size,
            display: &mut self.display,
            depths: vec![1.0; total_size],
            colors: vec![Color::new(0.0, 0.0, 0.0); total_size]
        }
    }
}

pub struct NormalSurface<'a, T>
{
    size: (usize, usize),
    depths: Vec<f64>,
    colors: Vec<Color>,
    display: &'a mut T
}

impl<'a, T: DrawableDisplay> DrawSurface<'a> for NormalSurface<'a, T>
{
    fn display(self)
    {
        self.display.prepare(self.size);
        self.display.display(self.size, &self.colors);
    }
}

#[allow(dead_code)]
impl<'a, T> Drawable<'a> for NormalSurface<'a, T>
{
    fn set_pixel_data(&mut self, point: Point<usize>, shader: &'a FaceShader)
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
            let pixel_info = PixelInfo{interpolated: point.interpolated, shader: Some(shader)};

            self.colors[index] = color_shader::execute(&pixel_info);
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

pub struct DeferredDrawable<T>
{
    size: (usize, usize),
    display: T
}

impl<T> DeferredDrawable<T>
{
    pub fn new(size: (usize, usize), display: T) -> Self
    {
        Self{
            size,
            display
        }
    }
}

pub struct DeferredSurface<'a, T>
{
    size: (usize, usize),
    pixels: Vec<PixelInfo<'a>>,
    display: &'a mut T
}

impl<T: DrawableDisplay> DrawableNormal for &mut DeferredDrawable<T>
{
    type SurfaceType<'a> = DeferredSurface<'a, T> where Self: 'a;

    fn surface<'b>(&'b mut self) -> Self::SurfaceType<'b>
    {
        let total_size = self.size.0 * self.size.1;

        let mut empty = INTERPOLATED_ZEROS;
        empty[ShaderValue::Depth as usize] = 1.0;

        DeferredSurface{
            size: self.size,
            display: &mut self.display,
            pixels: vec![PixelInfo::new(empty); total_size]
        }
    }
}

impl<'a, T: DrawableDisplay> DrawSurface<'a> for DeferredSurface<'a, T>
{
    fn display(self)
    {
        self.display.prepare(self.size);

        let colors = self.pixels.iter().map(color_shader::execute).collect::<Vec<Color>>();
        self.display.display(self.size, &colors);
    }
}

#[allow(dead_code)]
impl<'a, T> Drawable<'a> for DeferredSurface<'a, T>
{
    fn set_pixel_data(&mut self, point: Point<usize>, shader: &'a FaceShader)
    {
        let index = (self.size.1 - point.y - 1) * self.size.0 + point.x;

        let depth = point.get(ShaderValue::Depth);
        if depth < -1.0 || depth > 1.0
            || point.x >= self.size.0
            || point.y >= self.size.1
        {
            return;
        }

        let pixel_depth = self.pixels[index].get(ShaderValue::Depth);
        if depth < pixel_depth
        {
            self.pixels[index].set(shader, point.interpolated);
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
