use crate::{
    render::camera::Camera,
    render::scene::Scene,
    render::{Points, Reals, LANES},
};
use rayon::prelude::*;

use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::thread_rng;

pub struct Renderer {
    dimensions: (u16, u16),
    samples_per_pixel: u32,
    max_depth: u32,
    x_deltas: Vec<Reals>,
    y_deltas: Vec<Reals>,
}

impl Renderer {
    pub fn new(dimensions: (u16, u16), samples_per_pixel: u32, max_depth: u32) -> Self {
        let mut rng = thread_rng();
        let unit_distr = Uniform::new(0.0f32, 1.0f32);
        let mut x_deltas = vec![Reals::splat(0.0); samples_per_pixel as usize];
        let mut y_deltas = vec![Reals::splat(0.0); samples_per_pixel as usize];

        for sample in 0..samples_per_pixel {
            for i in 0..LANES {
                x_deltas[sample as usize][i] = unit_distr.sample(&mut rng);
                y_deltas[sample as usize][i] = unit_distr.sample(&mut rng);
            }
        }

        Self {
            dimensions,
            samples_per_pixel,
            max_depth,
            x_deltas,
            y_deltas,
        }
    }

    pub fn render<S>(&self, scene: &S, camera: &Camera, buffer: &mut [[u8; 4]])
    where
        S: Scene + Sync + Send,
    {
        let (width, height) = self.dimensions;

        let lanes_per_line = width as usize / LANES;

        buffer
            .par_chunks_exact_mut(LANES)
            .enumerate()
            .for_each(|(n, slice)| {
                let y = n / lanes_per_line;
                let x = n % lanes_per_line;
                let mut pixels_colors = Points::splat(0.0, 0.0, 0.0);
                for sample in 0..self.samples_per_pixel {
                    let mut x_offsets = Reals::splat(0.0);
                    let mut y_offsets = Reals::splat(y as f32);
                    for i in 0..LANES {
                        x_offsets[i] = (x * LANES + i) as f32;
                    }

                    x_offsets += &self.x_deltas[sample as usize];
                    y_offsets += &self.y_deltas[sample as usize];
                    x_offsets /= Reals::splat(width as f32);
                    y_offsets /= Reals::splat(height as f32);

                    let rays = camera.pixel_rays(x_offsets, y_offsets);
                    pixels_colors += scene.rays_colors(rays, self.max_depth);
                }
                pixels_colors /= Reals::splat(self.samples_per_pixel as f32);
                pixels_colors = pixels_colors /*.sqrt()*/
                    .normalize();
                for i in 0..LANES {
                    slice[i][0] = (pixels_colors.xs[i] * 255.0) as u8;
                    slice[i][1] = (pixels_colors.ys[i] * 255.0) as u8;
                    slice[i][2] = (pixels_colors.zs[i] * 255.0) as u8;
                }
            });
    }
}
