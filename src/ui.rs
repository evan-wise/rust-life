use std::io::Write;
use terminal_size::{terminal_size, Width, Height};
use crate::life::LifeWorld;

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

fn acquire_terminal() {
    // Save cursor position
    print!("\x1B7");
    // Hide cursor
    print!("\x1B[?25l");
    // Enter alternate screen buffer
    print!("\x1B[?1049h");
    flush!();
}

fn release_terminal() {
    // Exit alternate screen buffer
    print!("\x1B[?1049l");
    // Restore cursor position
    print!("\x1B8");
    // Show cursor
    print!("\x1B[?25h");
    flush!();
}

#[derive(Debug)]
pub struct Terminal {
    pub width: u16,
    pub height: u16,
}

impl Terminal {
    pub fn new() -> Terminal {
        acquire_terminal();
        match terminal_size() {
            Some((Width(w), Height(h))) => {
                Terminal { width: w, height: h }
            },
            None => {
                release_terminal();
                println!("Unable to get terminal size");
                std::process::exit(1);
            },
        }
    }

    pub fn clear(&self) {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        flush!();
    }

    pub fn reset_cursor(&self) {
        // Move cursor to top-left
        print!("\x1B[1;1H");
        flush!();
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        release_terminal();
    }
}
