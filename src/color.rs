use num::clamp;
use std::ops::{Add, AddAssign, Mul, MulAssign};

#[derive(Clone, Copy)]
pub struct Color([f32; 3]);

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Color([r, g, b])
    }

    pub fn normalize(&self) -> Self {
        Color([
            clamp(self.0[0], 0.0, 1.0),
            clamp(self.0[1], 0.0, 1.0),
            clamp(self.0[2], 0.0, 1.0),
        ])
    }

    pub fn into_rgb(self) -> [u8; 3] {
        [
            clamp(self.0[0] * 255.0, 0.0, 255.0) as u8,
            clamp(self.0[1] * 255.0, 0.0, 255.0) as u8,
            clamp(self.0[2] * 255.0, 0.0, 255.0) as u8,
        ]
    }

    #[allow(dead_code)]
    pub fn r(&self) -> f32 {
        self.0[0]
    }

    #[allow(dead_code)]
    pub fn g(&self) -> f32 {
        self.0[0]
    }

    #[allow(dead_code)]
    pub fn b(&self) -> f32 {
        self.0[0]
    }

    #[allow(dead_code)]
    pub fn blend(&self, rhs: Color) -> Self {
        ((*self + rhs) * 0.5).normalize()
    }

    #[allow(dead_code)]
    pub fn blend_with_coef(&self, rhs: Color, coef: f32) -> Self {
        assert!(coef <= 1.0);
        (*self * coef + (rhs * (1.0 - coef))).normalize()
    }

    pub fn sqrt(&self) -> Self {
        Color([self.0[0].sqrt(), self.0[1].sqrt(), self.0[2].sqrt()])
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Color([rhs * self.0[0], rhs * self.0[1], rhs * self.0[2]])
    }
}

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        self.0[0] *= rhs;
        self.0[1] *= rhs;
        self.0[2] *= rhs;
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color([
            rhs.0[0] * self.0[0],
            rhs.0[1] * self.0[1],
            rhs.0[2] * self.0[2],
        ])
    }
}

impl MulAssign<Color> for Color {
    fn mul_assign(&mut self, rhs: Color) {
        self.0[0] *= rhs.0[0];
        self.0[1] *= rhs.0[1];
        self.0[2] *= rhs.0[2];
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Self) -> Self::Output {
        Color([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.0[0] += rhs.0[0];
        self.0[1] += rhs.0[1];
        self.0[2] += rhs.0[2];
    }
}
