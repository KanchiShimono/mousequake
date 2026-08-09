use std::fmt::{self, Display, Formatter};
use std::num::ParseFloatError;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, TryFromFloatSecsError};

use anyhow::{self, Context};
use clap::error::ErrorKind;
use clap::{Args, Command, CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;
use enigo::Coordinate::Rel;
use enigo::{Enigo, InputError, Mouse, Settings};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use thiserror::Error;

mod trajectory;
use trajectory::{Trajectory, TrajectoryExtent, TrajectorySpec, TrajectoryType};

const MIN_MOVEMENT_INTERVAL: Duration = Duration::from_millis(20);
const MAX_MOVEMENT_INTERVAL: Duration = Duration::from_secs(365 * 24 * 60 * 60);
const TERMINATION_CHECK_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MovementInterval(Duration);

impl MovementInterval {
    fn duration(self) -> Duration {
        self.0
    }
}

impl TryFrom<f64> for MovementInterval {
    type Error = MovementIntervalError;

    fn try_from(seconds: f64) -> Result<Self, Self::Error> {
        if !seconds.is_finite() {
            return Err(MovementIntervalError::NotFinite);
        }
        if seconds <= 0.0 {
            return Err(MovementIntervalError::NotPositive);
        }
        if seconds > MAX_MOVEMENT_INTERVAL.as_secs_f64() {
            return Err(MovementIntervalError::AboveMaximum {
                maximum_seconds: MAX_MOVEMENT_INTERVAL.as_secs(),
            });
        }

        let duration = Duration::try_from_secs_f64(seconds)?;
        if duration < MIN_MOVEMENT_INTERVAL {
            return Err(MovementIntervalError::BelowMinimum {
                minimum_milliseconds: MIN_MOVEMENT_INTERVAL.as_millis(),
            });
        }

        Ok(Self(duration))
    }
}

impl Default for MovementInterval {
    fn default() -> Self {
        Self(Duration::from_secs(10))
    }
}

impl Display for MovementInterval {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.duration().as_secs_f64())
    }
}

impl FromStr for MovementInterval {
    type Err = MovementIntervalError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let seconds = value.parse::<f64>()?;
        Self::try_from(seconds)
    }
}

