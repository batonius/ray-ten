use crate::color::Color;
use parry3d::math::{Point, Real, Vector};
use parry3d::query::Ray;
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::thread_rng;
pub enum RayInteractionResult {
    Colored(Color),
    Reflected(Ray),
}

pub trait Material {
    fn interact_with_ray(&self, poi: Point<Real>, normal: Vector<Real>) -> RayInteractionResult;
    fn attenuate_reflected_color(&self, color: Color) -> Color {
        color
    }
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
    fn interact_with_ray(&self, _poi: Point<Real>, _normal: Vector<Real>) -> RayInteractionResult {
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
    fn interact_with_ray(&self, poi: Point<Real>, normal: Vector<Real>) -> RayInteractionResult {
        RayInteractionResult::Reflected(Ray {
            origin: poi,
            dir: normal + random_unit_vect(),
        })
    }

    fn attenuate_reflected_color(&self, color: Color) -> Color {
        color * self.attenuation
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
    fn interact_with_ray(&self, poi: Point<Real>, normal: Vector<Real>) -> RayInteractionResult {
        RayInteractionResult::Reflected(Ray {
            origin: poi,
            dir: normal + random_unit_vect(),
        })
    }

    fn attenuate_reflected_color(&self, color: Color) -> Color {
        color.blend_with_coef(self.color, self.coef)
    }
}
