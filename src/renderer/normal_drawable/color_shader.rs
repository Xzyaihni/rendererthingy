use crate::renderer::common::{
    Point,
    Point3D,
    Color,
    ShaderValue,
    FaceShader,
    PixelInfo
};


pub fn execute(pixel: &PixelInfo) -> Color
{
    if let Some(shader) = pixel.shader
    {
        let ambient = 0.2;
        let shininess = 32;

        let ambient_color = Color::new(0.0, 0.0, 0.0).lerp(&shader.color, ambient);

        let world_point = Point3D{
            x: pixel.get(ShaderValue::PositionX),
            y: pixel.get(ShaderValue::PositionY),
            z: pixel.get(ShaderValue::PositionZ)
        };

        let normal = Point3D{
            x: pixel.get(ShaderValue::NormalX),
            y: pixel.get(ShaderValue::NormalY),
            z: pixel.get(ShaderValue::NormalZ)
        };

        let mut brightness = 0.0;

        for light in shader.lights
        {
            let light_direction = (light.position - world_point).normalized();
            let diffuse = normal.dot(light_direction).max(0.0);

            //camera is always at 0 0 0 for me
            let camera_direction = (-world_point).normalized();
            let reflect_direction = -light_direction.reflect(normal);

            let specular = camera_direction.dot(reflect_direction).max(0.0).powi(shininess);

            brightness += (diffuse + specular) * light.intensity;
        }

        ambient_color.lerp(&Color::new(1.0, 1.0, 1.0), brightness)
    } else
    {
        Color::new(0.0, 0.0, 0.0)
    }
}