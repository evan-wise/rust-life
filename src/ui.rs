use crate::Program;
use anyhow::{anyhow, Result};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;

lazy_static! {
    static ref TERMINAL_ACQUIRED: AtomicBool = AtomicBool::new(false);
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
}

#[derive(Debug)]
pub struct Screen {
    pub width: u16,
    pub height: u16,
    pub camera: Camera,
}

impl Screen {
    pub fn new() -> Result<Screen> {
        Screen::acquire_terminal()?;
        let (w, h) = size()?;
        let camera = Camera::new();
        Ok(Screen {
            width: w,
            height: h,
            camera,
        })
    }

    pub fn acquire_terminal() -> Result<()> {
        if TERMINAL_ACQUIRED.compare_exchange(
            false, true, Ordering::SeqCst, Ordering::SeqCst
        ).is_err() {
            return Err(anyhow!("terminal already in use"));
        }
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(Hide)?;
        enable_raw_mode()?;
        Ok(())
    }

    pub fn release_terminal() -> Result<()> {
        let mut stdout = io::stdout();
        if TERMINAL_ACQUIRED.load(Ordering::SeqCst) {
            disable_raw_mode()?;
            stdout.execute(Show)?;
            stdout.execute(LeaveAlternateScreen)?;
            TERMINAL_ACQUIRED.store(false, Ordering::SeqCst);
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let mut stdout = io::stdout();
        stdout.execute(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn reset_cursor(&self) -> Result<()> {
        let mut stdout = io::stdout();
        stdout.execute(MoveTo(0, 0))?;
        Ok(())
    }

    pub fn render(&self, program: &Program) -> Result<()> {
        self.reset_cursor()?;
        let x0 = self.camera.x - (self.width as i32 / 2);
        let y0 = self.camera.y - (self.height as i32 / 2) + 1;
        let x1 = self.camera.x + (self.width as i32 / 2) + (self.width as i32 % 2);
        let y1 = self.camera.y + (self.height as i32 / 2) + (self.height as i32 % 2) - 1;

        for y in (y0..y1).rev() {
            for x in x0..x1 {
                let (cx, cy) = program.cursor;
                let a = program.world.alive(x, 2 * y);
                let b = program.world.alive(x, 2 * y + 1);
                let mut stdout = io::stdout();

                if x == cx && 2 * y == cy  {
                    stdout.execute(SetForegroundColor(Color::Green))?;
                    if b {
                        stdout.execute(SetBackgroundColor(Color::Grey))?;
                    }
                    print!("▄");
                    stdout.execute(ResetColor)?;
                } else if x == cx && 2 * y + 1 == cy {
                    stdout.execute(SetForegroundColor(Color::Green))?;
                    if a {
                        stdout.execute(SetBackgroundColor(Color::Grey))?;
                    }
                    print!("▀");
                    stdout.execute(ResetColor)?;
                } else {
                    match (x, y, a, b) {
                        (_, _, true, true)  => print!("█"),
                        (_, _, false, true) => print!("▀"),
                        (_, _, true, false) => print!("▄"),
                        (x, y, false, false) if x == 0 && y == 0 => print!("●"),
                        (x, y, false, false) if x % 4 == 0 && y % 2 == 0 => print!("┼"),
                        (x, _, false, false) if x % 8 == 0 => print!("│"),
                        (_, y, false, false) if y % 4 == 0 => print!("─"),
                        _ => print!(" "),
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::terminal::is_raw_mode_enabled;
    use serial_test::serial;

    #[test]
    #[serial]
    fn acquires_and_releases_terminal() -> Result<()> {
        assert!(!is_raw_mode_enabled()?);
        Screen::acquire_terminal()?;
        assert!(is_raw_mode_enabled()?);
        Screen::release_terminal()?;
        assert!(!is_raw_mode_enabled()?);
        Ok(())
    }

    #[test]
    #[serial]
    fn creates_screen() -> Result<()> {
        let screen = Screen::new()?;
        assert!(screen.width > 0);
        assert!(screen.height > 0);
        assert!(screen.camera.x == 0);
        assert!(screen.camera.y == 0);
        Ok(())
    }

    #[test]
    #[serial]
    fn lock_prevents_concurrent_access() -> Result<()> {
        let screen = Screen::new()?;
        let err = Screen::new().unwrap_err();
        assert_eq!(err.to_string(), String::from("terminal already in use"));
        assert!(screen.width > 0);
        assert!(screen.height > 0);
        assert!(screen.camera.x == 0);
        assert!(screen.camera.y == 0);
        Ok(())
    }
}
