use crate::math::{
    update_reals_if, Axis, Color, Colors, Integer, Integers, Point, Points, Rays, Real, Reals,
    Vector, Vectors, ZEROS, ZERO_POINTS,
};
use crate::scene::{Obstacle, Scene};
use std::simd::{SimdFloat, SimdPartialEq, SimdPartialOrd, StdFloat};

#[inline(never)]
pub fn trace_rays(scene: &Scene, rays: Rays, max_depth: usize) -> Colors {
    let mut projections = RaysProjections::new(rays, max_depth);
    loop {
        for sphere in scene.spheres() {
            projections.with_sphere(
                scene.sphere_pos(*sphere),
                scene.sphere_radius(*sphere),
                scene.obstacle_color(Obstacle::Sphere(*sphere)),
                scene.obstacle_reflectance(Obstacle::Sphere(*sphere)),
            );
        }
        for plane in scene.planes() {
            projections.with_axis_aligned_plane(
                scene.plane_alignment_axis(*plane),
                scene.plane_offset(*plane),
                scene.plane_normal(*plane),
                scene.obstacle_color(Obstacle::Plane(*plane)),
                scene.obstacle_reflectance(Obstacle::Plane(*plane)),
            );
        }
        if projections.reflect() {
            break;
        }
    }
    projections.finish(Colors::splat(1.0, 1.0, 1.0))
}

struct RaysProjections {
    rays: Rays,
    min_toi: Reals,
    obstacle_reflectances: Reals,
    obstacle_colors: Colors,
    obstacle_normals: Vectors,
    offset_colors: Colors,
    coef_colors: Colors,
    depth_left: usize,
}

const MIN_TOI: f32 = 0.001f32;

impl RaysProjections {
    fn new(rays: Rays, max_depth: usize) -> RaysProjections {
        RaysProjections {
            rays,
            min_toi: Reals::splat(std::f32::MAX),
            obstacle_reflectances: Reals::splat(std::f32::MAX),
            obstacle_colors: ZERO_POINTS,
            obstacle_normals: ZERO_POINTS,
            offset_colors: ZERO_POINTS,
            coef_colors: Colors::splat(1.0, 1.0, 1.0),
            depth_left: max_depth,
        }
    }

    fn with_axis_aligned_plane(
        &mut self,
        axis: Axis,
        offset_within_axis: Real,
        normal: Vector,
        color: Color,
        reflectance: Real,
    ) {
        let offset_within_axis = Reals::splat(offset_within_axis);
        let normal = Vectors::from_single(normal);
        let color = Colors::from_single(color);
        let reflectance = Reals::splat(reflectance);

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

    fn with_sphere(
        &mut self,
        sphere_pos: Point,
        sphere_radius: Real,
        color: Color,
        reflectance: Real,
    ) {
        let sphere_pos = Colors::from_single(sphere_pos);
        let color = Colors::from_single(color);
        let reflectance = Reals::splat(reflectance);
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
        normals /= Reals::splat(sphere_radius);

        self.obstacle_normals.update_if(mask, normals);
        update_reals_if(&mut self.obstacle_reflectances, mask, reflectance);
    }

    fn reflect(&mut self) -> bool {
        self.offset_colors += self.coef_colors * self.obstacle_colors;
        self.coef_colors *= self.obstacle_reflectances;

        self.depth_left -= 1;

        if self.depth_left == 0 || self.obstacle_reflectances.simd_eq(ZEROS).all() {
            return true;
        }

        let pois = self.rays.origins + self.rays.dirs * self.min_toi;
        let reflection_dirs = self.rays.dirs
            - (self.obstacle_normals
                * self.rays.dirs.dot(self.obstacle_normals)
                * Reals::splat(2.0));

        self.rays = Rays::new(pois, reflection_dirs);

        self.min_toi = Reals::splat(std::f32::MAX);

        false
    }

    fn finish(mut self, base_colors: Colors) -> Colors {
        self.coef_colors *= base_colors;
        self.offset_colors += self.coef_colors;
        self.offset_colors
    }
}
