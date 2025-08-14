use clap::Parser;
use enigo::Coordinate::Rel;
use enigo::{Enigo, InputError, Mouse, Settings};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = 1)]
    width: i32,

    #[arg(short, long, default_value_t = 10.0)]
    interval: f32,
}

struct Coordinate {
    x: i32,
    y: i32,
}

impl Coordinate {
    fn new(x: i32, y: i32) -> Self {
        Coordinate { x, y }
    }
}

struct Quaker {
    enigo: Enigo,
}

impl Quaker {
    fn new(enigo: Enigo) -> Self {
        Quaker { enigo }
    }

    fn quake(&mut self, delta: Coordinate) -> Result<Coordinate, InputError> {
        let next = Coordinate::new(-delta.x, delta.y);
        self.enigo.move_mouse(delta.x, delta.y, Rel)?;
        Ok(next)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let Cli { width, interval } = Cli::parse();
    let enigo = Enigo::new(&Settings::default())?;
    let mut quaker = Quaker::new(enigo);
    let mut delta = Coordinate::new(width, 0);
    let term = Arc::new(AtomicBool::new(false));
    let sig_check_interval: f32 = 0.5;

    for sig in TERM_SIGNALS {
        flag::register(*sig, Arc::clone(&term))?;
    }

    while !term.load(Ordering::Relaxed) {
        delta = quaker.quake(delta)?;

        let mut elapsed: f32 = 0.;
        while !term.load(Ordering::Relaxed) && elapsed < interval {
            thread::sleep(Duration::from_secs_f32(sig_check_interval));
            elapsed += sig_check_interval;
        }
    }

    Ok(())
}
