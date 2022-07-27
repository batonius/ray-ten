use crate::render::{splat_reals, Points, Rays, Reals};
use std::simd::{SimdFloat, SimdPartialOrd, StdFloat};

use super::{update_reals_if, Axis};

pub trait Scene {
    fn rays_colors(&self, rays: Rays, depth: u32) -> Points;
}

pub struct FixedScene {
    sphere_pos: Points,
}

impl FixedScene {
    pub fn new() -> Self {
        FixedScene {
            sphere_pos: Points::splat(-2.0, -1.0, -6.0),
        }
    }

    pub fn move_sphere(&mut self, delta_x: f32, delta_y: f32, delta_z: f32) {
        self.sphere_pos.xs += Reals::splat(delta_x);
        self.sphere_pos.ys += Reals::splat(delta_y);
        self.sphere_pos.zs += Reals::splat(delta_z);
    }
}

struct RaysProjections {
    rays: Rays,
    min_toi: Reals,
    obstacle_reflectances: Reals,
    obstacle_colors: Points,
    obstacle_normals: Points,
    offset_colors: Points,
    coef_colors: Points,
}

impl RaysProjections {
    fn new(rays: Rays) -> RaysProjections {
        RaysProjections {
            rays,
            min_toi: Reals::splat(std::f32::MAX),
            obstacle_reflectances: Reals::splat(std::f32::MAX),
            obstacle_colors: Points::splat(0.0, 0.0, 0.0),
            obstacle_normals: Points::splat(0.0, 0.0, 0.0),
            offset_colors: Points::splat(0.0, 0.0, 0.0),
            coef_colors: Points::splat(1.0, 1.0, 1.0),
        }
    }

    #[inline(always)]
    fn with_axis_aligned_plane(
        &mut self,
        axis: Axis,
        offset_within_axis: Reals,
        normal: Points,
        color: Points,
        reflectance: Reals,
    ) {
        let toi =
            (offset_within_axis - self.rays.origins.get_axis(axis)) / self.rays.dirs.get_axis(axis);
        let mask = toi.simd_gt(Reals::splat(MIN_TOI)) & toi.simd_lt(self.min_toi);
        update_reals_if(&mut self.min_toi, mask, toi);

        self.obstacle_colors.update_if(mask, color);
        self.obstacle_normals.update_if(mask, normal);

        update_reals_if(&mut self.obstacle_reflectances, mask, reflectance);
    }

    #[inline(always)]
    fn with_sphere(
        &mut self,
        sphere_pos: Points,
        sphere_radius: f32,
        color: Points,
        reflectance: Reals,
    ) {
        let deltas = self.rays.origins - sphere_pos;

        let dirs_squared = self.rays.dirs * self.rays.dirs;
        let dirs_squared_sum = dirs_squared.xs + dirs_squared.ys + dirs_squared.zs;
        let r_squared = Reals::splat(sphere_radius * sphere_radius);
        let mut d = r_squared * dirs_squared_sum;
        let a = self.rays.dirs.xs * deltas.ys - self.rays.dirs.ys * deltas.xs;
        d -= a * a;
        if !d.simd_ge(Reals::splat(0.0)).any() {
            return;
        }
        let b = self.rays.dirs.xs * deltas.zs - self.rays.dirs.zs * deltas.xs;
        d -= b * b;
        if !d.simd_ge(Reals::splat(0.0)).any() {
            return;
        }
        let c = self.rays.dirs.ys * deltas.zs - self.rays.dirs.zs * deltas.ys;
        d -= c * c;

        let mask = d.simd_ge(Reals::splat(0.0));
        if !mask.any() {
            return;
        }

        d = d.simd_max(Reals::splat(0.0));
        let tts = Reals::splat(0.0)
            - deltas.xs * self.rays.dirs.xs
            - deltas.ys * self.rays.dirs.ys
            - deltas.zs * self.rays.dirs.zs;
        let mut t1s = (tts + d.sqrt()) / dirs_squared_sum;
        let mut t2s = (tts - d.sqrt()) / dirs_squared_sum;
        t1s = t1s.simd_max(Reals::splat(0.0));
        t2s = t2s.simd_max(Reals::splat(0.0));
        let toi = t1s.simd_min(t2s);
        let mask = mask & toi.simd_gt(Reals::splat(MIN_TOI)) & toi.simd_lt(self.min_toi);

        update_reals_if(&mut self.min_toi, mask, toi);

        self.obstacle_colors.update_if(mask, color);

        let pois = self.rays.origins + self.rays.dirs * self.min_toi;
        let mut normals = pois - sphere_pos;
        let magnitudes =
            (normals.xs * normals.xs + normals.ys * normals.ys + normals.zs * normals.zs).sqrt();
        normals /= magnitudes;

        self.obstacle_normals.update_if(mask, normals);
        update_reals_if(&mut self.obstacle_reflectances, mask, reflectance);
    }

