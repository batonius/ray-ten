use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::simd::{Mask as SimdMask, Simd, SimdFloat, StdFloat};

pub const LANES: usize = 8usize;

pub type Real = f32;
pub type Integer = i32;
pub type Reals = Simd<Real, LANES>;
pub type Integers = Simd<Integer, LANES>;
pub type Mask = SimdMask<i32, LANES>;

pub const ZEROS: Reals = Reals::from_array([0.0; LANES]);
pub const ZERO_POINTS: Points = Points::splat(0.0, 0.0, 0.0);

pub const fn splat_reals(x: Real) -> Reals {
    Reals::from_array([x; LANES])
}

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
    pub xs: Reals,
    pub ys: Reals,
    pub zs: Reals,
}

pub type Colors = Points;
pub type Vectors = Points;

impl Points {
    pub const fn splat(x: f32, y: f32, z: f32) -> Self {
        Points {
            xs: splat_reals(x),
            ys: splat_reals(y),
            zs: splat_reals(z),
        }
    }

    pub fn from_single(point: Point) -> Self {
        Self::splat(point.x(), point.y(), point.z())
    }

    pub fn get_axis(&self, axis: Axis) -> &Reals {
        match axis {
            Axis::XS => &self.xs,
            Axis::YS => &self.ys,
            Axis::ZS => &self.zs,
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
            xs: self.xs.simd_clamp(zeros, ones),
            ys: self.ys.simd_clamp(zeros, ones),
            zs: self.zs.simd_clamp(zeros, ones),
        }
    }

    pub fn dot(&self, rhs: Points) -> Reals {
        self.xs * rhs.xs + self.ys * rhs.ys + self.zs * rhs.zs
    }

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
    pub origins: Points,
    pub dirs: Vectors,
}

impl Rays {
    pub fn new(origins: Points, dirs: Points) -> Self {
        Self { origins, dirs }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Point(Simd<Real, 4>);

pub type Color = Point;
pub type Vector = Point;

impl Point {
    pub const fn new(x: Real, y: Real, z: Real) -> Point {
        Point(Simd::<Real, 4>::from_array([x, y, z, 0.0]))
    }

    pub fn x(&self) -> Real {
        self.0[0]
    }

    pub fn y(&self) -> Real {
        self.0[1]
    }

    pub fn z(&self) -> Real {
        self.0[2]
    }

    pub fn get_axis(&self, axis: Axis) -> Real {
        match axis {
            Axis::XS => self.x(),
            Axis::YS => self.y(),
            Axis::ZS => self.z(),
        }
    }

    pub fn x_mut(&mut self) -> &mut Real {
        &mut self.0[0]
    }

    pub fn y_mut(&mut self) -> &mut Real {
        &mut self.0[1]
    }

    pub fn z_mut(&mut self) -> &mut Real {
        &mut self.0[2]
    }

    pub fn get_axis_mut(&mut self, axis: Axis) -> &mut Real {
        match axis {
            Axis::XS => self.x_mut(),
            Axis::YS => self.y_mut(),
            Axis::ZS => self.z_mut(),
        }
    }

    pub fn dot(&self, rhs: Point) -> Real {
        self.x() * rhs.x() + self.y() * rhs.y() + self.z() * rhs.z()
    }
}

impl Add<Point> for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Self::Output {
        Point(self.0 + rhs.0)
    }
}

impl Sub<Point> for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Self::Output {
        Point(self.0 - rhs.0)
    }
}

impl Mul<Real> for Point {
    type Output = Point;
    fn mul(self, rhs: Real) -> Self::Output {
        Point(self.0 * Simd::<Real, 4>::splat(rhs))
    }
}

impl Div<Real> for Point {
    type Output = Point;
    fn div(self, rhs: Real) -> Self::Output {
        Point(self.0 / Simd::<Real, 4>::splat(rhs))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Directions {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Directions {
    pub fn new(up: bool, down: bool, left: bool, right: bool) -> Self {
        Self {
            up,
            down,
            left,
            right,
        }
    }

    pub fn to_vector(self, speed: Real) -> Vector {
        Vector::new(
            speed * self.right as usize as Real + (-speed) * self.left as usize as Real,
            speed * self.up as usize as Real + (-speed) * self.down as usize as Real,
            0.0,
        )
    }
}
