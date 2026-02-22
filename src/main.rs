use std::{
    f64,
    env,
    thread,
    time::{Duration, Instant}
};

use config::{DrawMode, Config};

use renderer::{
    Transform,
    Camera,
    Object,
    common::{Point3D, Light},
    model::Model,
    normal_drawable::{
        DrawableNormal,
        DrawableDisplay,
        DrawSurface,
        NormalDrawable,
        DeferredDrawable
    },
    picture::Picture,
    console_screen::ConsoleScreen
};

mod config;

mod renderer;


fn main()
{
    let config = Config::parse(env::args().skip(1))
        .unwrap_or_else(|err| Config::help_message(Some(err)));

    let size = mode_size(&config);

    let model = Model::read_obj(&config.model_path).unwrap();
    let transform = Transform::new(
        (0.0, 0.0, -config.distance),
        (1.0, 1.0, 1.0),
        config.rotation,
        (0.2, 0.3, 0.4)
    );

    let fov = 60.0;
    let aspect = size.0 as f64 / size.1 as f64;

    let camera = Camera::new(0.1, 100.0, (fov * f64::consts::PI) / 180.0, aspect);

    let lights = [Light{
        position: Point3D{x: 50.0, y: 20.0, z: 30.0},
        //color: Color::new(1.0, 0.05, 0.08),
        intensity: 0.4
    }];

    let mut object = Object::new(&model, transform, &camera, &lights);

    draw_full(config, &mut object);
}

fn draw<D: DrawableNormal>(object: &Object, drawable: &mut D)
{
    let mut surface = drawable.surface();

    object.draw(&mut surface);
    surface.display();
}

fn draw_length<D: DrawableNormal>(config: &Config, object: &mut Object, mut drawable: D)
{
    match config.draw_mode
    {
        DrawMode::Picture =>
        {
            draw(&object, &mut drawable);
        },
        DrawMode::Console =>
        {
            let frame_delay = Duration::from_millis(100);
            loop
            {
                let frame_begin = Instant::now();

                draw(&object, &mut drawable);

                let rotation = object.rotation();
                object.set_rotation(rotation + 0.25);
                object.update_transform();

                if let Some(to_frame) = frame_delay.checked_sub(frame_begin.elapsed())
                {
                    thread::sleep(to_frame);
                }
            }
        }
    }
}

fn draw_mode<D: DrawableDisplay>(config: &Config, object: &mut Object, display: D)
{
    let size = mode_size(config);

    if config.undeferred
    {
        draw_length(config, object, &mut NormalDrawable::new(size, display));
    } else
    {
        draw_length(config, object, &mut DeferredDrawable::new(size, display));
    }
}

fn draw_full(config: Config, object: &mut Object)
{
    match config.draw_mode
    {
        DrawMode::Picture => draw_mode(&config, object, Picture::new(config.filename.clone())),
        DrawMode::Console => draw_mode(&config, object, ConsoleScreen::new())
    }
}

fn mode_size(config: &Config) -> (usize, usize)
{
    match config.draw_mode
    {
        DrawMode::Picture => config.size.unwrap_or((512, 512)),
        DrawMode::Console =>
        {
            let size = ConsoleScreen::terminal_size();

            config.size.unwrap_or_else(|| (size.0, size.1.saturating_sub(1)))
        }
    }
}
