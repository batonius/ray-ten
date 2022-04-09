use anyhow::Result;
use image::Rgb;
use num::clamp;
use parry3d::math::{Isometry, Real, UnitVector, Vector};
use parry3d::query::Ray;
use parry3d::shape::{Ball, Cuboid};
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::thread_rng;

use crate::camera::Camera;
use crate::material::{Diffuse, Material, RayInteractionResult, SolidColor};
use crate::{Buffer, Color};

pub struct Body {
    shape: Box<dyn parry3d::shape::Shape>,
    translation: Isometry<Real>,
    material: Box<dyn Material>,
}

pub struct Scene {
    camera: Camera,
    bodies: Vec<Body>,
}

impl Scene {
    pub fn new(aspect_ratio: f32) -> Self {
        Scene {
            camera: Camera::new(aspect_ratio),
            bodies: vec![
                Body {
                    shape: Box::new(Ball::new(2f32)),
                    translation: Isometry::translation(0.0, 0.0, -20.0),
                    material: Box::new(SolidColor::new(Rgb([10, 10, 128u8]))),
                },
                Body {
                    shape: Box::new(Ball::new(15f32)),
                    translation: Isometry::translation(0.0, -17.0, -20.0),
                    material: Box::new(Diffuse::new(0.8)),
                },
                Body {
                    shape: Box::new(Ball::new(2f32)),
                    translation: Isometry::translation(5.0, -2.0, -17.0),
                    material: Box::new(SolidColor::new(Rgb([255u8, 0, 0]))),
                },
                Body {
                    shape: Box::new(Cuboid::new(Vector::new(4.0, 0.2, 4.0))),
                    translation: Isometry::translation(5.0, 4.0, -18.0),
                    material: Box::new(Diffuse::new(0.4)),
                },
            ],
        }
    }

    pub fn render(&self, buffer: &mut Buffer, samples_per_pixel: u32) -> Result<()> {
        let (width, height) = buffer.dimensions();
        let mut rng = thread_rng();
        let unit_distr = Uniform::new(0.0f32, 1.0f32);

        for x in 0..width {
            for y in 0..height {
                let mut r = 0.0f32;
                let mut g = 0.0f32;
                let mut b = 0.0f32;

                for _ in 0..samples_per_pixel {
                    let ray = self.camera.pixel_ray(
                        (width, height),
                        (x, y),
                        (unit_distr.sample(&mut rng), unit_distr.sample(&mut rng)),
                    );
                    let pixel = self.ray_color(ray, 50);
                    r += pixel[0] as f32 / 255.0f32;
                    g += pixel[1] as f32 / 255.0f32;
                    b += pixel[2] as f32 / 255.0f32;
                }
                let pixel = Rgb([
                    clamp((r / samples_per_pixel as f32).sqrt() * 255.0, 0.0, 255.0) as u8,
                    clamp((g / samples_per_pixel as f32).sqrt() * 255.0, 0.0, 255.0) as u8,
                    clamp((b / samples_per_pixel as f32).sqrt() * 255.0, 0.0, 255.0) as u8,
                ]);
                buffer.put_pixel(x, y, pixel);
            }
        }
        Ok(())
    }

    fn ray_color(&self, ray: Ray, depth: u32) -> Color {
        if depth == 0 {
            return Rgb([0u8, 0, 0]);
        }

        let mut min_toi = std::f32::MAX;
        let mut closest_body = None;

        for body_idx in 0..self.bodies.len() {
            if let Some(intersection) = self.bodies[body_idx].shape.cast_ray_and_get_normal(
                &self.bodies[body_idx].translation,
                &ray,
                f32::MAX,
                false,
            ) {
                if intersection.toi > 0.001 && min_toi > intersection.toi {
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
            // Rgb([255u8, 255u8, 255u8])
            let unit = UnitVector::new_normalize(ray.dir);
            let t = 0.5f32 * (unit[1] + 1.0f32);
            let color = (1.0 - t) * Vector::new(1.0, 1.0, 1.0) + Vector::new(0.5, 0.7, 1.0) * t;
            Rgb([
                clamp(color[0] * 255.0, 0.0, 255.0) as u8,
                clamp(color[1] * 255.0, 0.0, 255.0) as u8,
                clamp(color[1] * 255.0, 0.0, 255.0) as u8,
            ])
        }
    }
}
