use parry3d::math::{Isometry, Real, UnitVector, Vector};
use parry3d::na::ComplexField;
use parry3d::query::Ray;
use parry3d::shape::{Ball, Cuboid};

use crate::color::Color;
use crate::material::{ColorDiffuse, Material, Metal, RayInteractionResult};

pub struct Body {
    shape: Box<dyn parry3d::shape::Shape>,
    translation: Isometry<Real>,
    material: Box<dyn Material>,
}

pub trait Scene {
    fn ray_color(&self, ray: Ray, depth: u32) -> Color;
}

pub struct DynamicScene {
    bodies: Vec<Body>,
}

impl DynamicScene {
    pub fn new() -> Self {
        DynamicScene {
            bodies: vec![
                Body {
                    shape: Box::new(Ball::new(2f32)),
                    translation: Isometry::translation(0.0, 0.0, -15.0),
                    material: Box::new(ColorDiffuse::new(Color::new(0.05, 0.05, 0.5), 0.7)),
                },
                Body {
                    shape: Box::new(Ball::new(15f32)),
                    translation: Isometry::translation(0.0, -17.0, -15.0),
                    // material: Box::new(ColorDiffuse::new(Color::new(0.0, 0.4, 0.0), 0.5)),
                    material: Box::new(Metal::new(Color::new(0.5, 0.8, 0.5), 0.5)),
                },
                Body {
                    shape: Box::new(Ball::new(2f32)),
                    translation: Isometry::translation(5.0, 0.0, -12.0),
                    // material: Box::new(SolidColor::new(Color::new(1.0, 0.0, 0.0))),
                    material: Box::new(Metal::new(Color::new(0.9, 0.3, 0.3), 0.001)),
                },
                Body {
                    shape: Box::new(Cuboid::new(Vector::new(8.0, 0.1, 4.0))),
                    translation: Isometry::translation(2.0, 3.0, -18.0)
                        * Isometry::rotation(Vector::new(-1.0, 0.0, 0.0)),
                    material: Box::new(Metal::new(Color::new(0.5, 0.5, 0.7), 0.0)),
                    // material: Box::new(Diffuse::new(0.7)),
                },
            ],
        }
    }

    #[allow(dead_code)]
    pub fn ray_color_iter(&self, mut ray: Ray, depth: u32) -> Color {
        let mut coef_color = Color::new(1.0, 1.0, 1.0);
        let mut offset_color = Color::new(0.0, 0.0, 0.0);
        let mut base_color = Color::new(0.0, 0.0, 0.0);

        for _ in 0..depth {
            if let Some((body_idx, intersection)) = self.find_closest_body(ray) {
                match self.bodies[body_idx].material.interact_with_ray(
                    &ray,
                    ray.point_at(intersection.toi),
                    intersection.normal,
                ) {
                    RayInteractionResult::Colored(color) => {
                        base_color = color;
                        break;
                    }
                    RayInteractionResult::Reflected {
                        ray: reflected_ray,
                        coef,
                        offset,
                    } => {
                        ray = reflected_ray;
                        offset_color += coef_color * offset;
                        coef_color *= coef;
                    }
                }
            } else {
                base_color = self.ambient_color(ray);
                break;
            }
        }

        offset_color + coef_color * base_color
    }

    #[allow(dead_code)]
    pub fn ray_color(&self, ray: Ray, depth: u32) -> Color {
        if depth == 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        if let Some((body_idx, intersection)) = self.find_closest_body(ray) {
            match self.bodies[body_idx].material.interact_with_ray(
                &ray,
                ray.point_at(intersection.toi),
                intersection.normal,
            ) {
                RayInteractionResult::Colored(color) => color,
                RayInteractionResult::Reflected { ray, coef, offset } => {
                    offset + coef * self.ray_color(ray, depth - 1)
                }
            }
        } else {
            self.ambient_color(ray)
        }
    }

    fn find_closest_body(&self, ray: Ray) -> Option<(usize, parry3d::query::RayIntersection)> {
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
        closest_body
    }

    fn ambient_color(&self, ray: Ray) -> Color {
        let unit = UnitVector::new_normalize(ray.dir);
        let t = 0.5f32 * (unit[1] + 1.0f32);
        let color = Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t;
        color
    }
}

