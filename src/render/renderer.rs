use crate::{
    math::{Reals, LANES, ZEROS, ZERO_POINTS},
    render::camera::Camera,
    render::tracer::trace_rays,
    scene::Scene,
};
use rayon::prelude::*;

use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::thread_rng;

pub struct Renderer {
    width: f32,
    height: f32,
    lanes_per_line: usize,
    samples_per_pixel: usize,
    max_depth: usize,
    x_deltas: Vec<Reals>,
    y_deltas: Vec<Reals>,
}

impl Renderer {
    pub fn new(dimensions: (u16, u16), samples_per_pixel: usize, max_depth: usize) -> Self {
        let (width, height) = dimensions;
        let lanes_per_line = width as usize / LANES;
        let mut rng = thread_rng();
        let unit_distr = Uniform::new(0.0f32, 1.0f32);
        let mut x_deltas = vec![ZEROS; samples_per_pixel];
        let mut y_deltas = vec![ZEROS; samples_per_pixel];

        for sample in 0..samples_per_pixel {
            for i in 0..LANES {
                x_deltas[sample][i] = unit_distr.sample(&mut rng);
                y_deltas[sample][i] = unit_distr.sample(&mut rng);
            }
        }

        Self {
            width: width as f32,
            height: height as f32,
            lanes_per_line,
            samples_per_pixel,
            max_depth,
            x_deltas,
            y_deltas,
        }
    }

    pub fn render(&self, scene: &Scene, camera: &Camera, coef: f32, buffer: &mut [[u8; 4]]) {
        buffer
            .par_chunks_exact_mut(LANES)
            .enumerate()
            .for_each(|(n, slice)| {
                let y = n / self.lanes_per_line;
                let x = n % self.lanes_per_line;
                let mut pixels_colors = ZERO_POINTS;
                for sample in 0..self.samples_per_pixel {
                    let mut x_offsets = ZEROS;
                    let mut y_offsets = Reals::splat(y as f32);
                    for i in 0..LANES {
                        x_offsets[i] = (x * LANES + i) as f32;
                    }

                    x_offsets += &self.x_deltas[sample];
                    y_offsets += &self.y_deltas[sample];
                    x_offsets /= Reals::splat(self.width);
                    y_offsets /= Reals::splat(self.height);

                    let rays = camera.pixel_rays(x_offsets, y_offsets);
                    pixels_colors += trace_rays(scene, rays, self.max_depth);
                }
                pixels_colors /= Reals::splat(self.samples_per_pixel as f32);
                pixels_colors = pixels_colors.sqrt().normalize();
                for (i, pixel) in slice.iter_mut().enumerate().take(LANES) {
                    pixel[0] = (pixels_colors.xs[i] * 255.0 * coef) as u8;
                    pixel[1] = (pixels_colors.ys[i] * 255.0 * coef) as u8;
                    pixel[2] = (pixels_colors.zs[i] * 255.0 * coef) as u8;
                }
            });
    }
}
