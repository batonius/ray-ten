use crate::{color::Color, Buffer, Camera, Scene};
use image::Rgb;
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::thread_rng;

pub fn render(
    scene: &impl Scene,
    camera: &Camera,
    buffer: &mut Buffer,
    samples_per_pixel: u32,
    max_depth: u32,
) {
    let (width, height) = buffer.dimensions();
    let mut rng = thread_rng();
    let unit_distr = Uniform::new(0.0f32, 1.0f32);

    for x in 0..width {
        for y in 0..height {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..samples_per_pixel {
                let ray = camera.pixel_ray(
                    (x as f32 + unit_distr.sample(&mut rng)) / width as f32,
                    (y as f32 + unit_distr.sample(&mut rng)) / height as f32,
                );
                pixel_color += scene.ray_color(ray, max_depth);
            }
            let pixel = Rgb((pixel_color * (1.0 / samples_per_pixel as f32))
                .sqrt()
                .normalize()
                .into_rgb());
            buffer.put_pixel(x, y, pixel);
        }
    }
}
