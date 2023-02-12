use std::{
    collections::{HashMap, hash_map::Entry},
    path::Path,
    io::{self, Read},
    fs::File
};

use image::error::ImageError;

use crate::renderer::common::{Color, Point2D, Point3D, Texture};


#[allow(dead_code)]
#[derive(Debug)]
pub struct ModelError
{
    line_index: Option<usize>,
    error_type: ModelErrorType
}

#[allow(dead_code)]
#[derive(Debug)]
enum ModelErrorType
{
    Io(io::Error),
    Material(Option<usize>),
    ParsingError(String),
    TextureLoadError(ImageError),
    GenericError,
    MissingValue,
    MissingMaterial,
    NoMaterial,
    NoNormals,
    MalformedObj
}

impl From<io::Error> for ModelError
{
    fn from(value: io::Error) -> Self
    {
        ModelError{line_index: None, error_type: ModelErrorType::from(value)}
    }
}

impl From<io::Error> for ModelErrorType
{
    fn from(value: io::Error) -> Self
    {
        ModelErrorType::Io(value)
    }
}

#[derive(Debug, Clone)]
struct ObjLine<'a, I>
{
    index: usize,
    field: &'a str,
    rest: &'a str,
    values: I
}

impl<'a, I> ObjLine<'a, I>
{
    pub fn rest(&self) -> &str
    {
        self.rest
    }
}

impl<'a, I: Iterator<Item=&'a str>> ObjLine<'a, I>
{
    pub fn next_value(&mut self) -> Result<&'a str, ModelErrorType>
    {
        let value = loop
        {
            let value = self.values.next().ok_or(ModelErrorType::MissingValue);

            if value.as_ref().unwrap_or(&"").is_empty()
            {
                continue;
            }

            break value;
        };

        value
    }
}

#[derive(Debug, Clone)]
pub struct Material
{
    pub diffuse_color: Option<Color>,
    pub diffuse_texture: Option<Texture>
}

impl Material
{
    pub fn new() -> Self
    {
        Material{diffuse_color: None, diffuse_texture: None}
    }
}

struct Materials
{
    fallback_material: Material,
    material_indices: HashMap<String, usize>,
    materials: Vec<Material>,
    current_material: Option<usize>
}

impl Materials
{
    pub fn new() -> Self
    {
        Materials{
            fallback_material: Material{
                diffuse_color: Some(Color::new(0.5, 0.5, 0.5)),
                diffuse_texture: None
            },
            material_indices: HashMap::new(),
            materials: Vec::new(),
            current_material: None
        }
    }

    pub fn materials(&self) -> &[Material]
    {
        &self.materials
    }

    pub fn add(&mut self, name: String) -> Result<(), ModelErrorType>
    {
        if let Entry::Vacant(entry) = self.material_indices.entry(name.clone())
        {
            let index = self.materials.len();

            entry.insert(index);
            self.materials.push(Material::new());

            self.current_material = Some(index);

            Ok(())
        } else
        {
            Err(ModelErrorType::Material(None))
        }
    }

    pub fn current(&mut self) -> &mut Material
    {
        if let Some(index) = self.current_material
        {
            &mut self.materials[index]
        } else
        {
            eprintln!("no current material, using fallback");
            &mut self.fallback_material
        }
    }

    pub fn current_index(&self) -> Option<usize>
    {
        self.current_material
    }

    pub fn set_current(&mut self, name: &str) -> Result<(), ModelErrorType>
    {
        let index = self.material_indices.get(name).ok_or(ModelErrorType::MissingMaterial);
        self.current_material = match index
        {
            Ok(index) => Some(*index),
            Err(_) => None
        };

        index.map(|_| ())
    }

    pub fn clear_current(&mut self)
    {
        self.current_material = None;
    }

    pub fn set_diffuse(&mut self, color: Color)
    {
        let material = self.current();
        *material = Material{diffuse_color: Some(color), ..material.clone()};
    }

    pub fn set_diffuse_texture(&mut self, texture: Texture)
    {
        let material = self.current();
        *material = Material{diffuse_texture: Some(texture), ..*material};
    }
}

