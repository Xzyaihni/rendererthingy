use crate::renderer::common::{
    Point,
    FaceShader,
    combine_interpolated,
    Interpolator,
    Interpolated,
    INTERPOLATED_ZEROS
};


#[derive(Debug, Clone, Copy)]
pub struct PointDesc
{
    limit: usize,
    interpolated: Interpolated
}

#[derive(Debug, Clone)]
pub struct Limits
{
    lower: PointDesc,
    upper: PointDesc
}

pub trait Drawable<'a>
{
    fn set_pixel_data(&mut self, point: Point<usize>, shader: &'a FaceShader);
    fn to_local(&self, point: Point) -> Point<usize>;

    fn line(&mut self, p0: Point, p1: Point, shader: &'a FaceShader)
    {
        Self::line_points(self.to_local(p0), self.to_local(p1), |point|
        {
            self.set_pixel_data(point, shader)
        });
    }

    fn line_pixel(
        point: Point<usize>,
        y_begin: usize,
        y_points: &mut [Limits]
    )
    {
        if y_points.len() != 0
        {
            let limits = y_points.get_mut(point.y - y_begin).unwrap();

            let desc = PointDesc{limit: point.x, interpolated: point.interpolated};
            if limits.lower.limit > point.x
            {
                limits.lower = desc;
            }

            if limits.upper.limit < point.x
            {
                limits.upper = desc;
            }
        }
    }

    fn line_low_points(
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        interpolator: Interpolator,
        mut draw_function: impl FnMut(Point<usize>)
    )
    {
        let dx = (x1 - x0) as i32;
        let mut dy = y1 as i32 - y0 as i32;

        let sy: i32 = if dy < 0
        {
            dy = -dy;
            -1
        } else
        {
            1
        };

        let mut error = (2 * dy) - dx;
        let mut y = y0 as i32;

        let mut interpolator = interpolator.interpolator(dx as usize);
        for x in x0..=x1
        {
            draw_function(Point{
                x,
                y: y as usize,
                interpolated: interpolator.next().expect("infinite")
            });

            if error > 0
            {
                y += sy;
                error += 2 * (dy - dx);
            } else
            {
                error += 2 * dy;
            }
        }
    }

    fn line_high_points(
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        interpolator: Interpolator,
        mut draw_function: impl FnMut(Point<usize>)
    )
    {
        let mut dx = x1 as i32 - x0 as i32;
        let dy = (y1 - y0) as i32;

        let sx: i32 = if dx < 0
        {
            dx = -dx;
            -1
        } else
        {
            1
        };

        let mut error = (2 * dx) - dy;

        let mut x = x0 as i32;

        let mut interpolator = interpolator.interpolator(dy as usize);
        for y in y0..=y1
        {
            draw_function(Point{
                x: x as usize,
                y,
                interpolated: interpolator.next().expect("infinite")
            });

            if error > 0
            {
                x += sx;
                error += 2 * (dx - dy);
            } else
            {
                error += 2 * dx;
            }
        }
    }

    fn line_vertical(
        x: usize,
        y: usize,
        length: usize,
        interpolator: Interpolator,
        mut draw_function: impl FnMut(Point<usize>)
    )
    {
        let mut interpolator = interpolator.interpolator(length);
        for y in y..y+length
        {
            draw_function(Point{x, y, interpolated: interpolator.next().expect("infinite")});
        }
    }

    fn line_horizontal(
        y: usize,
        x: usize,
        length: usize,
        interpolator: Interpolator,
        mut draw_function: impl FnMut(Point<usize>)
    )
    {
        let mut interpolator = interpolator.interpolator(length);
        for x in x..x+length
        {
            draw_function(Point{x, y, interpolated: interpolator.next().expect("infinite")});
        }
    }

    fn line_points(
        p0: Point<usize>,
        p1: Point<usize>,
        draw_function: impl FnMut(Point<usize>)
    )
    {
        let values = combine_interpolated(p0.interpolated, p1.interpolated);
        let y_abs_diff = (p1.y as i32 - p0.y as i32).abs() as usize;
        let x_abs_diff = (p1.x as i32 - p0.x as i32).abs() as usize;

        if p0.x == p1.x
        {
            if p0.y > p1.y
            {
                Self::line_vertical(
                    p0.x, p1.y,
                    y_abs_diff+1,
                    Interpolator::new_reversed(values),
                    draw_function
                );
            } else
            {
                Self::line_vertical(
                    p0.x, p0.y,
                    y_abs_diff+1,
                    Interpolator::new(values),
                    draw_function
                );
            }
        } else if p0.y == p1.y
        {
            if p0.x > p1.x
            {
                Self::line_horizontal(
                    p0.y, p1.x,
                    x_abs_diff+1,
                    Interpolator::new_reversed(values),
                    draw_function
                );
            } else
            {
                Self::line_horizontal(
                    p0.y, p0.x,
                    x_abs_diff+1,
                    Interpolator::new(values),
                    draw_function
                );
            }
        } else if y_abs_diff < x_abs_diff
        {
            if p0.x > p1.x
            {
                Self::line_low_points(
                    p1.x, p1.y, p0.x, p0.y,
                    Interpolator::new_reversed(values),
                    draw_function
                );
            } else
            {
                Self::line_low_points(
                    p0.x, p0.y, p1.x, p1.y,
                    Interpolator::new(values),
                    draw_function
                );
            }
        } else
        {
            if p0.y > p1.y
            {
                Self::line_high_points(
                    p1.x, p1.y, p0.x, p0.y,
                    Interpolator::new_reversed(values),
                    draw_function
                );
            } else
            {
                Self::line_high_points(
                    p0.x, p0.y, p1.x, p1.y,
                    Interpolator::new(values),
                    draw_function
                );
            }
        }
    }

    fn triangle(
        &mut self,
        o0: Point,
        o1: Point,
        o2: Point,
        shader: &'a FaceShader
    )
    {
        let p0 = self.to_local(o0);
        let p1 = self.to_local(o1);
        let p2 = self.to_local(o2);

        let min_y = p0.y.min(p1.y.min(p2.y));
        let max_y = p0.y.max(p1.y.max(p2.y));

        let mut points_slice = vec![Limits{
            lower: PointDesc{limit: usize::MAX, interpolated: INTERPOLATED_ZEROS},
            upper: PointDesc{limit: 0, interpolated: INTERPOLATED_ZEROS}
        }; max_y - min_y + 1];

        let mut pixel_fn = |point| Self::line_pixel(point, min_y, &mut points_slice);

        Self::line_points(p0, p1, &mut pixel_fn);
        Self::line_points(p1, p2, &mut pixel_fn);
        Self::line_points(p2, p0, pixel_fn);

        for (index, limit) in points_slice.into_iter().enumerate()
        {
            let y = min_y + index;

            let line_length = limit.upper.limit - limit.lower.limit;
            let values = combine_interpolated(limit.lower.interpolated, limit.upper.interpolated);

            Self::line_horizontal(
                y, limit.lower.limit, line_length + 1,
                Interpolator::new(values),
                |point| {self.set_pixel_data(point, &shader)}
            );
        }
    }

    fn triangle_wireframe(
        &mut self,
        p0: Point,
        p1: Point,
        p2: Point,
        shader: &'a FaceShader
    )
    {
        self.line(p0, p1, &shader);
        self.line(p1, p2, &shader);
        self.line(p2, p0, &shader);
    }
}