use crate::renderer::common::Color;
use crate::renderer::normal_drawable::DrawableDisplay;


pub struct ConsoleScreen
{
    written: bool
}

impl ConsoleScreen
{
    pub fn new() -> Self
    {
        ConsoleScreen{written: false}
    }

    fn move_cursor(&self, size: (usize, usize))
    {
        let (terminal_width, terminal_height) = Self::terminal_size();
        if size.0 > terminal_width || size.1 > terminal_height
        {
            panic!("size too big");
        }

        let x = 0;
        let y = terminal_height - size.1;

        print!("\x1b[{y};{x}H");
    }

    pub fn terminal_size() -> (usize, usize)
    {
        let winsize = libc::winsize{
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0
        };

        unsafe
        {
            libc::ioctl(0, libc::TIOCGWINSZ, &winsize);
        }

        (winsize.ws_col as usize, winsize.ws_row as usize)
    }

    fn output_color(color: Color)
    {
        let charset =
            "`.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";

        let charset = charset.as_bytes();

        let lightness = (color.r + color.g + color.b) / 3.0;

        let light_index = (lightness * charset.len() as f64) as usize;

        let index = light_index.min(charset.len()-1);

        let character;

        let inverted = false;
        if inverted
        {
            character = charset[charset.len() - 1 - index] as char;
        } else
        {
            character = charset[index] as char;
        }

        let colorify = |color| ((color * 6.0) as u8).min(5);

        let r = colorify(color.r);
        let g = colorify(color.g);
        let b = colorify(color.b);

        let color_code = 16 + r * 36 + g * 6 + b;

        print!("\x1b[38;5;{color_code}m{character}");
    }
}

#[allow(dead_code)]
impl DrawableDisplay for ConsoleScreen
{
    fn prepare(&mut self, size: (usize, usize))
    {
        if self.written
        {
            self.move_cursor(size);
        }

        self.written = true;
    }

    fn display(&self, size: (usize, usize), colors: &[Color])
    {
        for (index, color) in colors.iter().enumerate()
        {
            Self::output_color(*color);

            if (index % size.0) == (size.0 - 1)
            {
                println!();
            }
        }
    }
}