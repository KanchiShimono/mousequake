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
