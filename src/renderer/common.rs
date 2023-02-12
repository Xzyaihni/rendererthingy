use std::{
    f64,
    ops::{Sub, Mul, Neg}
};

pub use texture::*;
pub mod texture;

pub use matrices::*;
pub mod matrices;


#[derive(Debug, Clone, Copy)]
pub struct Color
{
    pub r: f64,
    pub g: f64,
    pub b: f64
}

impl Color
{
    pub fn new(r: f64, g: f64, b: f64) -> Self
    {
        Color{r, g, b}
    }

    pub fn lerp(&self, other: &Color, a: f64) -> Self
    {
        Color{
            r: Self::lerp_single(self.r, other.r, a),
            g: Self::lerp_single(self.g, other.g, a),
            b: Self::lerp_single(self.b, other.b, a)
        }
    }

    fn lerp_single(p0: f64, p1: f64, a: f64) -> f64
    {
        p0 * (1.0 - a) + p1 * a
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point2D<T=f64>
{
    pub x: T,
    pub y: T
}

#[derive(Debug, Clone, Copy)]
pub struct Point3D
{
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl Point3D
{
    pub fn normalized(self) -> Self
    {
        let magnitude = (self.x*self.x + self.y*self.y + self.z*self.z).sqrt();

        let x = self.x / magnitude;
        let y = self.y / magnitude;
        let z = self.z / magnitude;

        Self{x, y, z}
    }

    pub fn dot(self, other: Self) -> f64
    {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self
    {
        Self{
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x
        }
    }

    pub fn reflect(self, normal: Point3D) -> Self
    {
        self - (normal * 2.0 * normal.dot(self))
    }
}

impl Mul<f64> for Point3D
{
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output
    {
        Self{
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs
        }
    }
}

impl Sub for Point3D
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output
    {
        Self{
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z
        }
    }
}

impl Neg for Point3D
{
    type Output = Self;

    fn neg(self) -> Self::Output
    {
        Self{
            x: -self.x,
            y: -self.y,
            z: -self.z
        }
    }
}

pub enum ShaderValue
{
    Depth = 0,
    PositionX,
    PositionY,
    PositionZ,
    NormalX,
    NormalY,
    NormalZ,
    UvX,
    UvY,
    LAST
}

#[derive(Debug)]
pub struct IValue
{
    pub lower: f64,
    pub upper: f64
}

impl IValue
{
    fn reversed(self) -> Self
    {
        IValue{lower: self.upper, upper: self.lower}
    }

    fn interpolator(&self, amount: usize) -> IValueIter
    {
        IValueIter{start: self.lower, step: (self.upper - self.lower) / amount as f64}
    }
}

#[derive(Debug, Clone)]
struct IValueIter
{
    start: f64,
    step: f64
}

impl Iterator for IValueIter
{
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item>
    {
        let value = self.start;
        self.start += self.step;

        Some(value)
    }
}

pub const VALUES_AMOUNT: usize = ShaderValue::LAST as usize;

pub type ValuesType = [IValue; VALUES_AMOUNT];

pub fn combine_interpolated(p0: Interpolated, p1: Interpolated) -> ValuesType
{
    p0.iter().zip(p1.iter()).map(|(lower, upper)|
    {
        IValue{lower: *lower, upper: *upper}
    }).collect::<Vec<IValue>>().try_into().expect("same amount")
}

type ValuesTypeIter = [IValueIter; VALUES_AMOUNT];

pub type Interpolated = [f64; VALUES_AMOUNT];
pub const INTERPOLATED_ZEROS: Interpolated = [0.0; VALUES_AMOUNT];

#[derive(Debug, Clone, Copy)]
pub struct Point<T=f64>
{
    pub x: T,
    pub y: T,
    pub interpolated: Interpolated
}

impl<T> Point<T>
{
    pub fn get(&self, value: ShaderValue) -> f64
    {
        self.interpolated[value as usize]
    }
}

//theyre all called the same name and nobody cares
pub struct Interpolator
{
    values: ValuesType
}

impl Interpolator
{
    pub fn new(values: ValuesType) -> Self
    {
        Interpolator{values}
    }

    pub fn new_reversed(values: ValuesType) -> Self
    {
        Self::new(values.into_iter().map(|value| value.reversed())
            .collect::<Vec<IValue>>().try_into().expect("same amount of values just reversed"))
    }

    pub fn interpolator(&self, amount: usize) -> InterpolaterIter
    {
        InterpolaterIter{
            values: self.values.iter().map(|value| value.interpolator(amount + 1))
                .collect::<Vec<IValueIter>>().try_into().expect("it contains same amount")
        }
    }
}

pub struct InterpolaterIter
{
    values: ValuesTypeIter
}

impl Iterator for InterpolaterIter
{
    type Item = Interpolated;

    fn next(&mut self) -> Option<Self::Item>
    {
        Some(
            self.values.iter_mut().map(|value| value.next().expect("infinite iterator"))
                .collect::<Vec<f64>>().try_into().expect("same amount")
        )
    }
}

#[derive(Debug, Clone)]
pub struct FaceShader<'a>
{
    pub color: Color,
    pub lights: &'a [Light],
    pub texture: Option<&'a Texture>
}

#[derive(Debug, Clone)]
pub struct Light
{
    pub position: Point3D,
    //pub color: Color, no colored lights >:(
    pub intensity: f64
}

#[derive(Debug, Clone)]
pub struct PixelInfo<'a>
{
    pub shader: Option<&'a FaceShader<'a>>,
    pub interpolated: Interpolated
}

impl<'a> PixelInfo<'a>
{
    pub fn new(interpolated: Interpolated) -> Self
    {
        Self{shader: None, interpolated}
    }

    pub fn set(&mut self, shader: &'a FaceShader, interpolated: Interpolated)
    {
        self.shader = Some(shader);
        self.interpolated = interpolated;
    }

    pub fn get(&self, value: ShaderValue) -> f64
    {
        self.interpolated[value as usize]
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn close_enough(a: f64, b: f64) -> bool
    {
        dbg!(a, b);
        (a - b).abs() < 0.001
    }

    #[test]
    fn interpolation()
    {
        let mut values0 = INTERPOLATED_ZEROS;
        for value in values0.iter_mut()
        {
            *value = 0.3;
        }

        let mut values1 = INTERPOLATED_ZEROS;
        for value in values1.iter_mut()
        {
            *value = 0.9;
        }

        let interpolator = Interpolator::new(combine_interpolated(values0, values1));

        let amount = 8;
        let mut interpolator = interpolator.interpolator(amount);

        for a in 0..amount
        {
            let values = interpolator.next().unwrap();

            let iter = values0.iter().zip(values1.iter()).zip(values.iter());
            for ((value0, value1), test_value) in iter
            {
                let a = a as f64 / (amount + 1) as f64;
                let correct = value0 * (1.0 - a) + value1 * a;

                assert!(close_enough(correct, *test_value));
            }
        }
    }
}