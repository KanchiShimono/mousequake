use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

pub trait Trajectory: Send {
    fn next(&mut self) -> Point;
}

pub struct LinearTrajectory {
    current_x: f32,
}

impl LinearTrajectory {
    pub fn new(size: f32) -> Self {
        LinearTrajectory {
            current_x: size / 2.0,
        }
    }
}

impl Trajectory for LinearTrajectory {
    fn next(&mut self) -> Point {
        let point = Point::new(self.current_x, 0.0);
        self.current_x = -self.current_x;
        point
    }
}

pub struct CircleTrajectory {
    radius: f32,
    steps: usize,
    current_step: usize,
}

impl CircleTrajectory {
    pub fn new(size: f32) -> Self {
        CircleTrajectory {
            radius: size / 2.0,
            steps: 36,
            current_step: 0,
        }
    }
}

impl Trajectory for CircleTrajectory {
    fn next(&mut self) -> Point {
        let angle = 2.0 * PI * (self.current_step as f32) / (self.steps as f32);
        let next_angle = 2.0 * PI * ((self.current_step + 1) as f32) / (self.steps as f32);

        let current_x = self.radius * angle.cos();
        let current_y = self.radius * angle.sin();
        let next_x = self.radius * next_angle.cos();
        let next_y = self.radius * next_angle.sin();

        let dx = next_x - current_x;
        let dy = next_y - current_y;

        self.current_step = (self.current_step + 1) % self.steps;

        Point::new(dx, dy)
    }
}

pub struct StarTrajectory {
    points: Vec<(f32, f32)>,
    current_index: usize,
}

impl StarTrajectory {
    pub fn new(size: f32) -> Self {
        let mut points = Vec::new();
        let outer_radius = size / 2.0;
        let inner_radius = outer_radius * 0.4;

        for i in 0..10 {
            let angle = PI * (i as f32) / 5.0 - PI / 2.0;
            let radius = if i % 2 == 0 {
                outer_radius
            } else {
                inner_radius
            };
            points.push((radius * angle.cos(), radius * angle.sin()));
        }

        StarTrajectory {
            points,
            current_index: 0,
        }
    }
}

impl Trajectory for StarTrajectory {
    fn next(&mut self) -> Point {
        let current = self.points[self.current_index];
        let next_index = (self.current_index + 1) % self.points.len();
        let next = self.points[next_index];

        let dx = next.0 - current.0;
        let dy = next.1 - current.1;

        self.current_index = next_index;

        Point::new(dx, dy)
    }
}

pub struct SquareTrajectory {
    vertices: Vec<(f32, f32)>,
    current_vertex: usize,
}

impl SquareTrajectory {
    pub fn new(size: f32) -> Self {
        let half = size / 2.0;
        let vertices = vec![(half, half), (-half, half), (-half, -half), (half, -half)];

        SquareTrajectory {
            vertices,
            current_vertex: 0,
        }
    }
}

impl Trajectory for SquareTrajectory {
    fn next(&mut self) -> Point {
        let current = self.vertices[self.current_vertex];
        let next_vertex = (self.current_vertex + 1) % self.vertices.len();
        let next = self.vertices[next_vertex];

        let dx = next.0 - current.0;
        let dy = next.1 - current.1;

        self.current_vertex = next_vertex;

        Point::new(dx, dy)
    }
}

pub struct InfinityTrajectory {
    amplitude: f32,
    steps: usize,
    current_step: usize,
}

impl InfinityTrajectory {
    pub fn new(size: f32) -> Self {
        InfinityTrajectory {
            amplitude: size / 2.0,
            steps: 36,
            current_step: 0,
        }
    }
}

