use std::{thread, time::Duration};
use clap::{Parser, ValueEnum};
use rand::random;
pub use crate::life::{LifeWorld, LifeCell};
use crate::ui::{Camera, Terminal};
mod life;
mod ui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'd', long = "duration", default_value = "10.0")] duration: f64,
    #[arg(short = 'r', long = "rate", default_value = "250")]
    rate: u64,
    #[arg(short = 's', long = "snapshot")]
    snapshot: bool,
    #[arg(short = 'p', long = "pattern", value_enum, default_value_t = Pattern::Glider)]
    pattern: Pattern,
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

#[derive(Clone, Debug)]
enum Pattern {
    Glider,
    Blinker,
    Beacon,
    Random,
}

impl ValueEnum for Pattern {
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

fn create_world(pattern: Pattern) -> LifeWorld {
    let mut world = LifeWorld::new();
    match pattern {
        Pattern::Glider => {
            world.raise_cell(0, 0);
            world.raise_cell(1, 0);
            world.raise_cell(2, 0);
            world.raise_cell(2, 1);
            world.raise_cell(1, 2);
        }
        Pattern::Blinker => {
            world.raise_cell(0, 0);
            world.raise_cell(0, 1);
            world.raise_cell(0, 2);
        }
        Pattern::Beacon => {
            world.raise_cell(0, 0);
            world.raise_cell(0, 1);
            world.raise_cell(1, 0);
            world.raise_cell(1, 1);
            world.raise_cell(2, 2);
            world.raise_cell(3, 2);
            world.raise_cell(2, 3);
            world.raise_cell(3, 3);
        }
        Pattern::Random => {
            for _ in 0..1000 {
                let x = random::<i32>() % 80;
                let y = random::<i32>() % 25;
                world.raise_cell(x, y);
            }
        }
    }
    world
}

fn wait(time: u64) {
    thread::sleep(Duration::from_millis(time));
}

fn render_loop(world: &mut LifeWorld, camera: &Camera, terminal: &Terminal, duration_ms: u64, rate_ms: u64) {
    terminal.clear();
    let duration_steps = duration_ms / rate_ms;
    let mut count = 0;
    while count < duration_steps {
        terminal.reset_cursor();
        camera.render(&world, terminal.width, terminal.height);
        wait(rate_ms);
        world.evolve();
        count += 1;
    }
}

fn render_snapshot(world: &LifeWorld, camera: &Camera, terminal: &Terminal, duration_ms: u64) {
    terminal.clear();
    terminal.reset_cursor();
    camera.render(&world, terminal.width, terminal.height);
    wait(duration_ms);
}

fn main() {
    let args = Args::parse();
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let camera = Camera::new();
    let terminal = Terminal::new();
    let mut world = create_world(args.pattern);

    let duration_ms = (args.duration * 1000.0).round_ties_even() as u64;
    let rate_ms = args.rate;
    match args.snapshot {
        false => render_loop(&mut world, &camera, &terminal, duration_ms, rate_ms),
        true => render_snapshot(&world, &camera, &terminal, duration_ms),
    }
}
