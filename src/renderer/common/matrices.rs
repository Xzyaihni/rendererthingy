use std::{
    f64,
    fmt::{self, Display},
    ops::{Sub, Mul}
};


#[derive(Debug, Clone, Copy)]
pub struct Mat4x4
{
    pub mat: [[f64; 4]; 4]
}

impl Mat4x4
{
    pub fn new() -> Self
    {
        Self{
            mat: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ]}
    }
}

impl Display for Mat4x4
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "[{} {} {} {}]\n[{} {} {} {}]\n[{} {} {} {}]\n[{} {} {} {}]",
            self.mat[0][0], self.mat[0][1], self.mat[0][2], self.mat[0][3],
            self.mat[1][0], self.mat[1][1], self.mat[1][2], self.mat[1][3],
            self.mat[2][0], self.mat[2][1], self.mat[2][2], self.mat[2][3],
            self.mat[3][0], self.mat[3][1], self.mat[3][2], self.mat[3][3],
        )
    }
}

impl Mul<[f64; 4]> for Mat4x4
{
    type Output = [f64; 4];

    fn mul(self, rhs: [f64; 4]) -> Self::Output
    {
        let mut out = [0.0; 4];
        for i in 0..4
        {
            out[i] = self.mat[i].into_iter().zip(rhs.into_iter()).map(|(m, v)| m * v).sum();
        }

        out
    }
}

impl Mul for Mat4x4
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output
    {
        let mut out = Self{mat: [[0.0; 4]; 4]};

        for y in 0..4
        {
            for x in 0..4
            {
                let mut s = 0.0;
                for i in 0..4
                {
                    s += self.mat[y][i] * rhs.mat[i][x];
                }

                out.mat[y][x] = s;
            }
        }

        out
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mat3x3
{
    pub mat: [[f64; 3]; 3]
}

#[allow(dead_code)]
impl Mat3x3
{
    pub fn new() -> Self
    {
        Self{
            mat: [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0]
        ]}
    }

    pub fn swap(&mut self, x0: usize, y0: usize, x1: usize, y1: usize)
    {
        (self.mat[y0][x0], self.mat[y1][x1]) = (self.mat[y1][x1], self.mat[y0][x0]);
    }

    pub fn transpose(mut self) -> Self
    {
        self.swap(1, 0, 0, 1);
        self.swap(2, 1, 1, 2);
        self.swap(2, 0, 0, 2);

        self
    }

    pub fn determinant(&self) -> f64
    {
        let mut out = 0.0;
        for i in 0..3
        {
            let a = if i == 0 {1} else {0};
            let b = if i == 2 {1} else {2};

            let current = self.mat[0][i] * (
                self.mat[1][a] * self.mat[2][b] - self.mat[1][b] * self.mat[2][a]
            );

            if i == 1
            {
                out -= current;
            } else
            {
                out += current;
            }
        }

        out
    }

    pub fn adjoint_scaled(self, scale: f64) -> Self
    {
        let mut out = Self{mat: [[0.0; 3]; 3]};

        for x in 0..3
        {
            for y in 0..3
            {
                let a = if x == 0 {1} else {0};
                let b = if x == 2 {1} else {2};

                let c;
                let d;

                if x == 1
                {
                    d = (y + 1) % 3;
                    c = (y + 2) % 3;
                } else
                {
                    c = (y + 1) % 3;
                    d = (y + 2) % 3;
                }

                let value = self.mat[a][c] * self.mat[b][d] - self.mat[a][d] * self.mat[b][c];

                out.mat[y][x] = scale * value;
            }
        }

        out
    }

    pub fn inverse(self) -> Self
    {
        let inv_determinant = 1.0 / self.determinant();

        self.adjoint_scaled(inv_determinant)
    }
}

impl Mul<[f64; 3]> for Mat3x3
{
    type Output = [f64; 3];

    fn mul(self, rhs: [f64; 3]) -> Self::Output
    {
        let mut out = [0.0; 3];
        for i in 0..3
        {
            out[i] = self.mat[i].into_iter().zip(rhs.into_iter()).map(|(m, v)| m * v).sum();
        }

        out
    }
}

impl Sub for Mat3x3
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output
    {
        let mut out = Self{mat: [[0.0; 3]; 3]};

        for y in 0..3
        {
            for x in 0..3
            {
                out.mat[y][x] = self.mat[y][x] - other.mat[y][x];
            }
        }

        out
    }
}

impl From<Mat4x4> for Mat3x3
{
    fn from(value: Mat4x4) -> Self
    {
        Self{
            mat: [
            [value.mat[0][0], value.mat[0][1], value.mat[0][2]],
            [value.mat[1][0], value.mat[1][1], value.mat[1][2]],
            [value.mat[2][0], value.mat[2][1], value.mat[2][2]]
        ]}
    }
}