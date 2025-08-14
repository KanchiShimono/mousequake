use clap::{Args, Command, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use enigo::Coordinate::Rel;
use enigo::{Enigo, InputError, Mouse, Settings};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about,
    long_about = r#"Simple tool for automatically shaking the mouse pointer

By default (without subcommands), mousequake will start shaking your mouse pointer immediately.
Use -w/--width to control the shake distance and -i/--interval to control the frequency.

Press Ctrl+C to stop."#,
    after_help = r#"EXAMPLES:
    mousequake                  # Start shaking with default settings (1px every 10s)
    mousequake -w 5 -i 30       # Shake 5 pixels every 30 seconds
    mousequake completion bash  # Generate bash completion script"#
)]
struct Cli {
    #[arg(
        short,
        long,
        default_value_t = 1,
        help = "Distance to move the mouse (pixels)"
    )]
    width: i32,

    #[arg(
        short,
        long,
        default_value_t = 10.0,
        help = "Time between mouse movements (seconds)"
    )]
    interval: f32,

    #[command(subcommand)]
    command: Option<SubCommand>,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    #[command(about = "Generate shell completion scripts")]
    Completion(CompletionCommand),
}

#[derive(Debug, Args)]
struct CompletionCommand {
    #[arg(value_enum, help = "Target shell for completion script")]
    shell: Shell,
}

impl CompletionCommand {
    fn execute(&self, cmd: &mut Command) -> Result<(), Box<dyn Error>> {
        clap_complete::generate(
            self.shell,
            cmd,
            cmd.get_name().to_string(),
            &mut std::io::stdout(),
        );
        Ok(())
    }
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

fn execute_quaker(width: i32, interval: f32) -> Result<(), Box<dyn Error>> {
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

fn main() -> Result<(), Box<dyn Error>> {
    let Cli {
        width,
        interval,
        command,
    } = Cli::parse();

    if let Some(command) = command {
        return match command {
            SubCommand::Completion(cmd) => {
                let mut command = Cli::command();
                cmd.execute(&mut command)
            }
        };
    }

    execute_quaker(width, interval)
}
