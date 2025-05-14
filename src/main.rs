use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ctrlc;
use std::time::{Duration, Instant};
mod life;
mod ui;
pub use crate::life::{LifePattern, LifeWorld};
use crate::ui::Screen;

fn main() -> Result<()> {
    let args = Args::parse();
    let mut program = Program::new(args)?;
    program.run()?;
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 't', long = "timestep", default_value = "100")]
    timestep: u32,
    #[arg(short = 'p', long = "pattern", value_enum, default_value_t = LifePattern::Blank)]
    pattern: LifePattern,
}

impl ValueEnum for LifePattern {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Blank,
            Self::Glider,
            Self::Blinker,
            Self::Beacon,
            Self::Random(10000),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::Blank => Some(clap::builder::PossibleValue::new("blank").alias("b")),
            Self::Glider => Some(clap::builder::PossibleValue::new("glider").alias("g")),
            Self::Blinker => Some(clap::builder::PossibleValue::new("blinker").alias("bl")),
            Self::Beacon => Some(clap::builder::PossibleValue::new("beacon").alias("be")),
            Self::Random(_) => Some(clap::builder::PossibleValue::new("random").alias("r")),
        }
    }
}

#[derive(Debug)]
struct Program {
    pub state: State,
    pub world: LifeWorld,
    pub cursor: Position,
    pub screen: Screen,
    pub timestep_ms: u32,
    pub tickrate: f64,
}

impl Program {
    fn new(args: Args) -> Result<Self> {
        let state = State::Setup;
        let timestep_ms = args.timestep;

        let screen = Screen::new()?;
        let world = LifeWorld::from(&args.pattern);
        // Since we are using raw mode, Ctrl+C will not send a SIGINT but catch the signal just in
        // case the SIGINT gets sent by an external process.
        ctrlc::set_handler(|| {
            if let Err(e) = Screen::release_terminal() {
                eprintln!("Failed to release terminal: {:?}", e);
            }
            println!("Received Ctrl-C, exiting...");
            std::process::exit(0);
        })?;
        Ok(Self {
            state,
            world,
            screen,
            timestep_ms,
            tickrate: 1000. / timestep_ms as f64,
            cursor: (0, 0),
        })
    }

    fn run(&mut self) -> Result<()> {
        self.state.handle_command(&Command::Start)?;
        self.screen.clear()?;

        let mut timestep = Duration::new(0, 0);
        loop {
            match self.state {
                State::Done => break,
                State::Setup => return Err(anyhow!("invalid state")),
                State::Paused => {
                    self.handle_input()?;
                    self.screen.render(&self)?;
                }
                State::Running => {
                    let input_time = Instant::now();
                    self.handle_input()?;
                    timestep += input_time.elapsed();
                    if self.state == State::Running {
                        if timestep >= Duration::from_millis(self.timestep_ms.into()) {
                            let simulation_time = Instant::now();
                            self.world.evolve();
                            timestep += simulation_time.elapsed();
                            self.tickrate = 1000. / timestep.as_millis() as f64;
                            timestep = Duration::new(0, 0);
                        }
                    }
                    let render_time = Instant::now();
                    self.screen.render(&self)?;
                    timestep += render_time.elapsed();
                }
            }
        }
        Ok(())
    }

    fn handle_input(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(2))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.state.handle_command(&Command::Quit)?;
                    }
                    KeyCode::Char(' ') => match self.state {
                        State::Running => {
                            self.state.handle_command(&Command::Pause)?;
                        }
                        State::Paused => {
                            self.state.handle_command(&Command::Resume)?;
                        }
                        _ => (),
                    },
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.screen.camera.y += 1;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.screen.camera.y -= 1;
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.screen.camera.x -= 1;
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.screen.camera.x += 1;
                    }
                    KeyCode::Char('w') => {
                        self.cursor.1 += 1;
                    }
                    KeyCode::Char('s') => {
                        self.cursor.1 -= 1;
                    }
                    KeyCode::Char('a') => {
                        self.cursor.0 -= 1;
                    }
                    KeyCode::Char('d') => {
                        self.cursor.0 += 1;
                    }
                    KeyCode::Char('c') => {
                        self.cursor = (self.screen.camera.x, self.screen.camera.y);
                    }
                    KeyCode::Char('e') => {
                        self.world.toggle(self.cursor.0, self.cursor.1);
                    }
                    KeyCode::Char('o') => {
                        self.screen.camera.x = 0;
                        self.screen.camera.y = 0;
                    }
                    _ => (),
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum Command {
    Start,
    Pause,
    Resume,
    Quit,
}

#[derive(PartialEq, Clone, Debug)]
enum State {
    Setup,
    Running,
    Paused,
    Done,
}

impl State {
    pub fn handle_command(&mut self, command: &Command) -> Result<&State> {
        match (self.clone(), command) {
            (Self::Setup, Command::Start) | (Self::Paused, Command::Resume) => {
                *self = Self::Running;
                Ok(self)
            }
            (Self::Running, Command::Pause) => {
                *self = Self::Paused;
                Ok(self)
            }
            (Self::Running, Command::Resume) | (Self::Paused, Command::Pause) => Ok(self),
            (_, Command::Quit) => {
                *self = Self::Done;
                Ok(self)
            }
            _ => Err(anyhow!(
                "Invalid command {:?} for state {:?}",
                command,
                self
            )),
        }
    }
}

type Position = (i32, i32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_from_setup() -> Result<()> {
        let mut state = State::Setup;
        state.handle_command(&Command::Start)?;
        assert_eq!(state, State::Running);
        Ok(())
    }

    #[test]
    fn resumes_from_pause() -> Result<()> {
        let mut state = State::Paused;
        state.handle_command(&Command::Resume)?;
        assert_eq!(state, State::Running);
        Ok(())
    }

    #[test]
    fn quits_from_running() -> Result<()> {
        let mut state = State::Running;
        state.handle_command(&Command::Quit)?;
        assert_eq!(state, State::Done);
        Ok(())
    }

    #[test]
    fn pauses_from_running() -> Result<()> {
        let mut state = State::Running;
        state.handle_command(&Command::Pause)?;
        assert_eq!(state, State::Paused);
        Ok(())
    }
}
