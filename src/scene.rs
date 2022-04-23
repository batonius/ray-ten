use std::f32::MIN;

use parry3d::math::{Isometry, Point, Real, UnitVector, Vector};
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
enum Obstacle {
    Top,
    Bottom,
    Left,
    Right,
    Far,
    Near,
    Sphere,
}

const OBSTACLE_COUNT: usize = 7;
const EPSILON: f32 = 0.001;
const OBSTACLE_NORMALS: [Vector<Real>; OBSTACLE_COUNT] = [
    Vector::new(0.0, -1.0, 0.0),
    Vector::new(0.0, 1.0, 0.0),
    Vector::new(1.0, 0.0, 0.0),
    Vector::new(-1.0, 0.0, 0.0),
    Vector::new(0.0, 0.0, 1.0),
    Vector::new(0.0, 0.0, -1.0),
    Vector::new(0.0, 0.0, 0.0),
];

const OBSTACLE_COLORS: [Color; OBSTACLE_COUNT] = [
    Color::new(0.5, 0.5, 0.0),
    Color::new(0.0, 0.5, 0.5),
    Color::new(0.5, 0.0, 0.5),
    Color::new(0.5, 0.0, 0.0),
    Color::new(0.0, 0.0, 0.5),
    Color::new(0.0, 0.5, 0.0),
    Color::new(0.5, 0.5, 0.5),
];

const OBSTACLE_OFFSETS: [f32; OBSTACLE_COUNT] = [2.0, -2.0, -2.0, 2.0, -8.0, 0.0, 0.0];

const SPHERE_RADIUS: f32 = 0.5;

const MIN_TOI: f32 = 0.001;

pub struct FixedScene {
    sphere_pos: Point<Real>,
}

impl FixedScene {
    pub fn new() -> Self {
        FixedScene {
            sphere_pos: Point::new(-1.0, 0.7, -4.0),
        }
    }

    fn intersects_sphere(&self, ray: &Ray) -> Option<f32> {
        let dir_squared = Point::new(
            ray.dir[0] * ray.dir[0],
            ray.dir[1] * ray.dir[1],
            ray.dir[2] * ray.dir[2],
        );
        let delta = ray.origin - self.sphere_pos;
        let r_squared = SPHERE_RADIUS * SPHERE_RADIUS;
        let d = r_squared * (dir_squared[0] + dir_squared[1] + dir_squared[2])
            - (ray.dir[0] * delta[1] - ray.dir[1] * delta[0]).powi(2)
            - (ray.dir[0] * delta[2] - ray.dir[2] * delta[0]).powi(2)
            - (ray.dir[1] * delta[2] - ray.dir[2] * delta[1]).powi(2);
        if d < 0.0 {
            return None;
        }
        let t1: f32 = (-delta[0] * ray.dir[0] - delta[1] * ray.dir[1] - delta[2] * ray.dir[2]
            + d.sqrt())
            / (dir_squared[0] + dir_squared[1] + dir_squared[2]);
        let mut t2 = f32::MAX;
        if d > EPSILON {
            t2 =
                (-delta[0] * ray.dir[0] - delta[1] * ray.dir[1] - delta[2] * ray.dir[2] - d.sqrt())
                    / (dir_squared[0] + dir_squared[1] + dir_squared[2]);
        }
        Some(t1.min(t2))
    }

    fn sphere_normal(&self, point: &Point<Real>) -> Vector<Real> {
        let norm = Vector::new(
            point[0] - self.sphere_pos[0],
            point[1] - self.sphere_pos[1],
            point[2] - self.sphere_pos[2],
        );
        norm / norm.magnitude()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intersects_sphere() {
        let scene = FixedScene::new();
        assert!(scene
            .intersects_sphere(&Ray::new(
                Point::new(0.0, 0.0, 0.0),
                Vector::new(0.0, 0.0, -1.0)
            ))
            .is_some());
    }
}

impl Scene for FixedScene {
    fn ray_color(&self, mut ray: Ray, depth: u32) -> Color {
        let mut coef_color = Color::new(1.0, 1.0, 1.0);
        let mut offset_color = Color::new(0.0, 0.0, 0.0);
        let base_color = Color::new(1.0, 1.0, 1.0);

        for _ in 0..depth {
            let mut min_toi = std::f32::MAX;
            let mut closest_obstacle = None;

            match self.intersects_sphere(&ray) {
                Some(toi) if toi > MIN_TOI => {
                    min_toi = toi;
                    closest_obstacle = Some(Obstacle::Sphere);
                }
                _ => {
                    if ray.dir[0].abs() > EPSILON {
                        if ray.dir[0] > 0.0 {
                            let toi = (OBSTACLE_OFFSETS[Obstacle::Right as usize] - ray.origin[0])
                                / ray.dir[0];
                            if toi < min_toi {
                                min_toi = toi;
                                closest_obstacle = Some(Obstacle::Right);
                            }
                        } else {
                            let toi = (OBSTACLE_OFFSETS[Obstacle::Left as usize] - ray.origin[0])
                                / ray.dir[0];
                            if toi < min_toi {
                                min_toi = toi;
                                closest_obstacle = Some(Obstacle::Left);
                            }
                        }
                    }

                    if ray.dir[1].abs() > EPSILON {
                        if ray.dir[1] > 0.0 {
                            let toi = (OBSTACLE_OFFSETS[Obstacle::Top as usize] - ray.origin[1])
                                / ray.dir[1];
                            if toi < min_toi {
                                min_toi = toi;
                                closest_obstacle = Some(Obstacle::Top);
                            }
                        } else {
                            let toi = (OBSTACLE_OFFSETS[Obstacle::Bottom as usize] - ray.origin[1])
                                / ray.dir[1];
                            if toi < min_toi {
                                min_toi = toi;
                                closest_obstacle = Some(Obstacle::Bottom);
                            }
                        }
                    }

                    if ray.dir[2].abs() > EPSILON {
                        if ray.dir[2] > 0.0 {
                            let toi = (OBSTACLE_OFFSETS[Obstacle::Near as usize] - ray.origin[2])
                                / ray.dir[2];
                            if toi < min_toi {
                                closest_obstacle = Some(Obstacle::Near);
                            }
                        } else {
                            let toi = (OBSTACLE_OFFSETS[Obstacle::Far as usize] - ray.origin[2])
                                / ray.dir[2];
                            if toi < min_toi {
                                closest_obstacle = Some(Obstacle::Far);
                            }
                        }
                    }
                }
            }

            if let Some(obstacle) = closest_obstacle {
                let poi = ray.point_at(min_toi);
                let normal;
                if obstacle == Obstacle::Sphere {
                    normal = self.sphere_normal(&poi);
                } else {
                    normal = OBSTACLE_NORMALS[obstacle as usize];
                }
                let reflection_dir = ray.dir - 2.0 * ray.dir.dot(&normal) * normal;
                ray = Ray {
                    origin: poi,
                    dir: reflection_dir,
                };
                offset_color += coef_color * OBSTACLE_COLORS[obstacle as usize];
                coef_color *= Color::new(0.5, 0.5, 0.5);
            }
        }

        offset_color + coef_color * base_color
    }
}
