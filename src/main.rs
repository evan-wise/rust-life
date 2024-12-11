use clap::{Parser, ValueEnum};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ctrlc;
use std::io;
use std::time::{Duration, Instant};
mod life;
mod ui;
pub use crate::life::{LifeCell, LifePattern, LifeWorld};
use crate::ui::Screen;

fn main() {
    let args = Args::parse();
    if let Err(e) = args.validate() {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    let mut program = match Program::new(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    match program.run() {
        Ok(_) => {
            println!("Program completed successfully");
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'd', long = "duration", default_value = "10.0")]
    duration: f64,
    #[arg(short = 't', long = "timestep", default_value = "100")]
    timestep: u32,
    #[arg(short = 'p', long = "pattern", value_enum, default_value_t = LifePattern::Glider)]
    pattern: LifePattern,
}

impl Args {
    fn validate(&self) -> Result<(), ProgramError> {
        if self.duration <= 0.0 {
            return Err(ProgramError::ValidationError(
                "Duration must be positive".to_string(),
            ));
        }
        Ok(())
    }
}

impl ValueEnum for LifePattern {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Glider, Self::Blinker, Self::Beacon, Self::Random]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::Glider => Some(clap::builder::PossibleValue::new("glider").alias("g")),
            Self::Blinker => Some(clap::builder::PossibleValue::new("blinker").alias("bl")),
            Self::Beacon => Some(clap::builder::PossibleValue::new("beacon").alias("be")),
            Self::Random => Some(clap::builder::PossibleValue::new("random").alias("r")),
        }
    }
}

#[derive(Debug)]
struct Program {
    pub state: State,
    pub world: LifeWorld,
    pub screen: Screen,
    pub timestep_ms: u32,
    pub duration_ms: u32,
    pub perf: Performance,
}

impl Program {
    fn new(args: Args) -> Result<Self, ProgramError> {
        let state = State::Setup;
        let perf = Performance::new();
        let timestep_ms = args.timestep;
        let duration_ms = (args.duration * 1000.0).round_ties_even() as u32;

        let screen = Screen::new()?;
        let world = LifeWorld::from(&args.pattern);
        // Since we are using raw mode, we will have to handle this ourselves
        // later but catch the signal just in case the SIGINT gets sent by an
        // external process.
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
            duration_ms,
            perf,
        })
    }

    fn run(&mut self) -> Result<(), ProgramError> {
        match self.state {
            State::Setup => {
                self.screen.clear()?;
                self.state.handle_command(&Command::Start)?;
                let duration_steps = self.duration_ms / self.timestep_ms;
                let mut count = 0;
                let mut timestep_start = Instant::now();
                while count < duration_steps {
                    self.handle_input()?;
                    match self.state {
                        State::Running => {
                            if timestep_start.elapsed()
                                >= Duration::from_millis(self.timestep_ms.into())
                            {
                                self.perf.measured_timestep_ms =
                                    timestep_start.elapsed().as_millis() as u64;
                                timestep_start = Instant::now();
                                count += 1;
                                self.world.evolve();
                            }
                        }
                        State::Done => {
                            break;
                        }
                        _ => {}
                    }
                    let render_start = Instant::now();
                    self.screen.render(&self)?;
                    self.perf.render_ms = render_start.elapsed().as_millis() as u64;
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
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match code {
                    KeyCode::Esc => {
                        self.state.handle_command(&Command::Quit)?;
                    }
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        self.state.handle_command(&Command::Quit)?;
                    }
                    KeyCode::Char(' ') => match self.state {
                        State::Running => {
                            self.state.handle_command(&Command::Pause)?;
                        }
                        State::Paused => {
                            self.state.handle_command(&Command::Resume)?;
                        }
                        _ => {}
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
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Performance {
    pub measured_timestep_ms: u64,
    pub render_ms: u64,
}

impl Performance {
    fn new() -> Performance {
        Performance {
            measured_timestep_ms: 0,
            render_ms: 0,
        }
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
    ClapError(clap::Error),
    CtrlcError(ctrlc::Error),
    ValidationError(String),
    CommandError(String),
}

impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IoError: {:?}", e),
            Self::ClapError(e) => write!(f, "ClapError: {:?}", e),
            Self::ValidationError(e) => write!(f, "ValidationError: {:?}", e),
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

impl From<clap::Error> for ProgramError {
    fn from(e: clap::Error) -> Self {
        Self::ClapError(e)
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
