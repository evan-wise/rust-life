use crate::Program;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use std::io::{self, Write};

#[derive(Debug)]
pub struct Camera {
    pub x: i32,
    pub y: i32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera { x: 0, y: 0 }
    }
}

#[derive(Debug)]
pub struct Screen {
    pub width: u16,
    pub height: u16,
    pub camera: Camera,
}

impl Screen {
    pub fn new() -> Result<Screen, io::Error> {
        Screen::acquire_terminal()?;
        let (w, h) = size()?;
        let camera = Camera::new();
        Ok(Screen {
            width: w,
            height: h,
            camera,
        })
    }

    pub fn acquire_terminal() -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        stdout.execute(Hide)?;
        Ok(())
    }

    pub fn release_terminal() -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        stdout.execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        stdout.execute(Show)?;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        stdout.execute(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn reset_cursor(&self) -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        stdout.execute(MoveTo(0, 0))?;
        Ok(())
    }

    pub fn render(&self, program: &Program) -> Result<(), io::Error> {
        self.reset_cursor()?;
        let x0 = self.camera.x - (self.width as i32 / 2);
        let y0 = self.camera.y - (self.height as i32 / 2) + 1;
        let x1 = self.camera.x + (self.width as i32 / 2) + (self.width as i32 % 2);
        let y1 = self.camera.y + (self.height as i32 / 2) + (self.height as i32 % 2) - 1;

        for y in (y0..y1).rev() {
            for x in x0..x1 {
                match (x, y, program.world.get(x, y)) {
                    (_x, _y, Some(cell)) if cell => print!("█"),
                    (_, _, Some(_)) => print!("░"),
                    (x, y, None) if x == 0 && y == 0 => print!("●"),
                    (x, y, None) if x % 4 == 0 && y % 4 == 0 => print!("┼"),
                    (x, _, None) if x % 8 == 0 => print!("│"),
                    (_, y, None) if y % 8 == 0 => print!("─"),
                    (_, _, _) => print!(" "),
                }
            }
        }

        for x in x0..x1 {
            if x % 8 == 0 {
                print!("┷");
            } else {
                print!("━");
            }
        }

        let status = format!(
            "alive: {}, generations: {}, tickrate: {:.2}Hz",
            program.world.num_alive(),
            program.world.generations,
            program.tickrate,
        );
        let pad = std::iter::repeat(" ")
            .take(usize::from(self.width) - status.len())
            .collect::<String>();
        print!("{}{}", status, pad);
        io::stdout().flush()?;

        Ok(())
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        if let Err(e) = Screen::release_terminal() {
            eprintln!("Error releasing terminal: {}", e);
        }
    }
}