impl Trajectory for InfinityTrajectory {
    fn next(&mut self) -> Point {
        let t = 2.0 * PI * (self.current_step as f32) / (self.steps as f32);
        let next_t = 2.0 * PI * ((self.current_step + 1) as f32) / (self.steps as f32);

        let current_x = self.amplitude * t.sin();
        let current_y = self.amplitude * (2.0 * t).sin() / 2.0;

        let next_x = self.amplitude * next_t.sin();
        let next_y = self.amplitude * (2.0 * next_t).sin() / 2.0;

        let dx = next_x - current_x;
        let dy = next_y - current_y;

        self.current_step = (self.current_step + 1) % self.steps;

        Point::new(dx, dy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_trajectory() {
        let mut trajectory = LinearTrajectory::new(10.0);

        let p1 = trajectory.next();
        assert_eq!(p1.x, 5.0);
        assert_eq!(p1.y, 0.0);

        let p2 = trajectory.next();
        assert_eq!(p2.x, -5.0);
        assert_eq!(p2.y, 0.0);

        let p3 = trajectory.next();
        assert_eq!(p3.x, 5.0);
        assert_eq!(p3.y, 0.0);
    }

    #[test]
    fn test_circle_trajectory() {
        let mut trajectory = CircleTrajectory::new(10.0);
        let mut distances = Vec::new();
        let mut first_points = Vec::new();

        // Collect first cycle of movements
        for _ in 0..36 {
            let point = trajectory.next();
            let distance = (point.x * point.x + point.y * point.y).sqrt();
            distances.push(distance);
            first_points.push((point.x, point.y));
        }

        // All movement distances should be similar (within 10% tolerance)
        let avg_distance = distances.iter().sum::<f32>() / distances.len() as f32;
        for distance in &distances {
            assert!(
                (distance - avg_distance).abs() / avg_distance < 0.1,
                "Movement distances should be uniform in a circle"
            );
        }

        // Second cycle should match the first (periodic behavior)
        for i in 0..36 {
            let point = trajectory.next();
            assert!(
                (point.x - first_points[i].0).abs() < 0.01
                    && (point.y - first_points[i].1).abs() < 0.01,
                "Circle trajectory should be periodic"
            );
        }
    }

    #[test]
    fn test_star_trajectory() {
        let mut trajectory = StarTrajectory::new(20.0);
        let mut first_cycle = Vec::new();
        let mut position_x = 0.0;
        let mut position_y = 0.0;
        let mut radial_distances = Vec::new();

        // Collect one complete star cycle
        for _ in 0..10 {
            let point = trajectory.next();
            position_x += point.x;
            position_y += point.y;
            let radial_distance = (position_x * position_x + position_y * position_y).sqrt();
            radial_distances.push(radial_distance);
            first_cycle.push((point.x, point.y));
        }

        // Should return to origin after one complete star
        assert!(
            position_x.abs() < 0.1 && position_y.abs() < 0.1,
            "Star should form a closed path"
        );

        // Star should visit both outer and inner radii
        let max_radius = radial_distances
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let min_radius = radial_distances
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        assert!(
            *max_radius > *min_radius * 2.0,
            "Star should have distinct outer and inner vertices"
        );

        // Second cycle should match the first (periodic behavior)
        for i in 0..10 {
            let point = trajectory.next();
            assert!(
                (point.x - first_cycle[i].0).abs() < 0.01
                    && (point.y - first_cycle[i].1).abs() < 0.01,
                "Star trajectory should be periodic"
            );
        }
    }

    #[test]
    fn test_square_trajectory() {
        let mut trajectory = SquareTrajectory::new(10.0);

        let p1 = trajectory.next();
        assert_eq!(p1.x, -10.0);
        assert_eq!(p1.y, 0.0);

        let p2 = trajectory.next();
        assert_eq!(p2.x, 0.0);
        assert_eq!(p2.y, -10.0);

        let p3 = trajectory.next();
        assert_eq!(p3.x, 10.0);
        assert_eq!(p3.y, 0.0);

        let p4 = trajectory.next();
        assert_eq!(p4.x, 0.0);
        assert_eq!(p4.y, 10.0);

        let p5 = trajectory.next();
        assert_eq!(p5.x, -10.0);
        assert_eq!(p5.y, 0.0);
    }

    #[test]
    fn test_infinity_trajectory() {
        let mut trajectory = InfinityTrajectory::new(10.0);
        let mut x_positions = Vec::new();
        let mut first_cycle = Vec::new();
        let mut current_x = 0.0;

        // Track x positions through one cycle
        for _ in 0..36 {
            let point = trajectory.next();
            current_x += point.x;
            x_positions.push(current_x);
            first_cycle.push((point.x, point.y));
        }

        // X position should oscillate (go positive, then negative, then back)
        let max_x = x_positions
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let min_x = x_positions
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        assert!(
            *max_x > 2.0 && *min_x < -2.0,
            "Infinity pattern should oscillate in x direction"
        );

        // Should cross near center at least twice in a cycle
        let mut center_crossings = 0;
        for &x in &x_positions {
            if x.abs() < 0.5 {
                center_crossings += 1;
            }
        }
        assert!(
            center_crossings >= 2,
            "Infinity pattern should cross center multiple times"
        );

        // Second cycle should match the first (periodic behavior)
        for i in 0..36 {
            let point = trajectory.next();
            assert!(
                (point.x - first_cycle[i].0).abs() < 0.01
                    && (point.y - first_cycle[i].1).abs() < 0.01,
                "Infinity trajectory should be periodic"
            );
        }
    }
}
