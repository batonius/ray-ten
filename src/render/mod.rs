use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::simd::{Mask as SimdMask, Simd, SimdFloat, StdFloat};

pub mod camera;
pub mod render;
pub mod scene;

pub const LANES: usize = 8usize;

pub type Real = f32;
pub type Integer = i32;
pub type Reals = Simd<Real, LANES>;
pub type Integers = Simd<Integer, LANES>;
pub type Mask = SimdMask<i32, LANES>;

#[inline(always)]
pub const fn splat_reals(x: Real) -> Reals {
    Reals::from_array([x; LANES])
}

#[inline(always)]
pub fn update_reals_if(values: &mut Reals, mask: Mask, update_with: Reals) {
    *values = mask.select(update_with, *values);
}

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    XS,
    YS,
    ZS,
}

#[derive(Copy, Clone, Debug)]
pub struct Points {
    xs: Reals,
    ys: Reals,
    zs: Reals,
}

impl Points {
    #[inline(always)]
    pub const fn splat(x: f32, y: f32, z: f32) -> Self {
        Points {
            xs: splat_reals(x),
            ys: splat_reals(y),
            zs: splat_reals(z),
        }
    }

    #[inline(always)]
    pub fn get_axis(&self, axis: Axis) -> &Reals {
        match axis {
            Axis::XS => &self.xs,
            Axis::YS => &self.ys,
            Axis::ZS => &self.zs,
        }
    }

    #[inline(always)]
    pub fn sqrt(self) -> Self {
        Points {
            xs: self.xs.sqrt(),
            ys: self.ys.sqrt(),
            zs: self.zs.sqrt(),
        }
    }

    #[inline(always)]
    pub fn normalize(self) -> Self {
        let zeros = Reals::splat(0.0);
        let ones = Reals::splat(1.0);
        Points {
            xs: self.xs.simd_clamp(zeros, ones),
            ys: self.ys.simd_clamp(zeros, ones),
            zs: self.zs.simd_clamp(zeros, ones),
        }
    }

    #[inline(always)]
    pub fn dot(&self, rhs: Points) -> Reals {
        self.xs * rhs.xs + self.ys * rhs.ys + self.zs * rhs.zs
    }

    #[inline(always)]
    pub fn update_if(&mut self, mask: Mask, update_with: Points) {
        self.xs = mask.select(update_with.xs, self.xs);
        self.ys = mask.select(update_with.ys, self.ys);
        self.zs = mask.select(update_with.zs, self.zs);
    }
}

impl Mul<Reals> for Points {
    type Output = Points;

    fn mul(self, rhs: Reals) -> Self::Output {
        Points {
            xs: self.xs * rhs,
            ys: self.ys * rhs,
            zs: self.zs * rhs,
        }
    }
}

impl MulAssign<Reals> for Points {
    fn mul_assign(&mut self, rhs: Reals) {
        self.xs *= rhs;
        self.ys *= rhs;
        self.zs *= rhs;
    }
}

impl Mul<Points> for Points {
    type Output = Points;

    fn mul(self, rhs: Points) -> Self::Output {
        Points {
            xs: self.xs * rhs.xs,
            ys: self.ys * rhs.ys,
            zs: self.zs * rhs.zs,
        }
    }
}

impl MulAssign<Points> for Points {
    fn mul_assign(&mut self, rhs: Points) {
        self.xs *= rhs.xs;
        self.ys *= rhs.ys;
        self.zs *= rhs.zs;
    }
}

impl Div<Reals> for Points {
    type Output = Points;

    fn div(self, rhs: Reals) -> Self::Output {
        Points {
            xs: self.xs / rhs,
            ys: self.ys / rhs,
            zs: self.zs / rhs,
        }
    }
}

impl DivAssign<Reals> for Points {
    fn div_assign(&mut self, rhs: Reals) {
        self.xs /= rhs;
        self.ys /= rhs;
        self.zs /= rhs;
    }
}

impl Mul<Real> for Points {
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

impl Add<Points> for Points {
    type Output = Points;
    fn add(self, rhs: Points) -> Self::Output {
        Points {
            xs: self.xs + rhs.xs,
            ys: self.ys + rhs.ys,
            zs: self.zs + rhs.zs,
        }
    }
}

impl AddAssign<Points> for Points {
    fn add_assign(&mut self, rhs: Points) {
        self.xs += rhs.xs;
        self.ys += rhs.ys;
        self.zs += rhs.zs;
    }
}

impl Sub<Points> for Points {
    type Output = Points;
    fn sub(self, rhs: Points) -> Self::Output {
        Points {
            xs: self.xs - rhs.xs,
            ys: self.ys - rhs.ys,
            zs: self.zs - rhs.zs,
        }
    }
}

impl SubAssign<Points> for Points {
    fn sub_assign(&mut self, rhs: Points) {
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
