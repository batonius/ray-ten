#![feature(portable_simd)]

use macroquad::prelude::*;

const IMAGE_WIDTH: u16 = 1600;
const IMAGE_HEIGHT: u16 = 900;
const MAX_DEPTH: usize = 5;
const SAMPLES_PER_PIXEL: usize = 4;

mod render;

use render::{camera::Camera, renderer::Renderer, scene::FixedScene};

fn window_conf() -> Conf {
    Conf {
        window_title: "ray ten".to_owned(),
        fullscreen: false,
        window_height: IMAGE_HEIGHT as i32,
        window_width: IMAGE_WIDTH as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut camera = Camera::new(IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32, 2.0f32);
    let mut scene = FixedScene::new();
    let renderer = Renderer::new((IMAGE_WIDTH, IMAGE_HEIGHT), SAMPLES_PER_PIXEL, MAX_DEPTH);
    let mut image = Image::gen_image_color(IMAGE_WIDTH, IMAGE_HEIGHT, WHITE);
    let texture = Texture2D::from_image(&image);

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        let mut cam_delta = (0f32, 0f32);
        let mut sphere_delta = (0f32, 0f32, 0f32);
        if is_key_down(KeyCode::Up) {
            cam_delta.1 += 0.4;
        }
        if is_key_down(KeyCode::Down) {
            cam_delta.1 -= 0.4;
        }
        if is_key_down(KeyCode::Left) {
            cam_delta.0 -= 0.4;
        }
        if is_key_down(KeyCode::Right) {
            cam_delta.0 += 0.4;
        }
        if is_key_down(KeyCode::S) {
            sphere_delta.2 += 0.02;
        }
        if is_key_down(KeyCode::W) {
            sphere_delta.2 -= 0.02;
        }
        if is_key_down(KeyCode::Q) {
            sphere_delta.1 += 0.02;
        }
        if is_key_down(KeyCode::E) {
            sphere_delta.1 -= 0.02;
        }
        if is_key_down(KeyCode::D) {
            sphere_delta.0 += 0.02;
        }
        if is_key_down(KeyCode::A) {
            sphere_delta.0 -= 0.02;
        }
        camera.move_origin(
            get_frame_time() * cam_delta.0,
            get_frame_time() * cam_delta.1,
        );
        scene.move_sphere(sphere_delta.0, sphere_delta.1, sphere_delta.2);
        renderer.render(&scene, &camera, image.get_image_data_mut());
        texture.update(&image);
        draw_texture(texture, 0.0, 0.0, WHITE);
        draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 32., BLACK);
        next_frame().await
    }
}
