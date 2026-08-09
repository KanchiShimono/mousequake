use std::f64::consts::PI;
use std::fmt::{self, Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;

use thiserror::Error;

const PARAMETRIC_MAX_STEPS: usize = 40;
const PARAMETRIC_MIN_STEPS: usize = 8;
const STAR_INNER_RADIUS_RATIO: f64 = 0.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Displacement {
    x: i32,
    y: i32,
}

impl Displacement {
    fn between(current: Point, next: Point) -> Self {
        Self {
            x: next.x - current.x,
            y: next.y - current.y,
        }
    }

    pub(crate) fn components(self) -> (i32, i32) {
        (self.x, self.y)
    }
}

pub(crate) trait Trajectory: Send {
    fn next(&mut self) -> Displacement;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TrajectoryExtent(i32);

impl TrajectoryExtent {
    fn get(self) -> i32 {
        self.0
    }
}

impl Default for TrajectoryExtent {
    fn default() -> Self {
        Self(1)
    }
}

impl Display for TrajectoryExtent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.get())
    }
}

impl TryFrom<i32> for TrajectoryExtent {
    type Error = TrajectoryExtentError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value <= 0 {
            return Err(TrajectoryExtentError::NotPositive);
        }

        Ok(Self(value))
    }
}

impl FromStr for TrajectoryExtent {
    type Err = TrajectoryExtentError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value.parse::<i32>()?)
    }
}

