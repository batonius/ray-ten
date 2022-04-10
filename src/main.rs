use anyhow::Result;
use image::{ImageBuffer, Rgb};
use std::time::Instant;

mod camera;
mod color;
mod material;
mod render;
mod scene;

use camera::Camera;
use render::render;
use scene::Scene;

const IMAGE_WIDTH: u32 = 1600;
const IMAGE_HEIGHT: u32 = 900;
const MAX_DEPTH: u32 = 10;
const SAMPLES_PER_PIXEL: u32 = 64;

pub type Buffer = ImageBuffer<Rgb<u8>, Vec<u8>>;

fn main() -> Result<()> {
    let mut buffer = Buffer::from_pixel(IMAGE_WIDTH, IMAGE_HEIGHT, Rgb([255u8, 255, 255]));
    let camera = Camera::new(IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32);
    let scene = Scene::new();
    let now = Instant::now();
    render(&scene, &camera, &mut buffer, SAMPLES_PER_PIXEL, MAX_DEPTH);
    println!("Elapsed {}us", now.elapsed().as_micros());
    buffer.save("out.png")?;
    Ok(())
}
