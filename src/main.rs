#![feature(portable_simd)]

use anyhow::Result;
use image::{
    codecs::gif::{GifEncoder, Repeat},
    Frame, ImageBuffer, Rgba,
};
use std::fs::File;

const IMAGE_WIDTH: u32 = 1600;
const IMAGE_HEIGHT: u32 = 900;
const MAX_DEPTH: u32 = 5;
const SAMPLES_PER_PIXEL: u32 = 16;

pub type Buffer = ImageBuffer<Rgba<u8>, Vec<u8>>;

mod render;

use render::{camera::Camera, render::render, scene::FixedScene};

fn main() -> Result<()> {
    let mut camera = Camera::new(IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32, 2.0f32);
    let mut scene = FixedScene::new();
    let mut buffer = Buffer::from_pixel(IMAGE_WIDTH, IMAGE_HEIGHT, Rgba([255u8; 4]));

    // scene.move_sphere(0.05, 0.05, 0.05);
    // render(&scene, &camera, &mut buffer, SAMPLES_PER_PIXEL, MAX_DEPTH);
    // buffer.save("out.png")?;

    let mut frame = Frame::new(buffer);
    let file = File::create("out.gif")?;
    let mut encoder = GifEncoder::new_with_speed(file, 30);
    encoder.set_repeat(Repeat::Infinite)?;
    for _ in 0..100 {
        render(
            &scene,
            &camera,
            frame.buffer_mut(),
            SAMPLES_PER_PIXEL,
            MAX_DEPTH,
        );
        camera.move_origin(-0.05, 0.05);
        scene.move_sphere(0.05, 0.05, 0.07);
        encoder.encode_frame(frame.clone())?;
    }

    Ok(())
}
