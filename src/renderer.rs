use std::{
    ops::{Deref, DerefMut}
};

use common::{
    Color,
    Point,
    Point2D,
    Point3D,
    Mat3x3,
    Mat4x4,
    Light,
    FaceShader
};

use normal_drawable::drawable::Drawable;

use model::Model;

pub mod common;

pub mod normal_drawable;

pub mod model;

pub mod picture;
pub mod console_screen;


pub struct Transform
{
    position: (f64, f64, f64),
    scale: (f64, f64, f64),
    rotation: f64,
    rotation_axis: (f64, f64, f64),
    combined: Mat4x4
}

impl Transform
{
    pub fn new(
        position: (f64, f64, f64),
        scale: (f64, f64, f64),
        rotation: f64,
        rotation_axis: (f64, f64, f64)
    ) -> Self
    {
        let mut out = Transform{position, scale, rotation, rotation_axis, combined: Mat4x4::new()};

        out.combine();

        out
    }

    pub fn combine(&mut self)
    {
        let scale_mat = Mat4x4{mat: [
            [self.scale.0, 0.0, 0.0, 0.0],
            [0.0, self.scale.1, 0.0, 0.0],
            [0.0, 0.0, self.scale.2, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ]};

        let translate_mat = Mat4x4{mat: [
            [1.0, 0.0, 0.0, self.position.0],
            [0.0, 1.0, 0.0, self.position.1],
            [0.0, 0.0, 1.0, self.position.2],
            [0.0, 0.0, 0.0, 1.0]
        ]};

        let axis_magnitude = (self.rotation_axis.0 * self.rotation_axis.0
            + self.rotation_axis.1 * self.rotation_axis.1
            + self.rotation_axis.2 * self.rotation_axis.2).sqrt();

        //normalize the axis
        let axis = (self.rotation_axis.0 / axis_magnitude,
            self.rotation_axis.1 / axis_magnitude,
            self.rotation_axis.2 / axis_magnitude);

        let ca = self.rotation.cos();
        let nca = 1.0 - ca;

        let sa = self.rotation.sin();

        let rotate_mat = Mat4x4{mat: [
            [ca+axis.0*axis.0*nca, axis.0*axis.1*nca-axis.2*sa, axis.0*axis.2*nca+axis.1*sa, 0.0],
            [axis.1*axis.0*nca+axis.2*sa, ca+axis.1*axis.1*nca, axis.1*axis.2*nca-axis.0*sa, 0.0],
            [axis.2*axis.0*nca-axis.1*sa, axis.2*axis.1*nca+axis.0*sa, ca+axis.2*axis.2*nca, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ]};

        self.combined = translate_mat * rotate_mat * scale_mat;
    }

    pub fn set_rotation(&mut self, rotation: f64)
    {
        self.rotation = rotation;

        self.combine();
    }

    pub fn rotation(&self) -> f64
    {
        self.rotation
    }

    pub fn matrix(&self) -> Mat4x4
    {
        self.combined
    }
}

pub struct Camera
{
    near: f64,
    far: f64,
    fov: f64,
    aspect: f64,
    mat: Mat4x4
}

impl Camera
{
    pub fn new(near: f64, far: f64, fov: f64, aspect: f64) -> Self
    {
        let mut out = Camera{near, far, fov, aspect, mat: Mat4x4::new()};

        out.calculate_matrix();

        out
    }

    fn calculate_matrix(&mut self)
    {
        let th_fov = (self.fov / 2.0).tan();

        let a = -(self.far + self.near) / (self.far - self.near);
        let b = -(2.0 * self.far * self.near) / (self.far - self.near);

        self.mat = Mat4x4{mat: [
            [1.0 / (self.aspect * th_fov), 0.0, 0.0, 0.0],
            [0.0, 1.0 / th_fov, 0.0, 0.0],
            [0.0, 0.0, a, b],
            [0.0, 0.0, -1.0, 0.0]
        ]};
    }

    pub fn matrix(&self) -> Mat4x4
    {
        self.mat
    }
}

pub struct Object<'a>
{
    model: &'a Model,
    transform: Transform,
    camera: &'a Camera,
    lights: &'a [Light],
    points: Vec<Point3D>,
    world_points: Vec<Point3D>,
    normals: Vec<Point3D>,
    face_shaders: Vec<FaceShader<'a>>
}

impl<'a> Object<'a>
{
    pub fn new(
        model: &'a Model,
        transform: Transform,
        camera: &'a Camera,
        lights: &'a [Light]
    ) -> Self
    {
        let mut out = Object{
            model,
            transform,
            camera,
            lights,
            points: Vec::new(),
            world_points: Vec::new(),
            normals: Vec::new(),
            face_shaders: Vec::new()
        };

        out.update_transform();

        out
    }

    fn backface(p0: Point3D, p1: Point3D, p2: Point3D) -> (bool, Point3D)
    {
        let normal = (p1 - p0).cross(p2 - p0);

        (p0.dot(normal) >= 0.0, normal)
    }

    pub fn draw<'d>(&'d self, drawable: &mut impl Drawable<'d>)
    where 'a: 'd
    {
        for t in 0..(self.model.indices.len()/3)
        {
            self.draw_triangle(drawable, t);
        }
    }

    fn draw_triangle<'d>(&'d self, drawable: &mut impl Drawable<'d>, start_index: usize)
    where 'a: 'd
    {
        let meta_index = |point_index| start_index * 3 + point_index;
        let index_at = |point_index| self.model.indices[meta_index(point_index)];
        let world_point = |point_index| self.world_points[index_at(point_index)];

        let world_points = [world_point(0), world_point(1), world_point(2)];

        let (is_backface, normal) =
            Self::backface(world_points[0], world_points[1], world_points[2]);

        if is_backface
        {
            return;
        }

        let point_at = |point_index|
        {
            let meta_index = meta_index(point_index);
            let index = index_at(point_index);

            let normal: Point3D = if !self.normals.is_empty()
            {
                self.normals[meta_index]
            } else
            {
                normal.normalized()
            };

            let uv: Point2D = if !self.model.uvs.is_empty()
            {
                self.model.uvs[meta_index]
            } else
            {
                Point2D{x: 0.0, y: 0.0}
            };

            let point: Point3D = self.points[index];
            let world_point: Point3D = world_points[point_index];

            let shader_values = [
                point.z,
                world_point.x, world_point.y, world_point.z,
                normal.x, normal.y, normal.z,
                uv.x, uv.y
            ];

            Point{
                x: point.x,
                y: point.y,
                interpolated: shader_values
            }
        };

        let shader = &self.face_shaders[start_index];

        drawable.triangle(point_at(0), point_at(1), point_at(2), shader);
    }

    pub fn update_transform(&mut self)
    {
        let transform_matrix = self.transform.matrix();
        let projection_matrix = self.camera.matrix();

        (self.points, self.world_points) = (0..(self.model.vertices.len()/3)).map(|index|
        {
            let point = [
                self.model.vertices[index * 3],
                self.model.vertices[index * 3 + 1],
                self.model.vertices[index * 3 + 2],
                1.0
            ];

            let world_point = transform_matrix * point;
            let transformed = projection_matrix * world_point;

            let point = Point3D{
                x: (transformed[0] / transformed[3] + 1.0) / 2.0,
                y: (transformed[1] / transformed[3] + 1.0) / 2.0,
                z: transformed[2] / transformed[3]
            };

            (point, Point3D{x: world_point[0], y: world_point[1], z: world_point[2]})
        }).unzip();


        let normal_matrix = Mat3x3::from(transform_matrix).transpose().inverse();
        self.normals = self.model.normals.iter().map(|normal|
        {
            let normal = normal_matrix * [normal.x, normal.y, normal.z];
            Point3D{x: normal[0], y: normal[1], z: normal[2]}
        }).collect();

        self.face_shaders = self.model.material_indices.iter().map(|material_index|
        {
            let lights = self.lights;

            if let Some(index) = material_index
            {
                let material = &self.model.materials[*index];

                let color = material.diffuse_color.unwrap_or(Color::new(0.5, 0.5, 0.5));
                let texture = material.diffuse_texture.as_ref();

                FaceShader{color, lights, texture}
            } else
            {
                FaceShader{color: Color::new(0.5, 0.5, 0.5), lights, texture: None}
            }
        }).collect();
    }
}

impl<'a> Deref for Object<'a>
{
    type Target = Transform;

    fn deref(&self) -> &Self::Target
    {
        &self.transform
    }
}

impl<'a> DerefMut for Object<'a>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.transform
    }
}