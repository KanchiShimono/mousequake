use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use clap::{Args, Command, CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;
use enigo::Coordinate::Rel;
use enigo::{Enigo, InputError, Mouse, Settings};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;

mod trajectory;
use trajectory::{
    CircleTrajectory, InfinityTrajectory, LinearTrajectory, SquareTrajectory, StarTrajectory,
    Trajectory,
};

#[derive(Debug, Clone, ValueEnum)]
enum TrajectoryType {
    Linear,
    Circle,
    Star,
    Square,
    #[value(alias = "figure8")]
    Infinity,
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about,
    long_about = r#"Simple tool for automatically shaking the mouse pointer

By default (without subcommands), mousequake will start shaking your mouse pointer immediately.
Use -s/--size to control the pattern size and -i/--interval to control the frequency.

Press Ctrl+C to stop."#,
    after_help = r#"EXAMPLES:
    mousequake                      # Start shaking with default linear pattern (1px every 10s)
    mousequake -s 5 -i 30           # Pattern size of 5 pixels every 30 seconds
    mousequake -t circle -s 10      # Move in a circle with 10px diameter
    mousequake -t star -s 20 -i 5   # Draw a star pattern, 20px size, every 5 seconds
    mousequake -t infinity -s 15    # Move in figure-8/infinity pattern, 15px size
    mousequake completion bash      # Generate bash completion script"#
)]
struct Cli {
    #[arg(
        short,
        long,
        default_value_t = 1,
        help = "Maximum width of the trajectory pattern (pixels)"
    )]
    size: i32,

    #[arg(
        short,
        long,
        default_value_t = 10.0,
        help = "Time between mouse movements (seconds)"
    )]
    interval: f32,

    #[arg(
        short,
        long,
        value_enum,
        default_value_t = TrajectoryType::Linear,
        help = "Trajectory pattern to use"
    )]
    trajectory: TrajectoryType,

    #[command(subcommand)]
    command: Option<Subcommand>,
}

#[derive(Debug, clap::Subcommand)]
enum Subcommand {
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

struct Quaker {
    enigo: Enigo,
    trajectory: Box<dyn Trajectory>,
}

impl Quaker {
    fn new(enigo: Enigo, trajectory: Box<dyn Trajectory>) -> Self {
        Quaker { enigo, trajectory }
    }

    fn quake(&mut self) -> Result<(), InputError> {
        let point = self.trajectory.next();
        self.enigo.move_mouse(point.x as i32, point.y as i32, Rel)?;
        Ok(())
    }
}

fn create_trajectory(trajectory_type: TrajectoryType, size: f32) -> Box<dyn Trajectory> {
    match trajectory_type {
        TrajectoryType::Linear => Box::new(LinearTrajectory::new(size)),
        TrajectoryType::Circle => Box::new(CircleTrajectory::new(size)),
        TrajectoryType::Star => Box::new(StarTrajectory::new(size)),
        TrajectoryType::Square => Box::new(SquareTrajectory::new(size)),
        TrajectoryType::Infinity => Box::new(InfinityTrajectory::new(size)),
    }
}

fn execute_quaker(
    size: i32,
    interval: f32,
    trajectory_type: TrajectoryType,
) -> Result<(), Box<dyn Error>> {
    let enigo = Enigo::new(&Settings::default())?;
    let trajectory = create_trajectory(trajectory_type, size as f32);
    let mut quaker = Quaker::new(enigo, trajectory);
    let term = Arc::new(AtomicBool::new(false));
    let sig_check_interval: f32 = 0.5;

    for sig in TERM_SIGNALS {
        flag::register(*sig, Arc::clone(&term))?;
    }

    while !term.load(Ordering::Relaxed) {
        quaker.quake()?;

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
        size,
        interval,
        trajectory,
        command,
    } = Cli::parse();

    if let Some(command) = command {
        return match command {
            Subcommand::Completion(cmd) => {
                let mut command = Cli::command();
                cmd.execute(&mut command)
            }
        };
    }

    execute_quaker(size, interval, trajectory)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn test_cli_default_values() {
        let cli = Cli::parse_from(["mousequake"]);
        assert_eq!(cli.size, 1);
        assert_eq!(cli.interval, 10.0);
        assert!(matches!(cli.trajectory, TrajectoryType::Linear));
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_custom_size() {
        let cli = Cli::parse_from(["mousequake", "-s", "5"]);
        assert_eq!(cli.size, 5);
        assert_eq!(cli.interval, 10.0);
    }

    #[test]
    fn test_cli_custom_interval() {
        let cli = Cli::parse_from(["mousequake", "-i", "30.5"]);
        assert_eq!(cli.size, 1);
        assert_eq!(cli.interval, 30.5);
    }

    #[test]
    fn test_cli_trajectory_types() {
        let cli = Cli::parse_from(["mousequake", "-t", "circle"]);
        assert!(matches!(cli.trajectory, TrajectoryType::Circle));

        let cli = Cli::parse_from(["mousequake", "-t", "star"]);
        assert!(matches!(cli.trajectory, TrajectoryType::Star));

        let cli = Cli::parse_from(["mousequake", "-t", "figure8"]);
        assert!(matches!(cli.trajectory, TrajectoryType::Infinity));
    }

    #[test]
    fn test_cli_completion_subcommand() {
        let cli = Cli::parse_from(["mousequake", "completion", "bash"]);
        assert!(matches!(cli.command, Some(Subcommand::Completion(_))));
    }
}
