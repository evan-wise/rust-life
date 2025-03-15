use clap::{Parser, ValueEnum};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ctrlc;
use std::io;
use std::time::{Duration, Instant};
mod life;
mod ui;
pub use crate::life::{LifePattern, LifeWorld};
use crate::ui::Screen;

fn main() -> Result<(), ProgramError> {
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
    #[arg(short = 'p', long = "pattern", value_enum, default_value_t = LifePattern::Glider)]
    pattern: LifePattern,
}

impl ValueEnum for LifePattern {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Glider, Self::Blinker, Self::Beacon, Self::Random(100)]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
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
    pub screen: Screen,
    pub timestep_ms: u32,
    pub tickrate: f64,
}

impl Program {
    fn new(args: Args) -> Result<Self, ProgramError> {
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
        })
    }

    fn run(&mut self) -> Result<(), ProgramError> {
        match self.state {
            State::Setup => {
                self.screen.clear()?;
                self.state.handle_command(&Command::Start)?;

                let mut timestep = Duration::new(0, 0);
                loop {
                    let loop_start = Instant::now();
                    match self.state {
                        State::Running => {
                            if timestep >= Duration::from_millis(self.timestep_ms.into()) {
                                timestep = Duration::new(0, 0);
                                self.world.evolve();
                            }
                        }
                        State::Done => {
                            break;
                        }
                        _ => (),
                    }
                    self.handle_input()?;
                    self.screen.render(&self)?;
                    self.tickrate = 1000. / loop_start.elapsed().as_millis() as f64;
                    timestep += loop_start.elapsed();
                }
                Ok(())
            }
            _ => {
                return Err(ProgramError::CommandError(
                    "Run called in invalid state".to_string(),
                ))
            }
        }
    }

    fn handle_input(&mut self) -> Result<(), ProgramError> {
        if event::poll(Duration::from_millis(10))? {
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
                    KeyCode::Up => {
                        self.screen.camera.y += 1;
                    }
                    KeyCode::Down => {
                        self.screen.camera.y -= 1;
                    }
                    KeyCode::Left => {
                        self.screen.camera.x -= 1;
                    }
                    KeyCode::Right => {
                        self.screen.camera.x += 1;
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

#[derive(Clone, Debug)]
enum Command {
    Start,
    Pause,
    Resume,
    Quit,
}

#[derive(Clone, Debug)]
enum State {
    Setup,
    Running,
    Paused,
    Done,
}

impl State {
    pub fn handle_command(&mut self, command: &Command) -> Result<&State, String> {
        match (self.clone(), command) {
            (Self::Setup, Command::Start) | (Self::Paused, Command::Resume) => {
                *self = Self::Running;
                Ok(self)
            }
            (Self::Running, Command::Pause) => {
                *self = Self::Paused;
                Ok(self)
            }
            (Self::Running, Command::Start | Command::Resume) | (Self::Paused, Command::Pause) => {
                Ok(self)
            }
            (_, Command::Quit) => {
                *self = Self::Done;
                Ok(self)
            }
            _ => Err(format!(
                "Invalid command {:?} for state {:?}",
                command, self
            )),
        }
    }
}

#[derive(Debug)]
enum ProgramError {
    IoError(io::Error),
    CtrlcError(ctrlc::Error),
    CommandError(String),
}

impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IoError: {:?}", e),
            Self::CtrlcError(e) => write!(f, "CtrlcError: {:?}", e),
            Self::CommandError(e) => write!(f, "CommandError: {:?}", e),
        }
    }
}

impl From<io::Error> for ProgramError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<ctrlc::Error> for ProgramError {
    fn from(e: ctrlc::Error) -> Self {
        Self::CtrlcError(e)
    }
}

impl From<String> for ProgramError {
    fn from(e: String) -> Self {
        Self::CommandError(e)
    }
}
