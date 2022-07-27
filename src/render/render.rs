use crate::{
    render::camera::Camera,
    render::scene::Scene,
    render::{Points, Reals, LANES},
};
use rayon::prelude::*;

use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::thread_rng;

pub fn render<S>(
    scene: &S,
    camera: &Camera,
    buffer: &mut [[u8; 4]],
    dimensions: (u16, u16),
    samples_per_pixel: u32,
    max_depth: u32,
) where
    S: Scene + Sync + Send,
{
    let (width, height) = dimensions;
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

    let lanes_per_line = width as usize / LANES;

    buffer
        .par_chunks_exact_mut(LANES)
        .enumerate()
        .for_each(|(n, slice)| {
            let y = n / lanes_per_line;
            let x = n % lanes_per_line;
            let mut pixels_colors = Points::splat(0.0, 0.0, 0.0);
            for sample in 0..samples_per_pixel {
                let mut x_offsets = Reals::splat(0.0);
                let mut y_offsets = Reals::splat(y as f32);
                for i in 0..LANES {
                    x_offsets[i] = (x * LANES + i) as f32;
                }

                x_offsets += &x_deltas[sample as usize];
                y_offsets += &y_deltas[sample as usize];

                x_offsets /= Reals::splat(width as f32);
                y_offsets /= Reals::splat(height as f32);
                let rays = camera.pixel_rays(&x_offsets, &y_offsets);
                pixels_colors += scene.rays_colors(rays, max_depth);
            }
            pixels_colors /= Reals::splat(samples_per_pixel as f32);
            pixels_colors = pixels_colors.sqrt().normalize();
            for i in 0..LANES {
                slice[i][0] = (pixels_colors.xs[i] * 255.0) as u8;
                slice[i][1] = (pixels_colors.ys[i] * 255.0) as u8;
                slice[i][2] = (pixels_colors.zs[i] * 255.0) as u8;
            }
        });
}
