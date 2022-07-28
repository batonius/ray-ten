use crate::render::{
    splat_reals, update_reals_if, Axis, Integer, Integers, Points, Rays, Reals, ZEROS, ZERO_POINTS,
};
use std::simd::{SimdFloat, SimdPartialEq, SimdPartialOrd, StdFloat};

pub trait Scene {
    fn rays_colors(&self, rays: Rays, depth: usize) -> Points;
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
            obstacle_colors: ZERO_POINTS,
            obstacle_normals: ZERO_POINTS,
            offset_colors: ZERO_POINTS,
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
        if !mask.any() {
            return;
        }
        update_reals_if(&mut self.min_toi, mask, toi);
        self.obstacle_colors.update_if(mask, color);
        update_reals_if(&mut self.obstacle_reflectances, mask, reflectance);

        let mut offset_pois =
            self.rays.origins + self.rays.dirs * toi + Points::splat(1000.0, 1000.0, 1000.0);
        offset_pois *= 1.5;
        let checkered_mask = mask
            & ((offset_pois.xs.cast::<Integer>()
                + offset_pois.ys.cast::<Integer>()
                + offset_pois.zs.cast::<Integer>())
                % Integers::splat(2))
            .simd_eq(Integers::splat(0));

        update_reals_if(&mut self.obstacle_reflectances, checkered_mask, ZEROS);

        self.obstacle_normals.update_if(mask, normal);
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
        if !d.simd_ge(ZEROS).any() {
            return;
        }
        let b = self.rays.dirs.xs * deltas.zs - self.rays.dirs.zs * deltas.xs;
        d -= b * b;
        if !d.simd_ge(ZEROS).any() {
            return;
        }
        let c = self.rays.dirs.ys * deltas.zs - self.rays.dirs.zs * deltas.ys;
        d -= c * c;

        let mask = d.simd_ge(ZEROS);
        if !mask.any() {
            return;
        }

        d = d.simd_max(ZEROS);
        let tts = ZEROS
            - deltas.xs * self.rays.dirs.xs
            - deltas.ys * self.rays.dirs.ys
            - deltas.zs * self.rays.dirs.zs;
        let mut t1s = (tts + d.sqrt()) / dirs_squared_sum;
        let mut t2s = (tts - d.sqrt()) / dirs_squared_sum;
        t1s = t1s.simd_max(ZEROS);
        t2s = t2s.simd_max(ZEROS);
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
    fn reflect(&mut self) -> bool {
        let pois = self.rays.origins + self.rays.dirs * self.min_toi;
        let reflection_dirs = self.rays.dirs
            - (self.obstacle_normals
                * self.rays.dirs.dot(self.obstacle_normals)
                * Reals::splat(2.0));

        self.rays = Rays::new(pois, reflection_dirs);

        self.offset_colors += self.coef_colors * self.obstacle_colors;
        self.coef_colors *= self.obstacle_reflectances;
        self.min_toi = Reals::splat(std::f32::MAX);

        self.obstacle_reflectances.simd_eq(ZEROS).all()
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
    Points::splat(0.8, 0.8, 0.1),
    Points::splat(0.1, 0.8, 0.8),
    Points::splat(0.8, 0.1, 0.8),
    Points::splat(0.8, 0.1, 0.1),
    Points::splat(0.1, 0.1, 0.8),
    Points::splat(0.1, 0.8, 0.1),
    Points::splat(0.1, 0.1, 0.1),
];

const OBSTACLE_REFLECTANCES: [Reals; OBSTACLE_COUNT] = [
    splat_reals(0.3),
    splat_reals(0.3),
    splat_reals(0.3),
    splat_reals(0.3),
    splat_reals(0.3),
    splat_reals(0.3),
    splat_reals(0.5),
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
    fn rays_colors(&self, rays: Rays, depth: usize) -> Points {
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
            projections.with_sphere(
                Points::splat(0.0, 0.0, -19.9),
                4.0,
                ZERO_POINTS,
                Reals::splat(0.0),
            );
            projections.with_sphere(
                Points::splat(0.0, 0.0, 3.9),
                4.0,
                Points::splat(1.0, 1.0, 1.0),
                Reals::splat(0.0),
            );
            if projections.reflect() {
                break;
            }
        }
        projections.finish(Points::splat(1.0, 1.0, 1.0))
    }
}