    #[inline(always)]
    fn reflect(&mut self) {
        let pois = self.rays.origins + self.rays.dirs * self.min_toi;
        let reflection_dirs = self.rays.dirs
            - (self.obstacle_normals
                * self.rays.dirs.dot(self.obstacle_normals)
                * Reals::splat(2.0));

        self.rays = Rays::new(pois, reflection_dirs);

        self.offset_colors += self.coef_colors * self.obstacle_colors;
        self.coef_colors *= self.obstacle_reflectances;
        self.min_toi = Reals::splat(std::f32::MAX);
    }

    #[inline(always)]
    fn finish(mut self, base_colors: Points) -> Points {
        self.coef_colors *= base_colors;
        self.offset_colors += self.coef_colors;
        self.offset_colors
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
    None,
}

const OBSTACLE_COUNT: usize = Obstacle::None as usize;
const OBSTACLE_NORMALS: [Points; OBSTACLE_COUNT] = [
    Points::splat(0.0, -1.0, 0.0),
    Points::splat(0.0, 1.0, 0.0),
    Points::splat(1.0, 0.0, 0.0),
    Points::splat(-1.0, 0.0, 0.0),
    Points::splat(0.0, 0.0, 1.0),
    Points::splat(0.0, 0.0, -1.0),
    Points::splat(0.0, 0.0, 0.0),
];

const OBSTACLE_COLORS: [Points; OBSTACLE_COUNT] = [
    Points::splat(0.9, 0.9, 0.0),
    Points::splat(0.0, 0.9, 0.9),
    Points::splat(0.9, 0.0, 0.9),
    Points::splat(0.9, 0.0, 0.0),
    Points::splat(0.0, 0.0, 0.9),
    Points::splat(0.0, 0.9, 0.0),
    Points::splat(0.0, 0.0, 0.0),
];

const OBSTACLE_REFLECTANCES: [Reals; OBSTACLE_COUNT] = [
    splat_reals(0.1),
    splat_reals(0.1),
    splat_reals(0.1),
    splat_reals(0.1),
    splat_reals(0.1),
    splat_reals(0.1),
    splat_reals(0.8),
];

const OBSTACLE_OFFSETS: [Reals; OBSTACLE_COUNT] = [
    splat_reals(2.0),
    splat_reals(-2.0),
    splat_reals(-4.0),
    splat_reals(4.0),
    splat_reals(-16.0),
    splat_reals(0.0),
    splat_reals(0.0),
];

const SPHERE_RADIUS: f32 = 0.5;

const MIN_TOI: f32 = 0.001;

impl Scene for FixedScene {
    #[inline(always)]
    fn rays_colors(&self, rays: Rays, depth: u32) -> Points {
        let mut projections = RaysProjections::new(rays);
        for _ in 0..depth {
            projections.with_sphere(
                self.sphere_pos,
                SPHERE_RADIUS,
                OBSTACLE_COLORS[Obstacle::Sphere as usize],
                OBSTACLE_REFLECTANCES[Obstacle::Sphere as usize],
            );
            for (obstacle, axis) in [
                (Obstacle::Top, Axis::YS),
                (Obstacle::Bottom, Axis::YS),
                (Obstacle::Left, Axis::XS),
                (Obstacle::Right, Axis::XS),
                (Obstacle::Far, Axis::ZS),
                (Obstacle::Near, Axis::ZS),
            ] {
                let obstacle = obstacle as usize;
                projections.with_axis_aligned_plane(
                    axis,
                    OBSTACLE_OFFSETS[obstacle],
                    OBSTACLE_NORMALS[obstacle],
                    OBSTACLE_COLORS[obstacle],
                    OBSTACLE_REFLECTANCES[obstacle],
                )
            }
            projections.reflect();
        }
        projections.finish(Points::splat(1.0, 1.0, 1.0))
    }
}
