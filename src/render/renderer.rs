use crate::{
    math::{Reals, LANES, ZEROS, ZERO_POINTS},
    render::camera::Camera,
    render::tracer::trace_rays,
    scene::Scene,
};

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

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
        let mut x_deltas = vec![ZEROS; samples_per_pixel];
        let mut y_deltas = vec![ZEROS; samples_per_pixel];

        spread_samples((0.0, 1.0), (1.0, 0.0), &mut x_deltas, &mut y_deltas);

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
        #[cfg(not(target_arch = "wasm32"))]
        let chunks = buffer.par_chunks_exact_mut(LANES);
        #[cfg(target_arch = "wasm32")]
        let chunks = buffer.chunks_exact_mut(LANES);
        chunks.enumerate().for_each(|(n, slice)| {
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

fn spread_samples(
    top_left: (f32, f32),
    bottom_right: (f32, f32),
    x_deltas: &mut [Reals],
    y_deltas: &mut [Reals],
) {
    let samples_left = x_deltas.len();
    if samples_left == 1 {
        x_deltas[0] = Reals::splat((top_left.0 + bottom_right.0) / 2.0);
        y_deltas[0] = Reals::splat((top_left.1 + bottom_right.1) / 2.0);
    } else {
        let samples_count_a = samples_left / 2;
        let (x_deltas_a, x_deltas_b) = x_deltas.split_at_mut(samples_count_a);
        let (y_deltas_a, y_deltas_b) = y_deltas.split_at_mut(samples_count_a);
        let width = bottom_right.0 - top_left.0;
        let height = top_left.1 - bottom_right.1;
        if width > height {
            spread_samples(
                top_left,
                (top_left.0 + width / 2.0, bottom_right.1),
                x_deltas_a,
                y_deltas_a,
            );
            spread_samples(
                (top_left.0 + width / 2.0, top_left.1),
                bottom_right,
                x_deltas_b,
                y_deltas_b,
            );
        } else {
            spread_samples(
                top_left,
                (bottom_right.0, top_left.1 + height / 2.0),
                x_deltas_a,
                y_deltas_a,
            );
            spread_samples(
                (top_left.0, top_left.1 + height / 2.0),
                bottom_right,
                x_deltas_b,
                y_deltas_b,
            );
        }
    }
}