#[derive(Debug, Error)]
pub(crate) enum TrajectoryExtentError {
    #[error("size must be an integer number of pixels")]
    Parse(#[from] ParseIntError),
    #[error("size must be greater than 0 pixels")]
    NotPositive,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum TrajectoryType {
    #[default]
    Linear,
    Circle,
    Star,
    Square,
    Infinity,
}

impl Display for TrajectoryType {
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

#[derive(Debug)]
pub(crate) struct TrajectorySpec {
    displacements: Vec<Displacement>,
}

impl TrajectorySpec {
    pub(crate) fn try_new(
        trajectory_type: TrajectoryType,
        extent: TrajectoryExtent,
    ) -> Result<Self, TrajectorySpecError> {
        let minimum_extent = trajectory_type.minimum_extent();
        if extent.get() < minimum_extent {
            return Err(TrajectorySpecError::Unrepresentable {
                trajectory_type,
                extent,
                minimum_extent,
            });
        }

        let points = match trajectory_type {
            TrajectoryType::Linear => linear_points(extent),
            TrajectoryType::Circle => circle_points(extent),
            TrajectoryType::Star => star_points(extent),
            TrajectoryType::Square => square_points(extent),
            TrajectoryType::Infinity => infinity_points(extent),
        };
        let displacements =
            cycle_displacements(points).ok_or(TrajectorySpecError::Unrepresentable {
                trajectory_type,
                extent,
                minimum_extent,
            })?;

        Ok(Self { displacements })
    }

    pub(crate) fn into_trajectory(self) -> Box<dyn Trajectory> {
        Box::new(self.into_cyclic_trajectory())
    }

    fn into_cyclic_trajectory(self) -> CyclicTrajectory {
        CyclicTrajectory {
            displacements: self.displacements,
            current_step: 0,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum TrajectorySpecError {
    #[error(
        "{trajectory_type} trajectory cannot represent size {extent}; minimum supported size is {minimum_extent} pixels"
    )]
    Unrepresentable {
        trajectory_type: TrajectoryType,
        extent: TrajectoryExtent,
        minimum_extent: i32,
    },
}

impl TrajectoryType {
    fn minimum_extent(self) -> i32 {
        match self {
            Self::Star | Self::Infinity => 2,
            Self::Linear | Self::Circle | Self::Square => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CyclicTrajectory {
    displacements: Vec<Displacement>,
    current_step: usize,
}

impl Trajectory for CyclicTrajectory {
    fn next(&mut self) -> Displacement {
        let displacement = self.displacements[self.current_step];
        self.current_step = (self.current_step + 1) % self.displacements.len();
        displacement
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

fn linear_points(extent: TrajectoryExtent) -> Vec<Point> {
    let negative_extent = extent.get() / 2;
    let positive_extent = extent.get() - negative_extent;

    vec![
        Point::new(0, 0),
        Point::new(positive_extent, 0),
        Point::new(-negative_extent, 0),
    ]
}

fn circle_points(extent: TrajectoryExtent) -> Vec<Point> {
    let size = f64::from(extent.get());
    let radius = size / 2.0;
    let steps = parametric_steps(extent);

    (0..steps)
        .map(|step| {
            let angle = 2.0 * PI * step as f64 / steps as f64;
            quantized_point(
                radius + radius * angle.cos(),
                radius + radius * angle.sin(),
                extent,
            )
        })
        .collect()
}

fn star_points(extent: TrajectoryExtent) -> Vec<Point> {
    let size = f64::from(extent.get());
    let outer_radius = size / (2.0 * (PI / 10.0).cos());

    (0..10)
        .map(|index| {
            let angle = PI * index as f64 / 5.0 - PI / 2.0;
            let radius = if index % 2 == 0 {
                outer_radius
            } else {
                outer_radius * STAR_INNER_RADIUS_RATIO
            };
            quantized_point(
                size / 2.0 + radius * angle.cos(),
                outer_radius + radius * angle.sin(),
                extent,
            )
        })
        .collect()
}

fn square_points(extent: TrajectoryExtent) -> Vec<Point> {
    let size = extent.get();
    vec![
        Point::new(size, size),
        Point::new(0, size),
        Point::new(0, 0),
        Point::new(size, 0),
    ]
}

fn infinity_points(extent: TrajectoryExtent) -> Vec<Point> {
    let size = f64::from(extent.get());
    let horizontal_radius = size / 2.0;
    let vertical_radius = size / 4.0;
    let steps = parametric_steps(extent);

    (0..steps)
        .map(|step| {
            let angle = 2.0 * PI * step as f64 / steps as f64;
            quantized_point(
                horizontal_radius + horizontal_radius * angle.sin(),
                vertical_radius + vertical_radius * (2.0 * angle).sin(),
                extent,
            )
        })
        .collect()
}

fn parametric_steps(extent: TrajectoryExtent) -> usize {
    let adaptive_steps = match extent.get() {
        1..=2 => 8,
        3..=4 => 16,
        5..=6 => 24,
        7..=8 => 32,
        _ => PARAMETRIC_MAX_STEPS,
    };
    adaptive_steps.clamp(PARAMETRIC_MIN_STEPS, PARAMETRIC_MAX_STEPS)
}

fn quantized_point(x: f64, y: f64, extent: TrajectoryExtent) -> Point {
    let maximum = f64::from(extent.get());
    Point::new(
        x.clamp(0.0, maximum).round() as i32,
        y.clamp(0.0, maximum).round() as i32,
    )
}

fn cycle_displacements(points: Vec<Point>) -> Option<Vec<Displacement>> {
    let mut distinct_points = Vec::with_capacity(points.len());
    for point in points {
        if distinct_points.last() != Some(&point) {
            distinct_points.push(point);
        }
    }
    if distinct_points.len() > 1 && distinct_points.first() == distinct_points.last() {
        distinct_points.pop();
    }
    if distinct_points.len() < 2 {
        return None;
    }

    let mut displacements = Vec::with_capacity(distinct_points.len());
    for points in distinct_points.windows(2) {
        displacements.push(Displacement::between(points[0], points[1]));
    }
    displacements.push(Displacement::between(
        *distinct_points.last()?,
        distinct_points[0],
    ));
    Some(displacements)
}

#[cfg(test)]
mod tests {
    use super::{
        Displacement, PI, Trajectory, TrajectoryExtent, TrajectoryExtentError, TrajectorySpec,
        TrajectorySpecError, TrajectoryType,
    };

    struct BoundingBox {
        width: i32,
        height: i32,
    }

    fn extent(value: i32) -> TrajectoryExtent {
        TrajectoryExtent::try_from(value).unwrap()
    }

    fn cycle(
        trajectory_type: TrajectoryType,
        trajectory_extent: TrajectoryExtent,
    ) -> Vec<Displacement> {
        let spec = TrajectorySpec::try_new(trajectory_type, trajectory_extent).unwrap();
        let period = spec.displacements.len();
        let mut trajectory = spec.into_trajectory();
        (0..period).map(|_| trajectory.next()).collect()
    }

    fn bounding_box(displacements: &[Displacement]) -> BoundingBox {
        let (mut x, mut y) = (0, 0);
        let (mut minimum_x, mut maximum_x) = (0, 0);
        let (mut minimum_y, mut maximum_y) = (0, 0);

        for displacement in displacements {
            x += displacement.x;
            y += displacement.y;
            minimum_x = minimum_x.min(x);
            maximum_x = maximum_x.max(x);
            minimum_y = minimum_y.min(y);
            maximum_y = maximum_y.max(y);
        }

        BoundingBox {
            width: maximum_x - minimum_x,
            height: maximum_y - minimum_y,
        }
    }

    fn assert_closed(trajectory_type: TrajectoryType, size: i32, displacements: &[Displacement]) {
        let sum = displacements.iter().fold((0_i64, 0_i64), |sum, point| {
            (sum.0 + i64::from(point.x), sum.1 + i64::from(point.y))
        });
        assert_eq!(
            sum,
            (0, 0),
            "{trajectory_type} size {size} did not form a closed cycle"
        );
    }

    fn assert_bounding_box_near(
        trajectory_type: TrajectoryType,
        size: i32,
        ideal_width: f64,
        ideal_height: f64,
    ) {
        let bounds = bounding_box(&cycle(trajectory_type, extent(size)));
        assert!(
            (f64::from(bounds.width) - ideal_width).abs() <= 1.0,
            "{trajectory_type} size {size} had width {} instead of approximately {ideal_width}",
            bounds.width
        );
        assert!(
            (f64::from(bounds.height) - ideal_height).abs() <= 1.0,
            "{trajectory_type} size {size} had height {} instead of approximately {ideal_height}",
            bounds.height
        );
    }

    #[test]
    fn test_extent_accepts_only_positive_integers() {
        assert_eq!(TrajectoryExtent::try_from(1).unwrap().get(), 1);
        for (value, expected) in [("1", 1), ("10", 10), ("2147483647", i32::MAX)] {
            assert_eq!(value.parse::<TrajectoryExtent>().unwrap().get(), expected);
        }
        assert!(matches!(
            TrajectoryExtent::try_from(0),
            Err(TrajectoryExtentError::NotPositive)
        ));
        assert!(matches!(
            TrajectoryExtent::try_from(-1),
            Err(TrajectoryExtentError::NotPositive)
        ));
        assert!(matches!(
            "1.5".parse::<TrajectoryExtent>(),
            Err(TrajectoryExtentError::Parse(_))
        ));
        assert!(matches!(
            "2147483648".parse::<TrajectoryExtent>(),
            Err(TrajectoryExtentError::Parse(_))
        ));
    }

    #[test]
    fn test_linear_trajectory() {
        let trajectory_type = TrajectoryType::default();
        assert_eq!(trajectory_type, TrajectoryType::Linear);

        for (trajectory_extent, expected_minimum_x, expected_maximum_x) in [
            (TrajectoryExtent::default(), 0, 1),
            (extent(2), -1, 1),
            (extent(3), -1, 2),
            (extent(10), -5, 5),
        ] {
            let size = trajectory_extent.get();
            let displacements = cycle(trajectory_type, trajectory_extent);
            assert!(
                displacements
                    .iter()
                    .all(|point| point.x != 0 || point.y != 0),
                "{trajectory_type} size {size} contained a zero movement"
            );

            let mut x = 0;
            let mut positions = vec![x];
            for displacement in &displacements {
                x += displacement.x;
                positions.push(x);
            }

            assert_eq!(positions.iter().copied().min(), Some(expected_minimum_x));
            assert_eq!(positions.iter().copied().max(), Some(expected_maximum_x));
            assert_closed(trajectory_type, size, &displacements);
        }
    }

    #[test]
    fn test_circle_trajectory() {
        assert_bounding_box_near(TrajectoryType::Circle, 10, 10.0, 10.0);
    }

    #[test]
    fn test_star_trajectory() {
        let star_size = 20.0;
        let star_height = star_size * (1.0 + (PI / 5.0).cos()) / (2.0 * (PI / 10.0).cos());
        assert_bounding_box_near(TrajectoryType::Star, 20, star_size, star_height);
    }

    #[test]
    fn test_square_trajectory() {
        assert_bounding_box_near(TrajectoryType::Square, 10, 10.0, 10.0);
    }

    #[test]
    fn test_infinity_trajectory() {
        assert_bounding_box_near(TrajectoryType::Infinity, 15, 15.0, 7.5);
    }

    #[test]
    fn test_pattern_support_boundaries() {
        for trajectory_type in [
            TrajectoryType::Linear,
            TrajectoryType::Circle,
            TrajectoryType::Square,
        ] {
            assert!(TrajectorySpec::try_new(trajectory_type, extent(1)).is_ok());
        }

        for trajectory_type in [TrajectoryType::Star, TrajectoryType::Infinity] {
            assert!(matches!(
                TrajectorySpec::try_new(trajectory_type, extent(1)),
                Err(TrajectorySpecError::Unrepresentable {
                    minimum_extent: 2,
                    ..
                })
            ));
            assert!(TrajectorySpec::try_new(trajectory_type, extent(2)).is_ok());
        }
    }

    #[test]
    fn test_maximum_extent_is_supported_without_overflow() {
        for trajectory_type in [
            TrajectoryType::Linear,
            TrajectoryType::Circle,
            TrajectoryType::Star,
            TrajectoryType::Square,
            TrajectoryType::Infinity,
        ] {
            let displacements = cycle(trajectory_type, extent(i32::MAX));
            assert!(
                displacements
                    .iter()
                    .all(|point| point.x != 0 || point.y != 0),
                "{trajectory_type} size {} contained a zero movement",
                i32::MAX
            );
            assert_closed(trajectory_type, i32::MAX, &displacements);
        }
    }

    #[test]
    fn test_small_supported_integer_cycles_are_nonzero_closed_and_periodic() {
        for (trajectory_type, minimum_size) in [
            (TrajectoryType::Linear, 1),
            (TrajectoryType::Circle, 1),
            (TrajectoryType::Star, 2),
            (TrajectoryType::Square, 1),
            (TrajectoryType::Infinity, 2),
        ] {
            for size in minimum_size..=512 {
                let spec = TrajectorySpec::try_new(trajectory_type, extent(size)).unwrap();
                let period = spec.displacements.len();
                let mut trajectory = spec.into_cyclic_trajectory();
                let initial_state = trajectory.clone();
                let displacements: Vec<_> = (0..period).map(|_| trajectory.next()).collect();

                assert!(
                    displacements
                        .iter()
                        .all(|point| point.x != 0 || point.y != 0),
                    "{trajectory_type} size {size} contained a zero movement"
                );
                assert_closed(trajectory_type, size, &displacements);
                assert_eq!(
                    trajectory, initial_state,
                    "{trajectory_type} size {size} did not restore its initial state"
                );
                assert_eq!(
                    (0..period).map(|_| trajectory.next()).collect::<Vec<_>>(),
                    displacements,
                    "{trajectory_type} size {size} did not repeat its displacement cycle"
                );
            }
        }
    }
}
