use crate::color::Color;
use parry3d::math::{Point, Real, Vector};
use parry3d::query::Ray;
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::thread_rng;
pub enum RayInteractionResult {
    Colored(Color),
    Reflected {
        ray: Ray,
        coef: Color,
        offset: Color,
    },
}

pub trait Material {
    fn interact_with_ray(
        &self,
        ray: &Ray,
        poi: Point<Real>,
        normal: Vector<Real>,
    ) -> RayInteractionResult;
}

pub struct SolidColor {
    color: Color,
}

impl SolidColor {
    pub fn new(color: Color) -> Self {
        SolidColor { color }
    }
}

impl Material for SolidColor {
    fn interact_with_ray(
        &self,
        _ray: &Ray,
        _poi: Point<Real>,
        _normal: Vector<Real>,
    ) -> RayInteractionResult {
        RayInteractionResult::Colored(self.color)
    }
}

pub struct Diffuse {
    attenuation: f32,
}

impl Diffuse {
    pub fn new(attenuation: f32) -> Self {
        Diffuse { attenuation }
    }
}

fn random_unit_vect() -> Vector<Real> {
    let mut rng = thread_rng();
    let tau_distr = Uniform::new(0.0f32, std::f32::consts::TAU);
    let alpha = tau_distr.sample(&mut rng);
    let beta = tau_distr.sample(&mut rng);
    Vector::new(
        alpha.cos() * beta.cos(),
        beta.sin(),
        alpha.sin() * beta.cos(),
    )
}

impl Material for Diffuse {
    fn interact_with_ray(
        &self,
        _ray: &Ray,
        poi: Point<Real>,
        normal: Vector<Real>,
    ) -> RayInteractionResult {
        RayInteractionResult::Reflected {
            ray: Ray {
                origin: poi,
                dir: normal + random_unit_vect(),
            },
            coef: Color::new(self.attenuation, self.attenuation, self.attenuation),
            offset: Color::new(0.0, 0.0, 0.0),
        }
    }
}

pub struct ColorDiffuse {
    color: Color,
    coef: f32,
}

impl ColorDiffuse {
    pub fn new(color: Color, coef: f32) -> Self {
        assert!(coef <= 1.0f32);
        ColorDiffuse { color, coef }
    }
}

impl Material for ColorDiffuse {
    fn interact_with_ray(
        &self,
        _ray: &Ray,
        poi: Point<Real>,
        normal: Vector<Real>,
    ) -> RayInteractionResult {
        RayInteractionResult::Reflected {
            ray: Ray {
                origin: poi,
                dir: normal + random_unit_vect(),
            },
            coef: Color::new(1.0 - self.coef, 1.0 - self.coef, 1.0 - self.coef),
            offset: self.color * self.coef,
        }
    }
}

pub struct Metal {
    attenuate: Color,
    fuzzyness: f32,
}

impl Metal {
    pub fn new(attenuate: Color, fuzzyness: f32) -> Self {
        assert!(fuzzyness >= 0.0 && fuzzyness <= 1.0);
        Self {
            attenuate,
            fuzzyness,
        }
    }
}

impl Material for Metal {
    fn interact_with_ray(
        &self,
        ray: &Ray,
        poi: Point<Real>,
        mut normal: Vector<Real>,
    ) -> RayInteractionResult {
        normal = normal / normal.norm();
        let reflection_dir =
            (ray.dir - 2.0 * ray.dir.dot(&normal) * normal) + random_unit_vect() * self.fuzzyness;
        RayInteractionResult::Reflected {
            ray: Ray {
                origin: poi,
                dir: reflection_dir,
            },
            coef: self.attenuate,
            offset: Color::new(0.0, 0.0, 0.0),
        }
    }
}
