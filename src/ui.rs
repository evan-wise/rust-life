use crate::life::LifeWorld;
use std::io::{self, Write};
use crossterm::{
    event::{self, Event, KeyCode}, ExecutableCommand
};
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::cursor::{MoveTo, Hide, Show};

macro_rules! flush {
    () => {
        std::io::stdout().flush().expect("Failed to flush stdout");
    };
}

#[derive(Debug)]
pub struct Camera {
    pub x: i32,
    pub y: i32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera { x: 0, y: 0 }
    }

    pub fn render(&self, world: &LifeWorld, w: u16, h: u16) {
        let x0 = self.x - (w as i32 / 2);
        let y0 = self.y - (h as i32 / 2);
        let x1 = self.x + (w as i32 / 2);
        let y1 = self.y + (h as i32 / 2);
        for y in (y0..y1).rev() {
            for x in x0..x1 {
                match (x, y, world.get(x, y)) {
                    (_x, _y, Some(cell)) if cell.alive => print!("\u{2588}"),
                    (_, _, Some(_)) => print!("\u{2591}"),
                    (x, y, None) if x == 0 && y == 0 => print!("\u{25CF}"),
                    (x, y, None) if x % 4 == 0 && y % 4 == 0 => print!("+"),
                    (x, _, None) if x % 8 == 0 => print!("|"),
                    (_, y, None) if y % 8 == 0 => print!("-"),
                    (_, _, _) => print!(" "),
                }
            }
            flush!();
        }
    }
}

#[derive(Debug)]
pub struct Screen {
    pub width: u16,
    pub height: u16,
    pub camera: Camera,
}

impl Screen {
    pub fn new() -> Result<Screen, io::Error>{
        Screen::acquire_terminal()?;
        let (w, h) = size()?;
        let camera = Camera::new();
        Ok(Screen { width: w, height: h, camera })
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
}

impl Drop for Screen {
    fn drop(&mut self) {
        if let Err(e) = Screen::release_terminal() {
            eprintln!("Error releasing terminal: {}", e);
        }
    }
}
