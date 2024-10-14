use std::{thread, time::Duration};
use clap::{Parser, ValueEnum};
use rand::random;
pub use crate::life::{LifeWorld, LifeCell, LifePattern};
use crate::ui::{Camera, Terminal};
mod life;
mod ui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'd', long = "duration", default_value = "10.0")] duration: f64,
    #[arg(short = 'r', long = "rate", default_value = "250")]
    rate: u64,
    #[arg(short = 'p', long = "pattern", value_enum, default_value_t = LifePattern::Glider)]
    pattern: LifePattern,
}

impl Args {
    fn validate(&self) -> Result<(), String> {
        if self.duration <= 0.0 {
            Err("Duration must be positive".to_string())
        } else {
            Ok(())
        }
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
}

impl State {
    fn handle_command(&mut self, command: &Command) -> Result<&State, String> {
        match (self.clone(), command) {
            (Self::Setup, Command::Start) | (Self::Paused, Command::Resume) => {
                *self = Self::Running;
                Ok(self)
            },
            (Self::Running, Command::Pause) => {
                *self = Self::Paused;
                Ok(self)
            },
            (Self::Running, Command::Start | Command::Resume) | (Self::Paused, Command::Pause) => {
                Ok(self)
            },
            (_, Command::Quit) => {
                *self = Self::Setup;
                Ok(self)
            },
            _ => {
                Err(format!("Invalid command {:?} for state {:?}", command, self))
            },
        }
    }
}

#[derive(Debug)]
struct Program {
    state: State,
    world: LifeWorld,
    camera: Camera,
    terminal: Terminal,
    rate_ms: u64,
}

impl Program {
    fn new(args: Args) -> Self {
        let state = State::Setup;
        let camera = Camera::new();
        let terminal = Terminal::new();
        let world = LifeWorld::from(&args.pattern);
        let rate_ms = args.rate;
        Self { state, world, camera, terminal, rate_ms }
    }

    fn run(&mut self, duration_ms: u64) {
        match self.state.handle_command(&Command::Start) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        }
        self.terminal.clear();
        let duration_steps = duration_ms / self.rate_ms;
        let mut count = 0;
        while count < duration_steps {
            self.terminal.reset_cursor();
            self.camera.render(&self.world, self.terminal.width, self.terminal.height);
            wait(self.rate_ms);
            self.world.evolve();
            count += 1;
        }
    }
    
}

fn wait(time: u64) {
    thread::sleep(Duration::from_millis(time));
}

fn main() {
    let args = Args::parse();
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    let duration_ms = (args.duration * 1000.0).round_ties_even() as u64;

    let mut program = Program::new(args);
    program.run(duration_ms);
}
