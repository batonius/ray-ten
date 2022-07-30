use crate::math::{Axis, Color, Point, Real, Vector};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Plane {
    Top,
    Bottom,
    Left,
    Right,
    Far,
    Near,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Sphere {
    Ball,
    NearPaddle,
    FarPaddle,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Obstacle {
    Plane(Plane),
    Sphere(Sphere),
}

const PLANES_COUNT: usize = Plane::Near as usize + 1;

const PLANES_OFFSETS: [Real; PLANES_COUNT] = [2.0, -2.0, -4.0, 4.0, -16.0, 0.0];
const PLANES_NORMALS: [Vector; PLANES_COUNT] = [
    Vector::new(0.0, -1.0, 0.0),
    Vector::new(0.0, 1.0, 0.0),
    Vector::new(1.0, 0.0, 0.0),
    Vector::new(-1.0, 0.0, 0.0),
    Vector::new(0.0, 0.0, 1.0),
    Vector::new(0.0, 0.0, -1.0),
];
const PLANES_AXIS: [Axis; PLANES_COUNT] =
    [Axis::YS, Axis::YS, Axis::XS, Axis::XS, Axis::ZS, Axis::ZS];
const PLANES_COLORS: [Color; PLANES_COUNT] = [
    Color::new(0.8, 0.8, 0.1),
    Color::new(0.1, 0.8, 0.8),
    Color::new(0.8, 0.1, 0.8),
    Color::new(0.8, 0.1, 0.1),
    Color::new(0.1, 0.1, 0.8),
    Color::new(0.1, 0.8, 0.1),
];
const PLANES_REFLECTANCE: [Real; PLANES_COUNT] = [0.3, 0.3, 0.3, 0.3, 0.3, 0.3];

const SPHERES_COUNT: usize = Sphere::FarPaddle as usize + 1;
const SPHERES_RADII: [Real; SPHERES_COUNT] = [0.5, 4.0, 4.0];
const SPHERES_COLORS: [Color; SPHERES_COUNT] = [
    Color::new(0.1, 0.1, 0.1),
    Color::new(1.0, 1.0, 1.0),
    Color::new(0.0, 0.0, 0.0),
];
const SPHERES_REFLECTANCE: [Real; SPHERES_COUNT] = [0.5, 1.0, 1.0];

pub struct Scene {
    ball_pos: Point,
    near_paddle_pos: Point,
    far_paddle_pos: Point,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            ball_pos: Point::new(-2.0, -1.0, -6.0),
            near_paddle_pos: Point::new(0.0, 0.0, 3.87),
            far_paddle_pos: Point::new(0.0, 0.0, -19.87),
        }
    }

    pub fn plane_offset(&self, plane: Plane) -> Real {
        PLANES_OFFSETS[plane as usize]
    }

    pub fn sphere_pos(&self, sphere: Sphere) -> Point {
        match sphere {
            Sphere::Ball => self.ball_pos,
            Sphere::NearPaddle => self.near_paddle_pos,
            Sphere::FarPaddle => self.far_paddle_pos,
        }
    }

    pub fn sphere_radius(&self, sphere: Sphere) -> Real {
        SPHERES_RADII[sphere as usize]
    }

    pub fn plane_normal(&self, plane: Plane) -> Vector {
        PLANES_NORMALS[plane as usize]
    }

    pub fn plane_alignment_axis(&self, plane: Plane) -> Axis {
        PLANES_AXIS[plane as usize]
    }

    pub fn obstacle_color(&self, obstacle: Obstacle) -> Color {
        match obstacle {
            Obstacle::Plane(plane) => PLANES_COLORS[plane as usize],
            Obstacle::Sphere(sphere) => SPHERES_COLORS[sphere as usize],
        }
    }

    pub fn obstacle_reflectance(&self, obstacle: Obstacle) -> Real {
        match obstacle {
            Obstacle::Plane(plane) => PLANES_REFLECTANCE[plane as usize],
            Obstacle::Sphere(sphere) => SPHERES_REFLECTANCE[sphere as usize],
        }
    }

    pub fn move_sphere_to(&mut self, sphere: Sphere, position: Point) {
        let sphere_pos = match sphere {
            Sphere::Ball => &mut self.ball_pos,
            Sphere::NearPaddle => &mut self.near_paddle_pos,
            Sphere::FarPaddle => &mut self.far_paddle_pos,
        };
        *sphere_pos = position;
    }
}