impl Scene for DynamicScene {
    fn ray_color(&self, ray: Ray, depth: u32) -> Color {
        self.ray_color_iter(ray, depth)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Plane {
    Top,
    Bottom,
    Left,
    Right,
    Far,
    Near,
}

const PLANES_COUNT: usize = 6;
const EPSILON: f32 = 0.001;
const PLANES_NORMALS: [Vector<Real>; PLANES_COUNT] = [
    Vector::new(0.0, -1.0, 0.0),
    Vector::new(0.0, 1.0, 0.0),
    Vector::new(1.0, 0.0, 0.0),
    Vector::new(-1.0, 0.0, 0.0),
    Vector::new(0.0, 0.0, 1.0),
    Vector::new(0.0, 0.0, -1.0),
];

const PLANES_COLORS: [Color; PLANES_COUNT] = [
    Color::new(0.5, 0.5, 0.0),
    Color::new(0.0, 0.5, 0.5),
    Color::new(0.5, 0.0, 0.5),
    Color::new(0.5, 0.0, 0.0),
    Color::new(0.0, 0.0, 0.5),
    Color::new(0.0, 0.5, 0.0),
];

const PLANES_OFFSETS: [f32; PLANES_COUNT] = [2.0, -2.0, -2.0, 2.0, -8.0, 0.0];

pub struct FixedScene {}

impl FixedScene {
    pub fn new() -> Self {
        FixedScene {}
    }
}

impl Scene for FixedScene {
    fn ray_color(&self, mut ray: Ray, depth: u32) -> Color {
        let mut coef_color = Color::new(1.0, 1.0, 1.0);
        let mut offset_color = Color::new(0.0, 0.0, 0.0);
        let mut base_color = Color::new(1.0, 1.0, 1.0);

        for _ in 0..depth {
            let mut min_toi = std::f32::MAX;
            let mut closest_plane = None;

            if ray.dir[0].abs() > EPSILON {
                if ray.dir[0] > 0.0 {
                    let toi = (PLANES_OFFSETS[Plane::Right as usize] - ray.origin[0]) / ray.dir[0];
                    if toi < min_toi {
                        min_toi = toi;
                        closest_plane = Some(Plane::Right);
                    }
                } else {
                    let toi = (PLANES_OFFSETS[Plane::Left as usize] - ray.origin[0]) / ray.dir[0];
                    if toi < min_toi {
                        min_toi = toi;
                        closest_plane = Some(Plane::Left);
                    }
                }
            }

            if ray.dir[1].abs() > EPSILON {
                if ray.dir[1] > 0.0 {
                    let toi = (PLANES_OFFSETS[Plane::Top as usize] - ray.origin[1]) / ray.dir[1];
                    if toi < min_toi {
                        min_toi = toi;
                        closest_plane = Some(Plane::Top);
                    }
                } else {
                    let toi = (PLANES_OFFSETS[Plane::Bottom as usize] - ray.origin[1]) / ray.dir[1];
                    if toi < min_toi {
                        min_toi = toi;
                        closest_plane = Some(Plane::Bottom);
                    }
                }
            }

            if ray.dir[2].abs() > EPSILON {
                if ray.dir[2] > 0.0 {
                    let toi = (PLANES_OFFSETS[Plane::Near as usize] - ray.origin[2]) / ray.dir[2];
                    if toi < min_toi {
                        closest_plane = Some(Plane::Near);
                    }
                } else {
                    let toi = (PLANES_OFFSETS[Plane::Far as usize] - ray.origin[2]) / ray.dir[2];
                    if toi < min_toi {
                        closest_plane = Some(Plane::Far);
                    }
                }
            }

            if let Some(plane) = closest_plane {
                let poi = ray.point_at(min_toi);
                if plane != Plane::Far {
                    let normal = PLANES_NORMALS[plane as usize];
                    let reflection_dir = ray.dir - 2.0 * ray.dir.dot(&normal) * normal;
                    ray = Ray {
                        origin: poi,
                        dir: reflection_dir,
                    };
                    offset_color += coef_color * PLANES_COLORS[plane as usize];
                    coef_color *= Color::new(0.2, 0.2, 0.2);
                } else {
                    if (poi[0] - poi[1]).abs() < 0.2 {
                        base_color = Color::new(0.0, 0.0, 0.0);
                    } else {
                        base_color = PLANES_COLORS[plane as usize];
                    }
                }
            }
        }

        offset_color + coef_color * base_color
    }
}
