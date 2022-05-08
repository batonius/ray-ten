use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::simd::{Mask as SimdMask, Simd, StdFloat};

pub mod camera;
pub mod render;
pub mod scene;

pub const LANES: usize = 8usize;

pub type Real = f32;
pub type Reals = Simd<Real, LANES>;
pub type Ints = Simd<u32, LANES>;
pub type Mask = SimdMask<i32, LANES>;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    x: Real,
    y: Real,
    z: Real,
}

#[derive(Clone, Debug)]
pub struct Points {
    xs: Reals,
    ys: Reals,
    zs: Reals,
}

impl Points {
    pub const fn new(xs: Reals, ys: Reals, zs: Reals) -> Self {
        Points { xs, ys, zs }
    }

    pub const fn splat(x: f32, y: f32, z: f32) -> Self {
        Points {
            xs: Reals::splat(x),
            ys: Reals::splat(y),
            zs: Reals::splat(z),
        }
    }

    pub fn sqrt(self) -> Self {
        Points {
            xs: self.xs.sqrt(),
            ys: self.ys.sqrt(),
            zs: self.zs.sqrt(),
        }
    }

    pub fn normalize(self) -> Self {
        let zeros = Reals::splat(0.0);
        let ones = Reals::splat(1.0);
        Points {
            xs: self.xs.clamp(zeros, ones),
            ys: self.ys.clamp(zeros, ones),
            zs: self.zs.clamp(zeros, ones),
        }
    }

    pub fn dot(&self, rhs: &Points) -> Reals {
        self.xs * rhs.xs + self.ys * rhs.ys + self.zs * rhs.zs
    }
}

impl Mul<&Reals> for &Points {
    type Output = Points;

    fn mul(self, rhs: &Reals) -> Self::Output {
        Points {
            xs: self.xs * rhs,
            ys: self.ys * rhs,
            zs: self.zs * rhs,
        }
    }
}

impl MulAssign<&Reals> for Points {
    fn mul_assign(&mut self, rhs: &Reals) {
        self.xs *= rhs;
        self.ys *= rhs;
        self.zs *= rhs;
    }
}

impl Mul<&Points> for &Points {
    type Output = Points;

    fn mul(self, rhs: &Points) -> Self::Output {
        Points {
            xs: self.xs * rhs.xs,
            ys: self.ys * rhs.ys,
            zs: self.zs * rhs.zs,
        }
    }
}

impl MulAssign<&Points> for Points {
    fn mul_assign(&mut self, rhs: &Points) {
        self.xs *= rhs.xs;
        self.ys *= rhs.ys;
        self.zs *= rhs.zs;
    }
}

impl Div<&Reals> for &Points {
    type Output = Points;

    fn div(self, rhs: &Reals) -> Self::Output {
        Points {
            xs: self.xs / rhs,
            ys: self.ys / rhs,
            zs: self.zs / rhs,
        }
    }
}

impl DivAssign<&Reals> for Points {
    fn div_assign(&mut self, rhs: &Reals) {
        self.xs /= rhs;
        self.ys /= rhs;
        self.zs /= rhs;
    }
}

impl Mul<Real> for &Points {
    type Output = Points;

    fn mul(self, rhs: Real) -> Self::Output {
        Points {
            xs: self.xs * Reals::splat(rhs),
            ys: self.ys * Reals::splat(rhs),
            zs: self.zs * Reals::splat(rhs),
        }
    }
}

impl MulAssign<Real> for Points {
    fn mul_assign(&mut self, rhs: Real) {
        self.xs *= Reals::splat(rhs);
        self.ys *= Reals::splat(rhs);
        self.zs *= Reals::splat(rhs);
    }
}

impl Add<&Points> for &Points {
    type Output = Points;
    fn add(self, rhs: &Points) -> Self::Output {
        Points {
            xs: self.xs + rhs.xs,
            ys: self.ys + rhs.ys,
            zs: self.zs + rhs.zs,
        }
    }
}

impl AddAssign<&Points> for Points {
    fn add_assign(&mut self, rhs: &Points) {
        self.xs += rhs.xs;
        self.ys += rhs.ys;
        self.zs += rhs.zs;
    }
}

impl Sub<&Points> for &Points {
    type Output = Points;
    fn sub(self, rhs: &Points) -> Self::Output {
        Points {
            xs: self.xs - rhs.xs,
            ys: self.ys - rhs.ys,
            zs: self.zs - rhs.zs,
        }
    }
}

impl SubAssign<&Points> for Points {
    fn sub_assign(&mut self, rhs: &Points) {
        self.xs -= rhs.xs;
        self.ys -= rhs.ys;
        self.zs -= rhs.zs;
    }
}

#[derive(Clone, Debug)]
pub struct Rays {
    origins: Points,
    dirs: Points,
}

impl Rays {
    pub fn new(origins: Points, dirs: Points) -> Self {
        Self { origins, dirs }
    }
}
