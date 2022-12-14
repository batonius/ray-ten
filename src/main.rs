#![feature(portable_simd)]

use macroquad::prelude::*;

#[macro_use]
extern crate lazy_static;

#[cfg(not(target_arch = "wasm32"))]
const IMAGE_WIDTH: u16 = 1600;
#[cfg(target_arch = "wasm32")]
const IMAGE_WIDTH: u16 = 640;
#[cfg(not(target_arch = "wasm32"))]
const IMAGE_HEIGHT: u16 = 900;
#[cfg(target_arch = "wasm32")]
const IMAGE_HEIGHT: u16 = 360;

mod game_driver;
mod math;
mod motion;
mod render;
mod scene;
mod ui;

fn window_conf() -> Conf {
    Conf {
        window_title: "ray ten".to_owned(),
        fullscreen: false,
        window_height: IMAGE_HEIGHT as i32,
        window_width: IMAGE_WIDTH as i32,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game_driver = game_driver::GameDriver::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    loop {
        if !game_driver.next_frame() {
            break;
        }
        next_frame().await
    }
}
