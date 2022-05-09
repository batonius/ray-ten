use crate::color::Color;
use crate::simd::{Ints, Mask, Points, Rays, Reals};
use std::io::Read;
use std::simd::StdFloat;

use super::LANES;

pub trait Scene {
    fn rays_colors(&self, rays: Rays, depth: u32) -> Points;
}

pub struct FixedScene {
    sphere_pos: Points,
}

impl FixedScene {
    pub fn new() -> Self {
        FixedScene {
            sphere_pos: Points::splat(-1.0, 0.7, -4.0),
        }
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

const OBSTACLE_COUNT: usize = Obstacle::None as usize + 1;
const EPSILON: f32 = 0.001;
const EPSILONS: Reals = Reals::splat(EPSILON);
const OBSTACLE_NORMALS: [Points; OBSTACLE_COUNT] = [
    Points::splat(0.0, -1.0, 0.0),
    Points::splat(0.0, 1.0, 0.0),
    Points::splat(1.0, 0.0, 0.0),
    Points::splat(-1.0, 0.0, 0.0),
    Points::splat(0.0, 0.0, 1.0),
    Points::splat(0.0, 0.0, -1.0),
    Points::splat(0.0, 0.0, 0.0),
    Points::splat(0.0, 0.0, 0.0),
];

const OBSTACLE_COLORS: [Points; OBSTACLE_COUNT] = [
    Points::splat(0.5, 0.5, 0.0),
    Points::splat(0.0, 0.5, 0.5),
    Points::splat(0.5, 0.0, 0.5),
    Points::splat(0.5, 0.0, 0.0),
    Points::splat(0.0, 0.0, 0.5),
    Points::splat(0.0, 0.5, 0.0),
    Points::splat(0.5, 0.5, 0.5),
    Points::splat(1.0, 1.0, 1.0),
];

const OBSTACLE_OFFSETS: [Reals; OBSTACLE_COUNT] = [
    Reals::splat(2.0),
    Reals::splat(-2.0),
    Reals::splat(-2.0),
    Reals::splat(2.0),
    Reals::splat(-8.0),
    Reals::splat(0.0),
    Reals::splat(0.0),
    Reals::splat(0.0),
];

const SPHERE_RADIUS: f32 = 0.5;

const MIN_TOI: f32 = 0.001;

#[inline(always)]
fn intersect_plane(
    obstacle: Obstacle,
    min_toi: &mut Reals,
    obstacle_colors: &mut Points,
    obstacle_normals: &mut Points,
    origins: &Reals,
    dirs: &Reals,
) {
    let toi = (OBSTACLE_OFFSETS[obstacle as usize] - origins) / dirs;
    let mask = toi.lanes_gt(Reals::splat(MIN_TOI)) & toi.lanes_lt(*min_toi);
    *min_toi = mask.select(toi, *min_toi);

    obstacle_colors.xs = mask.select(OBSTACLE_COLORS[obstacle as usize].xs, obstacle_colors.xs);
    obstacle_colors.ys = mask.select(OBSTACLE_COLORS[obstacle as usize].ys, obstacle_colors.ys);
    obstacle_colors.zs = mask.select(OBSTACLE_COLORS[obstacle as usize].zs, obstacle_colors.zs);

    obstacle_normals.xs = mask.select(OBSTACLE_NORMALS[obstacle as usize].xs, obstacle_normals.xs);
    obstacle_normals.ys = mask.select(OBSTACLE_NORMALS[obstacle as usize].ys, obstacle_normals.ys);
    obstacle_normals.zs = mask.select(OBSTACLE_NORMALS[obstacle as usize].zs, obstacle_normals.zs);
}

#[inline(always)]
fn intersect_sphere(
    sphere_pos: &Points,
    min_toi: &mut Reals,
    obstacle_colors: &mut Points,
    obstacle_normals: &mut Points,
    origins: &Points,
    dirs: &Points,
) -> Mask {
    let dirs_squared = dirs * dirs;
    let dirs_squared_sum = dirs_squared.xs + dirs_squared.ys + dirs_squared.zs;
    let deltas = origins - sphere_pos;
    let r_squared = Reals::splat(SPHERE_RADIUS * SPHERE_RADIUS);
    let mut d = r_squared * dirs_squared_sum;
    let a = dirs.xs * deltas.ys - dirs.ys * deltas.xs;
    let b = dirs.xs * deltas.zs - dirs.zs * deltas.xs;
    let c = dirs.ys * deltas.zs - dirs.zs * deltas.ys;
    d -= a * a;
    d -= b * b;
    d -= c * c;
    let mask = d.lanes_ge(Reals::splat(0.0));

    if !mask.any() {
        return mask;
    }

    d = d.max(Reals::splat(0.0));
    let tts = Reals::splat(0.0) - deltas.xs * dirs.xs - deltas.ys * dirs.ys - deltas.zs * dirs.zs;
    let mut t1s = (tts + d.sqrt()) / dirs_squared_sum;
    let mut t2s = (tts - d.sqrt()) / dirs_squared_sum;
    t1s = t1s.max(Reals::splat(0.0));
    t2s = t2s.max(Reals::splat(0.0));
    let toi = t1s.min(t2s);
    let mask = mask & toi.lanes_gt(Reals::splat(MIN_TOI)) & toi.lanes_lt(*min_toi);

    *min_toi = mask.select(toi, *min_toi);

    obstacle_colors.xs = mask.select(
        OBSTACLE_COLORS[Obstacle::Sphere as usize].xs,
        obstacle_colors.xs,
    );
    obstacle_colors.ys = mask.select(
        OBSTACLE_COLORS[Obstacle::Sphere as usize].ys,
        obstacle_colors.ys,
    );
    obstacle_colors.zs = mask.select(
        OBSTACLE_COLORS[Obstacle::Sphere as usize].zs,
        obstacle_colors.zs,
    );

    let pois = origins + &(dirs * &*min_toi);
    let mut normals = &pois - &sphere_pos;
    let magnitudes =
        (normals.xs * normals.xs + normals.ys * normals.ys + normals.zs * normals.zs).sqrt();
    normals /= &magnitudes;

    obstacle_normals.xs = mask.select(normals.xs, obstacle_normals.xs);
    obstacle_normals.ys = mask.select(normals.ys, obstacle_normals.ys);
    obstacle_normals.zs = mask.select(normals.zs, obstacle_normals.zs);

    return mask;
}

impl Scene for FixedScene {
    #[inline(always)]
    fn rays_colors(&self, mut rays: Rays, depth: u32) -> Points {
        let mut offset_colors = Points::splat(0.0, 0.0, 0.0);
        let mut coef_colors = Points::splat(1.0, 1.0, 1.0);
        let base_colors = Points::splat(1.0, 1.0, 1.0);

        for _ in 0..depth {
            let mut min_toi = Reals::splat(std::f32::MAX);
            let mut obstacle_colors = Points::splat(0.0, 0.0, 0.0);
            let mut obstacle_normals = Points::splat(0.0, 0.0, 0.0);

            let sphere_mask = intersect_sphere(
                &self.sphere_pos,
                &mut min_toi,
                &mut obstacle_colors,
                &mut obstacle_normals,
                &rays.origins,
                &rays.dirs,
            );

            if !sphere_mask.all() {
                intersect_plane(
                    Obstacle::Right,
                    &mut min_toi,
                    &mut obstacle_colors,
                    &mut obstacle_normals,
                    &rays.origins.xs,
                    &rays.dirs.xs,
                );

                intersect_plane(
                    Obstacle::Left,
                    &mut min_toi,
                    &mut obstacle_colors,
                    &mut obstacle_normals,
                    &rays.origins.xs,
                    &rays.dirs.xs,
                );

                intersect_plane(
                    Obstacle::Top,
                    &mut min_toi,
                    &mut obstacle_colors,
                    &mut obstacle_normals,
                    &rays.origins.ys,
                    &rays.dirs.ys,
                );

                intersect_plane(
                    Obstacle::Bottom,
                    &mut min_toi,
                    &mut obstacle_colors,
                    &mut obstacle_normals,
                    &rays.origins.ys,
                    &rays.dirs.ys,
                );

                intersect_plane(
                    Obstacle::Far,
                    &mut min_toi,
                    &mut obstacle_colors,
                    &mut obstacle_normals,
                    &rays.origins.zs,
                    &rays.dirs.zs,
                );

                intersect_plane(
                    Obstacle::Near,
                    &mut min_toi,
                    &mut obstacle_colors,
                    &mut obstacle_normals,
                    &rays.origins.zs,
                    &rays.dirs.zs,
                );
            }

            let pois = &rays.origins + &(&rays.dirs * &min_toi);
            let reflection_dirs = &rays.dirs
                - &(&(&obstacle_normals * &rays.dirs.dot(&obstacle_normals)) * &Reals::splat(2.0));

            rays = Rays::new(pois, reflection_dirs);

            offset_colors += &(&coef_colors * &obstacle_colors);
            coef_colors *= &Points::splat(0.5, 0.5, 0.5);
        }

        coef_colors *= &base_colors;
        offset_colors += &coef_colors;
        offset_colors
    }
}
