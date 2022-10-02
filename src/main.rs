use enigo::{Enigo, MouseControllable};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use std::io::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "1")]
    width: i32,

    #[structopt(short, long, default_value = "10")]
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

    fn quake(&mut self, delta: Coordinate) -> Coordinate {
        let next = Coordinate::new(-delta.x, delta.y);
        self.enigo.mouse_move_relative(delta.x, delta.y);
        next
    }
}

fn main() -> Result<(), Error> {
    let Opt { width, interval } = Opt::from_args();
    let enigo = Enigo::new();
    let mut quaker = Quaker::new(enigo);
    let mut delta = Coordinate::new(width, 0);
    let term = Arc::new(AtomicBool::new(false));
    let sig_check_interval: f32 = 0.5;

    for sig in TERM_SIGNALS {
        flag::register(*sig, Arc::clone(&term))?;
    }

    while !term.load(Ordering::Relaxed) {
        delta = quaker.quake(delta);

        let mut elapsed: f32 = 0.;
        while !term.load(Ordering::Relaxed) && elapsed < interval {
            thread::sleep(Duration::from_secs_f32(sig_check_interval));
            elapsed += sig_check_interval;
        }
    }

    Ok(())
}
