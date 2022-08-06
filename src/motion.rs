use crate::math::{Axis, Directions, Point, Real, Vector};
use crate::scene::{Obstacle, Plane, Scene, Sphere};

#[derive(Clone, Copy, Debug)]
pub enum MotionResult {
    NoCollision,
    Colision(Obstacle),
}

const PADDLE_FRICTION: Real = 0.7;
const PADDLE_ACCELERATION: Real = 2.0;

#[derive(Clone, Copy, Debug)]
pub struct MotionTicker {
    ball_speed: Vector,
    near_paddle_speed: Vector,
    far_paddle_speed: Vector,
}

impl MotionTicker {
    pub fn new() -> Self {
        Self {
            ball_speed: Vector::new(1.5, 4.5, -4.0),
            near_paddle_speed: Vector::new(0.0, 0.0, 0.0),
            far_paddle_speed: Vector::new(0.0, 0.0, 0.0),
        }
    }

    pub fn tick(
        &mut self,
        scene: &mut Scene,
        elapsed: Real,
        near_paddle_directions: Directions,
        far_paddle_directions: Directions,
    ) -> MotionResult {
        let new_ball_pos = scene.sphere_pos(Sphere::Ball) + self.ball_speed * elapsed;
        scene.move_sphere_to(Sphere::Ball, new_ball_pos);
        Self::move_paddle(
            scene,
            elapsed,
            Sphere::FarPaddle,
            far_paddle_directions,
            &mut self.far_paddle_speed,
        );
        Self::move_paddle(
            scene,
            elapsed,
            Sphere::NearPaddle,
            near_paddle_directions,
            &mut self.near_paddle_speed,
        );

        for sphere in [Sphere::FarPaddle, Sphere::NearPaddle] {
            if let Some((new_pos, normal)) =
                Self::collide_sphere_with_sphere(scene, Sphere::Ball, sphere)
            {
                scene.move_sphere_to(Sphere::Ball, new_pos);
                self.ball_speed = Self::bounce(self.ball_speed, normal);
                return MotionResult::Colision(Obstacle::Sphere(sphere));
            }
        }

        for (axis, min_plane, max_plane) in [
            (Axis::XS, Plane::Left, Plane::Right),
            (Axis::YS, Plane::Bottom, Plane::Top),
            (Axis::ZS, Plane::Far, Plane::Near),
        ] {
            if let Some((new_pos, plane)) =
                Self::collide_sphere_with_planes(axis, scene, Sphere::Ball, min_plane, max_plane)
            {
                scene.move_sphere_to(Sphere::Ball, new_pos);
                self.ball_speed = Self::bounce(self.ball_speed, scene.plane_normal(plane));
                return MotionResult::Colision(Obstacle::Plane(plane));
            }
        }

        MotionResult::NoCollision
    }

    fn move_paddle(
        scene: &mut Scene,
        elapsed: Real,
        paddle: Sphere,
        directions: Directions,
        paddle_speed: &mut Vector,
    ) {
        *paddle_speed = *paddle_speed * PADDLE_FRICTION.powf(elapsed)
            + directions.to_vector(PADDLE_ACCELERATION) * elapsed;
        let mut new_pos = scene.sphere_pos(paddle) + *paddle_speed * elapsed;

        let left_limit = scene.plane_offset(Plane::Left) + 1.0;
        let right_limit = scene.plane_offset(Plane::Right) - 1.0;
        let bottom_limit = scene.plane_offset(Plane::Bottom) + 1.0;
        let top_limit = scene.plane_offset(Plane::Top) - 1.0;

        if new_pos.x() <= left_limit || new_pos.x() >= right_limit {
            *paddle_speed.x_mut() = -paddle_speed.x();
        }

        if new_pos.y() <= bottom_limit || new_pos.y() >= top_limit {
            *paddle_speed.y_mut() = -paddle_speed.y();
        }

        *new_pos.x_mut() = new_pos.x().clamp(left_limit, right_limit);
        *new_pos.y_mut() = new_pos.y().clamp(bottom_limit, top_limit);
        scene.move_sphere_to(paddle, new_pos);
    }

    fn collide_sphere_with_planes(
        axis: Axis,
        scene: &Scene,
        sphere: Sphere,
        min_plane: Plane,
        max_plane: Plane,
    ) -> Option<(Point, Plane)> {
        let mut sphere_pos = scene.sphere_pos(sphere);
        let min_plane_offset = scene.plane_offset(min_plane);
        let max_plane_offset = scene.plane_offset(max_plane);
        let radius = scene.sphere_radius(sphere);

        if (sphere_pos.get_axis(axis) - min_plane_offset) < radius {
            *sphere_pos.get_axis_mut(axis) = min_plane_offset + radius;
            return Some((sphere_pos, min_plane));
        }

        if (max_plane_offset - sphere_pos.get_axis(axis)) < radius {
            *sphere_pos.get_axis_mut(axis) = max_plane_offset - radius;
            return Some((sphere_pos, max_plane));
        }

        None
    }

    fn collide_sphere_with_sphere(
        scene: &Scene,
        sphere_a: Sphere,
        sphere_b: Sphere,
    ) -> Option<(Point, Vector)> {
        let sphere_a_pos = scene.sphere_pos(sphere_a);
        let sphere_b_pos = scene.sphere_pos(sphere_b);
        let sphere_a_radius = scene.sphere_radius(sphere_a);
        let sphere_b_radius = scene.sphere_radius(sphere_b);

        let diff = sphere_a_pos - sphere_b_pos;
        let distance = (diff.x() * diff.x() + diff.y() * diff.y() + diff.z() * diff.z()).sqrt();

        if distance < (sphere_a_radius + sphere_b_radius) {
            let normalized_diff = diff / distance;
            return Some((
                sphere_b_pos + normalized_diff * (sphere_a_radius + sphere_b_radius),
                normalized_diff,
            ));
        }

        None
    }

    fn bounce(speed: Vector, normal: Vector) -> Vector {
        speed - (normal * speed.dot(normal) * 2.0)
    }
}
