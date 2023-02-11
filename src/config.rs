use std::{
    env,
    process
};

pub enum DrawMode
{
    Picture,
    Console
}

pub enum ConfigError
{
    ParseError(String),
    InvalidArg(String),
    MissingValue(String),
    DimensionMissing,
    PathMissing
}

pub struct Config
{
    pub model_path: String,
    pub draw_mode: DrawMode,
    pub size: Option<(usize, usize)>,
    pub distance: f64,
    pub rotation: f64,
    pub undeferred: bool
}

impl Config
{
    pub fn parse<T: Iterator<Item=String>>(args: T) -> Result<Self, ConfigError>
    {
        let mut model_path = None;
        let mut draw_mode = DrawMode::Picture;
        let mut size = None;
        let mut distance = 50.0;
        let mut rotation = 0.9;
        let mut undeferred = false;

        let mut args = args.peekable();
        while let Some(arg) = args.next()
        {
            if args.peek().is_none()
            {
                model_path = Some(arg);
                break;
            }

            let mut next_value = || args.next().ok_or(ConfigError::MissingValue(arg.clone()));
            match arg.as_str()
            {
                "-m" | "--mode" =>
                {
                    let value = next_value()?;
                    match value.to_lowercase().as_str()
                    {
                        "picture" =>
                        {
                            draw_mode = DrawMode::Picture;
                        },
                        "console" =>
                        {
                            draw_mode = DrawMode::Console;
                        },
                        _ => return Err(ConfigError::ParseError(value))
                    }
                },
                "-s" | "--size" =>
                {
                    let value = next_value()?;
                    let mut pair = value.split(' ');

                    let mut parse_value = || -> Result<usize, ConfigError>
                    {
                        pair.next().ok_or(ConfigError::DimensionMissing)?
                            .trim().parse::<usize>()
                            .map_err(|_| ConfigError::ParseError(value.clone()))
                    };

                    size = Some((parse_value()?, parse_value()?));
                },
                "-d" | "--distance" =>
                {
                    let value = next_value()?;
                    distance = value.trim().parse().map_err(|_| ConfigError::ParseError(value))?;
                },
                "-r" | "--rotation" =>
                {
                    let value = next_value()?;
                    rotation = value.trim().parse().map_err(|_| ConfigError::ParseError(value))?;
                },
                "-u" | "--undeferred" => undeferred = true,
                _ => return Err(ConfigError::InvalidArg(arg))
            }
        }

        let model_path = model_path.ok_or(ConfigError::PathMissing)?;
        Ok(Config{model_path, draw_mode, size, distance, rotation, undeferred})
    }

    pub fn help_message(error: Option<ConfigError>) -> !
    {
        if let Some(error) = error
        {
            let description = match error
            {
                ConfigError::ParseError(value) => format!("error parsing: {value}"),
                ConfigError::InvalidArg(value) => format!("invalid argument: {value}"),
                ConfigError::MissingValue(value) => format!("{value} argument is missing value"),
                ConfigError::DimensionMissing => "missing height in size parameter".to_owned(),
                ConfigError::PathMissing => "missing model path".to_owned()
            };

            println!("{description}\n");
        }

        println!("usage: {} [args] path/to/model.obj", env::args().nth(0).unwrap());
        println!("args:");
        println!("    -m, --mode          drawing mode (default picture)");
        println!("    -s, --size          space separated size of the resulting image (default 512 by 512)");
        println!("    -d, --distance      distance from the camera (default 50)");
        println!("    -r, --rotation      rotation of the object in radians (default 0.9)");
        println!("    -u, --undeferred    disables deferred rendering, uses less ram but slower");
        println!("modes:");
        println!("    picture, console");

        process::exit(1)
    }
}