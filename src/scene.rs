use parry3d::math::{Isometry, Real, UnitVector, Vector};
use parry3d::query::Ray;
use parry3d::shape::{Ball, Cuboid};

use crate::color::Color;
use crate::material::{ColorDiffuse, Diffuse, Material, RayInteractionResult, SolidColor};

pub struct Body {
    shape: Box<dyn parry3d::shape::Shape>,
    translation: Isometry<Real>,
    material: Box<dyn Material>,
}

pub struct Scene {
    bodies: Vec<Body>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            bodies: vec![
                Body {
                    shape: Box::new(Ball::new(2f32)),
                    translation: Isometry::translation(0.0, 0.0, -15.0),
                    material: Box::new(ColorDiffuse::new(Color::new(0.05, 0.05, 0.5), 0.7)),
                },
                Body {
                    shape: Box::new(Ball::new(15f32)),
                    translation: Isometry::translation(0.0, -17.0, -15.0),
                    material: Box::new(ColorDiffuse::new(Color::new(0.0, 0.4, 0.0), 0.5)),
                },
                Body {
                    shape: Box::new(Ball::new(2f32)),
                    translation: Isometry::translation(5.0, -2.0, -12.0),
                    material: Box::new(SolidColor::new(Color::new(1.0, 0.0, 0.0))),
                },
                Body {
                    shape: Box::new(Cuboid::new(Vector::new(4.0, 0.2, 4.0))),
                    translation: Isometry::translation(5.0, 4.0, -13.0),
                    material: Box::new(Diffuse::new(0.7)),
                },
            ],
        }
    }

    pub fn ray_color(&self, ray: Ray, depth: u32) -> Color {
        if depth == 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let mut min_toi = std::f32::MAX;
        let mut closest_body = None;

        for body_idx in 0..self.bodies.len() {
            if let Some(intersection) = self.bodies[body_idx].shape.cast_ray_and_get_normal(
                &self.bodies[body_idx].translation,
                &ray,
                min_toi,
                true,
            ) {
                if intersection.toi > 0.001 {
                    min_toi = intersection.toi;
                    closest_body = Some((body_idx, intersection));
                }
            }
        }

        if let Some((body_idx, intersection)) = closest_body {
            match self.bodies[body_idx]
                .material
                .interact_with_ray(ray.point_at(intersection.toi), intersection.normal)
            {
                RayInteractionResult::Colored(color) => color,
                RayInteractionResult::Reflected(ray) => self.bodies[body_idx]
                    .material
                    .attenuate_reflected_color(self.ray_color(ray, depth - 1)),
            }
        } else {
            let unit = UnitVector::new_normalize(ray.dir);
            let t = 0.5f32 * (unit[1] + 1.0f32);
            let color = Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t;
            color
        }
    }
}