#[derive(Debug, Error)]
enum MovementIntervalError {
    #[error("interval must be a number of seconds")]
    Parse(#[from] ParseFloatError),
    #[error("interval must be finite")]
    NotFinite,
    #[error("interval must be greater than 0 seconds")]
    NotPositive,
    #[error("interval must be at least {minimum_milliseconds} milliseconds")]
    BelowMinimum { minimum_milliseconds: u128 },
    #[error("interval must not exceed {maximum_seconds} seconds")]
    AboveMaximum { maximum_seconds: u64 },
    #[error("interval cannot be represented as a duration")]
    NotRepresentable(#[from] TryFromFloatSecsError),
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum TrajectoryArg {
    #[default]
    Linear,
    Circle,
    Star,
    Square,
    #[value(alias = "figure8")]
    Infinity,
}

impl Display for TrajectoryArg {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Linear => "linear",
            Self::Circle => "circle",
            Self::Star => "star",
            Self::Square => "square",
            Self::Infinity => "infinity",
        };
        formatter.write_str(name)
    }
}

impl From<TrajectoryArg> for TrajectoryType {
    fn from(value: TrajectoryArg) -> Self {
        match value {
            TrajectoryArg::Linear => Self::Linear,
            TrajectoryArg::Circle => Self::Circle,
            TrajectoryArg::Star => Self::Star,
            TrajectoryArg::Square => Self::Square,
            TrajectoryArg::Infinity => Self::Infinity,
        }
    }
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
        default_value_t = TrajectoryExtent::default(),
        allow_hyphen_values = true,
        help = "Maximum width of the trajectory pattern (pixels; positive integer; star and infinity require size >= 2)"
    )]
    size: TrajectoryExtent,

    #[arg(
        short,
        long,
        default_value_t = MovementInterval::default(),
        allow_hyphen_values = true,
        help = "Time from one successful mouse movement to the next (seconds; >= 0.02, <= 31536000)"
    )]
    interval: MovementInterval,

    #[arg(
        short,
        long,
        value_enum,
        default_value_t = TrajectoryArg::default(),
        help = "Trajectory pattern to use"
    )]
    trajectory: TrajectoryArg,

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
    fn execute(&self, cmd: &mut Command) -> anyhow::Result<()> {
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

trait Clock {
    fn now(&self) -> Instant;
}

trait Sleeper {
    fn sleep(&self, duration: Duration);
}

struct MonotonicClock;

impl Clock for MonotonicClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

struct ThreadSleeper;

impl Sleeper for ThreadSleeper {
    fn sleep(&self, duration: Duration) {
        thread::sleep(duration);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaitOutcome {
    DeadlineReached,
    Terminated,
}

#[derive(Debug, Error)]
enum WaitError {
    #[error(
        "cannot schedule the next movement: adding interval {interval:?} exceeds the monotonic clock range"
    )]
    DeadlineOverflow { interval: Duration },
}

impl Quaker {
    fn new(enigo: Enigo, trajectory: Box<dyn Trajectory>) -> Self {
        Quaker { enigo, trajectory }
    }

    fn quake(&mut self) -> Result<(), InputError> {
        let displacement = self.trajectory.next();
        let (x, y) = displacement.components();
        self.enigo.move_mouse(x, y, Rel)?;
        Ok(())
    }
}

fn wait_for_next_movement<C, S, F>(
    successful_movement_at: Instant,
    interval: MovementInterval,
    clock: &C,
    sleeper: &S,
    mut should_terminate: F,
) -> Result<WaitOutcome, WaitError>
where
    C: Clock,
    S: Sleeper,
    F: FnMut() -> bool,
{
    let duration = interval.duration();
    let deadline = successful_movement_at
        .checked_add(duration)
        .ok_or(WaitError::DeadlineOverflow { interval: duration })?;

    loop {
        if should_terminate() {
            return Ok(WaitOutcome::Terminated);
        }

        let remaining = deadline.saturating_duration_since(clock.now());
        if remaining.is_zero() {
            return Ok(WaitOutcome::DeadlineReached);
        }

        sleeper.sleep(remaining.min(TERMINATION_CHECK_INTERVAL));
    }
}

fn execute_quaker(
    trajectory_spec: TrajectorySpec,
    interval: MovementInterval,
) -> anyhow::Result<()> {
    let trajectory = trajectory_spec.into_trajectory();
    let enigo =
        Enigo::new(&Settings::default()).context("failed to initialize mouse input backend")?;
    let mut quaker = Quaker::new(enigo, trajectory);
    let term = Arc::new(AtomicBool::new(false));
    let clock = MonotonicClock;
    let sleeper = ThreadSleeper;

    for sig in TERM_SIGNALS {
        flag::register(*sig, Arc::clone(&term))
            .with_context(|| format!("failed to register termination signal {sig}"))?;
    }

    while !term.load(Ordering::Relaxed) {
        quaker.quake().context("failed to move the mouse pointer")?;
        let successful_movement_at = clock.now();

        if wait_for_next_movement(successful_movement_at, interval, &clock, &sleeper, || {
            term.load(Ordering::Relaxed)
        })? == WaitOutcome::Terminated
        {
            break;
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
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

    let trajectory_spec =
        TrajectorySpec::try_new(trajectory.into(), size).unwrap_or_else(|error| {
            Cli::command()
                .error(ErrorKind::ValueValidation, error)
                .exit()
        });
    execute_quaker(trajectory_spec, interval)
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use clap::Parser;

    use super::*;

    struct FakeTime {
        now: Cell<Instant>,
        sleeps: RefCell<Vec<Duration>>,
    }

    impl FakeTime {
        fn new() -> Self {
            Self {
                now: Cell::new(Instant::now()),
                sleeps: RefCell::new(Vec::new()),
            }
        }

        fn advance(&self, duration: Duration) {
            self.now.set(self.now.get().checked_add(duration).unwrap());
        }
    }

    impl Clock for FakeTime {
        fn now(&self) -> Instant {
            self.now.get()
        }
    }

    impl Sleeper for FakeTime {
        fn sleep(&self, duration: Duration) {
            self.sleeps.borrow_mut().push(duration);
            self.advance(duration);
        }
    }

    #[test]
    fn test_cli_default_values() {
        let cli = Cli::parse_from(["mousequake"]);
        assert_eq!(cli.size, TrajectoryExtent::try_from(1).unwrap());
        assert_eq!(cli.interval.duration(), Duration::from_secs(10));
        assert!(matches!(cli.trajectory, TrajectoryArg::Linear));
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_custom_size() {
        let cli = Cli::parse_from(["mousequake", "-s", "5"]);
        assert_eq!(cli.size, TrajectoryExtent::try_from(5).unwrap());
        assert_eq!(cli.interval.duration(), Duration::from_secs(10));
    }

    #[test]
    fn test_cli_rejects_non_positive_sizes() {
        for size in ["0", "-1"] {
            let result = Cli::try_parse_from(["mousequake", "--size", size]);
            assert!(result.is_err(), "size {size:?} should be rejected");
        }
    }

    #[test]
    fn test_cli_custom_interval() {
        let cli = Cli::parse_from(["mousequake", "-i", "30.5"]);
        assert_eq!(cli.size, TrajectoryExtent::try_from(1).unwrap());
        assert_eq!(cli.interval.duration(), Duration::from_millis(30_500));
    }

    #[test]
    fn test_cli_rejects_invalid_intervals() {
        for interval in ["0", "-0", "-1", "NaN", "inf", "+inf", "-inf"] {
            let result = Cli::try_parse_from(["mousequake", "--interval", interval]);
            assert!(result.is_err(), "interval {interval:?} should be rejected");
        }
    }

    #[test]
    fn test_cli_rejects_intervals_outside_supported_range() {
        for interval in ["0.0000000001", "0.001", "0.019999999", "31536000.00000001"] {
            let result = Cli::try_parse_from(["mousequake", "--interval", interval]);
            assert!(result.is_err(), "interval {interval:?} should be rejected");
        }
    }

    #[test]
    fn test_movement_interval_preserves_error_categories() {
        assert!(matches!(
            "not-a-number".parse::<MovementInterval>(),
            Err(MovementIntervalError::Parse(_))
        ));
        assert!(matches!(
            MovementInterval::try_from(f64::NAN),
            Err(MovementIntervalError::NotFinite)
        ));
        assert!(matches!(
            MovementInterval::try_from(-0.0),
            Err(MovementIntervalError::NotPositive)
        ));
        assert!(matches!(
            MovementInterval::try_from(0.019),
            Err(MovementIntervalError::BelowMinimum {
                minimum_milliseconds: 20
            })
        ));
        assert!(matches!(
            MovementInterval::try_from(31_536_001.0),
            Err(MovementIntervalError::AboveMaximum {
                maximum_seconds: 31_536_000
            })
        ));
    }

    #[test]
    fn test_cli_accepts_valid_intervals() {
        for interval in ["0.02", "0.1", "0.6", "10.1", "8388609", "31536000"] {
            let result = Cli::try_parse_from(["mousequake", "--interval", interval]);
            assert!(result.is_ok(), "interval {interval:?} should be accepted");
        }
    }

    #[test]
    fn test_wait_uses_exact_remaining_duration_without_real_sleeping() {
        for (interval, expected_duration, expected_sleep_count) in [
            ("0.1", Duration::from_millis(100), 1),
            ("0.6", Duration::from_millis(600), 2),
            ("10.1", Duration::from_millis(10_100), 21),
        ] {
            let fake_time = FakeTime::new();
            let successful_movement_at = fake_time.now();
            let movement_interval = interval.parse::<MovementInterval>().unwrap();

            let outcome = wait_for_next_movement(
                successful_movement_at,
                movement_interval,
                &fake_time,
                &fake_time,
                || false,
            )
            .unwrap();

            let sleeps = fake_time.sleeps.borrow();
            let total_sleep = sleeps.iter().copied().sum::<Duration>();
            let (last_sleep, full_chunks) = sleeps.split_last().unwrap();
            assert_eq!(outcome, WaitOutcome::DeadlineReached);
            assert_eq!(movement_interval.duration(), expected_duration);
            assert_eq!(total_sleep, expected_duration);
            assert_eq!(sleeps.len(), expected_sleep_count);
            assert_eq!(*last_sleep, Duration::from_millis(100));
            assert!(
                full_chunks
                    .iter()
                    .all(|duration| *duration == TERMINATION_CHECK_INTERVAL)
            );
        }
    }

    #[test]
    fn test_wait_progresses_near_large_interval_deadline() {
        let fake_time = FakeTime::new();
        let successful_movement_at = fake_time.now();
        let interval = "8388609".parse::<MovementInterval>().unwrap();
        fake_time.advance(Duration::from_secs(8_388_608));

        let outcome = wait_for_next_movement(
            successful_movement_at,
            interval,
            &fake_time,
            &fake_time,
            || false,
        )
        .unwrap();

        assert_eq!(outcome, WaitOutcome::DeadlineReached);
        assert_eq!(
            fake_time.sleeps.borrow().as_slice(),
            [Duration::from_millis(500), Duration::from_millis(500)]
        );
    }

    #[test]
    fn test_wait_stops_when_termination_is_requested() {
        let fake_time = FakeTime::new();
        let successful_movement_at = fake_time.now();
        let interval = "10.1".parse::<MovementInterval>().unwrap();

        let outcome = wait_for_next_movement(
            successful_movement_at,
            interval,
            &fake_time,
            &fake_time,
            || !fake_time.sleeps.borrow().is_empty(),
        )
        .unwrap();

        assert_eq!(outcome, WaitOutcome::Terminated);
        assert_eq!(
            fake_time.sleeps.borrow().as_slice(),
            [Duration::from_millis(500)]
        );
    }

    #[test]
    fn test_cli_trajectory_types() {
        let cli = Cli::parse_from(["mousequake", "-t", "circle"]);
        assert!(matches!(cli.trajectory, TrajectoryArg::Circle));

        let cli = Cli::parse_from(["mousequake", "-t", "star"]);
        assert!(matches!(cli.trajectory, TrajectoryArg::Star));

        let cli = Cli::parse_from(["mousequake", "-t", "figure8"]);
        assert!(matches!(cli.trajectory, TrajectoryArg::Infinity));
    }

    #[test]
    fn test_cli_completion_subcommand() {
        let cli = Cli::parse_from(["mousequake", "completion", "bash"]);
        assert!(matches!(cli.command, Some(Subcommand::Completion(_))));
    }
}
