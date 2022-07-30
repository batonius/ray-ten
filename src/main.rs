#![feature(portable_simd)]

use macroquad::prelude::*;

const IMAGE_WIDTH: u16 = 1600;
const IMAGE_HEIGHT: u16 = 900;
const MAX_DEPTH: usize = 5;
const SAMPLES_PER_PIXEL: usize = 4;

mod ai;
mod math;
mod motion;
mod render;
mod scene;

use ai::control_far_paddle;
use motion::{MotionResult, MotionTicker, PaddleControls};
use render::{camera::Camera, renderer::Renderer};
use scene::{Obstacle, Plane, Scene, Sphere};

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
    let mut scene = Scene::new();
    let mut motion_ticker = MotionTicker::new();

    let renderer = Renderer::new((IMAGE_WIDTH, IMAGE_HEIGHT), SAMPLES_PER_PIXEL, MAX_DEPTH);
    let mut image = Image::gen_image_color(IMAGE_WIDTH, IMAGE_HEIGHT, WHITE);
    let texture = Texture2D::from_image(&image);

    let mut time_counter = 0usize;
    let mut sum_time: f32 = 0.0;
    let mut cpu_score = 0usize;
    let mut player_score = 0usize;

    let font = load_ttf_font_from_bytes(include_bytes!("../assets/LiberationMono-Regular.ttf"))
        .expect("Can't load fon&t.");

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        let player_paddle_controls = PaddleControls::new(
            is_key_down(KeyCode::Up),
            is_key_down(KeyCode::Down),
            is_key_down(KeyCode::Left),
            is_key_down(KeyCode::Right),
        );
        let ai_paddle_controls = control_far_paddle(&scene);
        let motion_result = motion_ticker.tick(
            &mut scene,
            get_frame_time(),
            player_paddle_controls,
            ai_paddle_controls,
        );

        match motion_result {
            MotionResult::Colision(Obstacle::Plane(Plane::Near)) => cpu_score += 1,
            MotionResult::Colision(Obstacle::Plane(Plane::Far)) => player_score += 1,
            _ => (),
        }

        // if let MotionResult::Colision(Plane::Near) = motion_result
        let near_paddle_pos = scene.sphere_pos(Sphere::NearPaddle);
        camera.move_origin_to(near_paddle_pos.x(), near_paddle_pos.y());
        renderer.render(&scene, &camera, image.get_image_data_mut());
        texture.update(&image);
        draw_texture(texture, 0.0, 0.0, WHITE);

        let player_score_text = format!("YOU:{player_score:03}");
        let cpu_score_text = format!("CPU:{cpu_score:03}");

        draw_text_ex(
            player_score_text.as_str(),
            12.,
            82.,
            TextParams {
                font_size: 100,
                color: BLACK,
                font: font,
                ..Default::default()
            },
        );
        draw_text_ex(
            player_score_text.as_str(),
            10.,
            80.,
            TextParams {
                font_size: 100,
                color: RED,
                font: font,
                ..Default::default()
            },
        );
        draw_text_ex(
            cpu_score_text.as_str(),
            IMAGE_WIDTH as f32 - 450.,
            IMAGE_HEIGHT as f32 - 80.,
            TextParams {
                font_size: 100,
                color: BLACK,
                font: font,
                ..Default::default()
            },
        );
        draw_text_ex(
            cpu_score_text.as_str(),
            IMAGE_WIDTH as f32 - 452.,
            IMAGE_HEIGHT as f32 - 82.,
            TextParams {
                font_size: 100,
                color: BLUE,
                font: font,
                ..Default::default()
            },
        );
        time_counter += 1;
        sum_time += get_frame_time();
        if time_counter % 100 == 0 {
            println!(
                "After {} frames, avg {}",
                time_counter,
                sum_time / time_counter as f32
            );
        }

        next_frame().await
    }
}
