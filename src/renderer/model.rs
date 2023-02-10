use std::{
    collections::{HashMap, hash_map::Entry},
    path::Path,
    io::{self, Read},
    fs::File
};

use crate::renderer::common::{Color, Point3D};


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
        self.values.next().ok_or(ModelErrorType::MissingValue)
    }
}

#[derive(Debug, Clone)]
struct Material
{
    diffuse_color: Option<Color>
}

impl Material
{
    pub fn new() -> Self
    {
        Material{diffuse_color: None}
    }
}

struct Materials
{
    fallback_material: Material,
    materials: HashMap<String, Material>,
    current_material: Option<String>
}

impl Materials
{
    pub fn new() -> Self
    {
        Materials{
            fallback_material: Material{diffuse_color: Some(Color::new(0.5, 0.5, 0.5))},
            materials: HashMap::new(),
            current_material: None
        }
    }

    pub fn add(&mut self, name: String) -> Result<(), ModelErrorType>
    {
        if let Entry::Vacant(entry) = self.materials.entry(name.clone())
        {
            self.current_material = Some(name);
            entry.insert(Material::new());

            Ok(())
        } else
        {
            Err(ModelErrorType::Material(None))
        }
    }

    pub fn current(&mut self) -> Result<&mut Material, ModelErrorType>
    {
        Ok(self.materials.get_mut(
            self.current_material.as_ref().ok_or(ModelErrorType::Material(None))?
        ).unwrap_or_else(||
        {
            eprintln!("error switching to material {:?}, using fallback", self.current_material);
            &mut self.fallback_material
        }))
    }

    pub fn set_current(&mut self, name: String)
    {
        self.current_material = Some(name);
    }

    pub fn clear_current(&mut self)
    {
        self.current_material = None;
    }

    pub fn set_diffuse(&mut self, color: Color) -> Result<(), ModelErrorType>
    {
        let material = self.current()?;

        *material = Material{diffuse_color: Some(color), ..*material};

        Ok(())
    }
}

struct ModelParser<'a>
{
    parent: &'a mut Model,
    materials: Materials,
    normals: Vec<Point3D>
}

impl<'a> ModelParser<'a>
{
    pub fn new(parent: &'a mut Model) -> Self
    {
        ModelParser{parent, materials: Materials::new(), normals: Vec::new()}
    }

    pub fn parse(&mut self, filename: &str) -> Result<(), ModelError>
    {
        let mut file_string = String::new();
        File::open(filename)?.read_to_string(&mut file_string)?;

        let parent_dir = Path::new(filename).parent().unwrap_or_else(|| Path::new(""));

        for (line_index, line) in Self::parse_obj(&file_string).enumerate()
        {
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
                        return Err(ModelError{line_index: Some(line_index), error_type});
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
                let path = parent_dir.join(line.rest());

                let mut mtl_string = String::new();
                File::open(path)?.read_to_string(&mut mtl_string)?;

                for (line_index, line) in Self::parse_obj(&mtl_string).enumerate()
                {
                    if let Err(_) = self.parse_mtl_line(line)
                    {
                        return Err(ModelErrorType::Material(Some(line_index)));
                    }
                }

                self.materials.clear_current();

                Ok(())
            },
            "usemtl" =>
            {
                self.materials.set_current(line.rest().to_owned());
                Ok(())
            },
            "v" => Self::parse_floats(line.values, &mut self.parent.vertices, 3),
            "vn" =>
            {
                let mut normal = Vec::with_capacity(3);
                Self::parse_floats(line.values, &mut normal, 3)?;

                self.normals.push(Point3D{x: normal[0], y: normal[1], z: normal[2]}.normalized());

                Ok(())
            },
            "f" => self.parse_face(line.values),
            _ => Ok(())
        }
    }

    fn parse_mtl_line<'b, I: Iterator<Item=&'b str>>(
        &mut self,
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
                    line.next_value()?.parse().map_err(|_| ModelErrorType::Material(None))
                };

                let color = Color::new(
                    component()?,
                    component()?,
                    component()?
                );

                self.materials.set_diffuse(color)
            },
            _ => Ok(())
        }
    }

    fn parse_obj<'b>(text: &'b str) -> impl Iterator<Item=ObjLine<'b, impl Iterator<Item=&'b str>>>
    {
        text.lines().filter_map(|line|
        {
            let line = Self::remove_comments(line);

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

            Some(ObjLine{field: data_type, rest: after_space, values: splits})
        })
    }

    fn parse_floats<'b>(
        mut unparsed: impl Iterator<Item=&'b str>,
        output_vector: &mut Vec<f64>,
        amount: usize
    ) -> Result<(), ModelErrorType>
    {
        for _ in 0..amount
        {
            let value = unparsed.next().ok_or(ModelErrorType::GenericError)?;
            let value: f64 = value.trim().parse()
                .map_err(|_| ModelErrorType::GenericError)?;

            output_vector.push(value);
        }

        Ok(())
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

        let material = self.materials.current()?;

        let mut insert_face = |index| -> Result<(), ModelErrorType>
        {
            let face: &FacePoint = &face[index];

            let normal_index = face.normal.ok_or(ModelErrorType::NoNormals)?;

            self.parent.indices.push(face.position);
            self.parent.normals.push(self.normals[normal_index]);

            Ok(())
        };

        for v in 2..face.len()
        {
            insert_face(v - 1)?;
            insert_face(v)?;
            insert_face(0)?;

            self.parent.colors.push(material.diffuse_color.ok_or(ModelErrorType::NoMaterial)?);
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
    pub colors: Vec<Color>,
    pub normals: Vec<Point3D>
}

#[allow(dead_code)]
impl Model
{
    pub fn new() -> Self
    {
        Model{vertices: Vec::new(), indices: Vec::new(), colors: Vec::new(), normals: Vec::new()}
    }

    pub fn read_obj(filename: &str) -> Result<Self, ModelError>
    {
        let mut model = Model::new();
        let mut parser = ModelParser::new(&mut model);

        parser.parse(filename)?;

        Ok(model)
    }
}