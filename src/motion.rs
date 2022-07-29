use crate::math::{Axis, Real, Vector};
use crate::scene::{Obstacle, Plane, Scene, Sphere};

#[derive(Clone, Copy, Debug)]
pub struct PaddleControls {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl PaddleControls {
    pub fn still() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
    pub fn new(up: bool, down: bool, left: bool, right: bool) -> Self {
        Self {
            up,
            down,
            left,
            right,
        }
    }

    fn to_vector(self, speed: Real) -> Vector {
        Vector::new(
            speed * self.right as usize as Real + (-speed) * self.left as usize as Real,
            speed * self.up as usize as Real + (-speed) * self.down as usize as Real,
            0.0,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MotionResult {
    NoCollision,
    Colision(Obstacle),
}

#[derive(Clone, Copy, Debug)]
pub struct MotionTicker {
    ball_speed: Vector,
}

impl MotionTicker {
    pub fn new() -> Self {
        Self {
            ball_speed: Vector::new(1.5, 1.5, -1.5),
        }
    }

    pub fn tick(
        &mut self,
        scene: &mut Scene,
        elapsed: Real,
        near_paddle_controls: PaddleControls,
        far_paddle_controls: PaddleControls,
    ) -> MotionResult {
        let new_ball_pos = scene.sphere_pos(Sphere::Ball) + self.ball_speed * elapsed;
        scene.move_sphere_to(Sphere::Ball, new_ball_pos);
        self.move_paddle(scene, Sphere::FarPaddle, far_paddle_controls);
        self.move_paddle(scene, Sphere::NearPaddle, near_paddle_controls);

        for (axis, min_plane, max_plane) in [
            (Axis::XS, Plane::Left, Plane::Right),
            (Axis::YS, Plane::Bottom, Plane::Top),
            (Axis::ZS, Plane::Far, Plane::Near),
        ] {
            if let Some(plane) =
                self.collide_sphere_with_plane(axis, scene, Sphere::Ball, min_plane, max_plane)
            {
                self.ball_speed = Self::bounce(self.ball_speed, scene.plane_normal(plane));
                return MotionResult::Colision(Obstacle::Plane(plane));
            }
        }

        MotionResult::NoCollision
    }

    fn move_paddle(&self, scene: &mut Scene, paddle: Sphere, controls: PaddleControls) {
        let mut new_pos = scene.sphere_pos(paddle) + controls.to_vector(0.1);
        *new_pos.x_mut() = new_pos.x().clamp(
            scene.plane_offset(Plane::Left) + 0.1,
            scene.plane_offset(Plane::Right) - 0.1,
        );
        *new_pos.y_mut() = new_pos.y().clamp(
            scene.plane_offset(Plane::Bottom) + 0.1,
            scene.plane_offset(Plane::Top) - 0.1,
        );
        scene.move_sphere_to(paddle, new_pos);
    }

    fn collide_sphere_with_plane(
        &mut self,
        axis: Axis,
        scene: &mut Scene,
        sphere: Sphere,
        min_plane: Plane,
        max_plane: Plane,
    ) -> Option<Plane> {
        let mut sphere_pos = scene.sphere_pos(sphere);
        let min_plane_offset = scene.plane_offset(min_plane);
        let max_plane_offset = scene.plane_offset(max_plane);
        let radius = scene.sphere_radius(sphere);

        if (sphere_pos.get_axis(axis) - min_plane_offset) < radius {
            *sphere_pos.get_axis_mut(axis) = min_plane_offset + radius;
            scene.move_sphere_to(sphere, sphere_pos);
            return Some(min_plane);
        }

        if (max_plane_offset - sphere_pos.get_axis(axis)) < radius {
            *sphere_pos.get_axis_mut(axis) = max_plane_offset - radius;
            scene.move_sphere_to(sphere, sphere_pos);
            return Some(max_plane);
        }

        None
    }

    fn bounce(speed: Vector, normal: Vector) -> Vector {
        speed - (normal * speed.dot(normal) * 2.0)
    }
}
