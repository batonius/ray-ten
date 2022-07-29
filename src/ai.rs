use crate::math::Real;
use crate::motion::PaddleControls;
use crate::scene::{Scene, Sphere};

const EPSILON: Real = 0.1;

pub fn control_far_paddle(scene: &Scene) -> PaddleControls {
    PaddleControls::new(
        (scene.sphere_pos(Sphere::Ball).y() - scene.sphere_pos(Sphere::FarPaddle).y()) > EPSILON,
        (scene.sphere_pos(Sphere::FarPaddle).y() - scene.sphere_pos(Sphere::Ball).y()) > EPSILON,
        (scene.sphere_pos(Sphere::FarPaddle).x() - scene.sphere_pos(Sphere::Ball).x()) > EPSILON,
        (scene.sphere_pos(Sphere::Ball).x() - scene.sphere_pos(Sphere::FarPaddle).x()) > EPSILON,
    )
}
