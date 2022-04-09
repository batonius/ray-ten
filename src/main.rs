use anyhow::Result;
use image::{ImageBuffer, Rgb};
use std::time::Instant;

mod camera;
mod material;
mod scene;

use scene::Scene;

const IMAGE_WIDTH: u32 = 1600;
const IMAGE_HEIGHT: u32 = 900;
// const MAX_DEPTH: u32 = 5;

pub type Color = Rgb<u8>;
pub type Buffer = ImageBuffer<Color, Vec<u8>>;

// static UNIT_DISTR: Uniform<f32> = Uniform::new(0.0f32, 1.0f32);

fn main() -> Result<()> {
    let mut buffer = Buffer::from_pixel(IMAGE_WIDTH, IMAGE_HEIGHT, Rgb([255u8, 255, 255]));
    let scene = Scene::new(IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32);
    let now = Instant::now();
    scene.render(&mut buffer, 8)?;
    println!("Elapsed {}us", now.elapsed().as_micros());
    buffer.save("out.png")?;
    Ok(())
}