struct ModelParser<'a>
{
    parent: &'a mut Model,
    materials: Materials,
    normals: Vec<Point3D>,
    uvs: Vec<Point2D>
}

impl<'a> ModelParser<'a>
{
    pub fn new(parent: &'a mut Model) -> Self
    {
        ModelParser{parent, materials: Materials::new(), normals: Vec::new(), uvs: Vec::new()}
    }

    pub fn parse(&mut self, filename: &str) -> Result<(), ModelError>
    {
        let mut file_string = String::new();
        File::open(filename)?.read_to_string(&mut file_string)?;

        let parent_dir = Path::new(filename).parent().unwrap_or_else(|| Path::new(""));

        for line in Self::parse_obj(&file_string)
        {
            let index = line.index;
            if let Err(error_type) = self.parse_obj_line(parent_dir, line)
            {
                match error_type
                {
                    ModelErrorType::Io(error) =>
                    {
                        eprintln!("ignoring io error: {error:?}");
                        continue;
                    },
                    _ =>
                    {
                        return Err(ModelError{line_index: Some(index), error_type});
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_obj_line<'b, I: Iterator<Item=&'b str>>(
        &mut self,
        parent_dir: &Path,
        line: ObjLine<'b, I>
    ) -> Result<(), ModelErrorType>
    {
        match line.field
        {
            "mtllib" =>
            {
                let path = parent_dir.join(Self::correctify_path(line.rest()));

                let mut mtl_string = String::new();
                File::open(path)?.read_to_string(&mut mtl_string)?;

                for line in Self::parse_obj(&mtl_string)
                {
                    let index = line.index;
                    if let Err(_) = self.parse_mtl_line(parent_dir, line)
                    {
                        return Err(ModelErrorType::Material(Some(index)));
                    }
                }

                self.materials.clear_current();
                self.parent.materials.extend(self.materials.materials().iter().cloned());

                Ok(())
            },
            "usemtl" =>
            {
                let _ = self.materials.set_current(&line.rest().to_owned());

                Ok(())
            },
            "v" =>
            {
                let mut vertices = Self::parse_floats(line.values, 3)?;
                self.parent.vertices.append(&mut vertices);

                Ok(())
            },
            "vn" =>
            {
                let normal = Self::parse_floats(line.values, 3)?;

                self.normals.push(Point3D{x: normal[0], y: normal[1], z: normal[2]}.normalized());

                Ok(())
            },
            "vt" =>
            {
                let uv = Self::parse_floats(line.values, 2)?;

                self.uvs.push(Point2D{x: uv[0], y: uv[1]});

                Ok(())
            },
            "f" => self.parse_face(line.values),
            _ => Ok(())
        }
    }

    fn parse_mtl_line<'b, I: Iterator<Item=&'b str>>(
        &mut self,
        parent_dir: &Path,
        mut line: ObjLine<'b, I>
    ) -> Result<(), ModelErrorType>
    {
        match line.field
        {
            "newmtl" =>
            {
                let material_name = line.next_value()?.to_owned();
                self.materials.add(material_name)
            },
            "Kd" =>
            {
                let mut component = || -> Result<f64, ModelErrorType>
                {
                    line.next_value()?.trim().parse().map_err(|_| ModelErrorType::Material(None))
                };

                let color = Color::new(
                    component()?,
                    component()?,
                    component()?
                );

                self.materials.set_diffuse(color);

                Ok(())
            },
            "map_Kd" =>
            {
                let path = parent_dir.join(Self::correctify_path(line.rest()));

                match Texture::load(&path)
                {
                    Ok(texture) => self.materials.set_diffuse_texture(texture),
                    Err(err) => eprintln!("error loading texture {err}")
                }

                Ok(())
            },
            _ => Ok(())
        }
    }

    fn correctify_path(wrong_path: &str) -> String
    {
        //i hate windows and its backslashes

        wrong_path.trim().replace('\\', "/")
    }

    fn parse_obj<'b>(text: &'b str) -> impl Iterator<Item=ObjLine<'b, impl Iterator<Item=&'b str>>>
    {
        text.lines().enumerate().filter_map(|(index, line)|
        {
            let line = Self::remove_comments(line).trim();

            if line.len() == 0
            {
                return None;
            }

            let mut splits = line.split(' ');
            let data_type = splits.next().unwrap();

            let space_position = line.find(' ');

            let mut after_space = "";
            if let Some(position) = space_position
            {
                after_space = &line[position+1..];
            }

            Some(ObjLine{index, field: data_type, rest: after_space, values: splits})
        })
    }

    fn parse_floats<'b>(
        mut unparsed: impl Iterator<Item=&'b str>,
        amount: usize
    ) -> Result<Vec<f64>, ModelErrorType>
    {
        (0..amount).map(|_|
        {
            let value = loop
            {
                let value = unparsed.next().ok_or(ModelErrorType::MissingValue)?;

                if value.is_empty()
                {
                    continue;
                }

                break value;
            };

            value.trim().parse()
                .map_err(|_| ModelErrorType::ParsingError(value.to_owned()))
        }).collect()
    }

    fn parse_face<'b>(
        &mut self,
        unparsed: impl Iterator<Item=&'b str>
    ) -> Result<(), ModelErrorType>
    {
        #[allow(dead_code)]
        struct FacePoint
        {
            position: usize,
            texture: Option<usize>,
            normal: Option<usize>
        }

        let mut face: Vec<FacePoint> = Vec::new();
        for value in unparsed
        {
            if value.is_empty()
            {
                continue;
            }

            let mut face_point: Option<FacePoint> = None;

            for (index_type, index) in value.split('/').enumerate()
            {
                let value: i32 = index.trim().parse()
                    .map_err(|_| ModelErrorType::GenericError)?;

                let value = if value < 0
                {
                    //start from the back
                    (self.parent.vertices.len() as i32 + value) as usize
                } else
                {
                    //the indices start from 1 in it for some reason??
                    (value - 1) as usize
                };

                match index_type
                {
                    0 =>
                    {
                        face_point = Some(FacePoint{position: value, texture: None, normal: None});
                    },
                    1 =>
                    {
                        face_point = Some(FacePoint{
                            texture: Some(value),
                            ..face_point.expect("already initialized")
                        });
                    },
                    2 =>
                    {
                        face_point = Some(FacePoint{
                            normal: Some(value),
                            ..face_point.expect("already initialized")
                        });
                    },
                    _ => ()
                }
            }

            face.push(face_point.expect("all faces have at least vertices"));
        }

        if face.len() < 3
        {
            return Err(ModelErrorType::GenericError);
        }

        let mut insert_face = |index| -> Result<(), ModelErrorType>
        {
            let face: &FacePoint = &face[index];

            self.parent.indices.push(face.position);

            if let Some(index) = face.texture
            {
                self.parent.uvs.push(self.uvs[index]);
            }

            if let Some(index) = face.normal
            {
                self.parent.normals.push(self.normals[index]);
            }

            Ok(())
        };

        for v in 2..face.len()
        {
            insert_face(v - 1)?;
            insert_face(v)?;
            insert_face(0)?;

            self.parent.material_indices.push(self.materials.current_index());
        }

        Ok(())
    }

    fn remove_comments<'b>(line: &'b str) -> &'b str
    {
        let comment_pos = line.find('#');

        if let Some(pos) = comment_pos
        {
            &line[..pos]
        } else
        {
            line
        }
    }
}

pub struct Model
{
    pub vertices: Vec<f64>,
    pub indices: Vec<usize>,
    pub material_indices: Vec<Option<usize>>,
    pub normals: Vec<Point3D>,
    pub uvs: Vec<Point2D>,
    pub materials: Vec<Material>
}

#[allow(dead_code)]
impl Model
{
    pub fn new() -> Self
    {
        Model{
            vertices: Vec::new(),
            indices: Vec::new(),
            material_indices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            materials: Vec::new()
        }
    }

    pub fn read_obj(filename: &str) -> Result<Self, ModelError>
    {
        let mut model = Model::new();
        let mut parser = ModelParser::new(&mut model);

        parser.parse(filename)?;

        Ok(model)
    }
}