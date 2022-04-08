use anyhow::Result;
use image::{ImageBuffer, Rgb};
use parry3d::math::{Isometry, Point, Real, Vector};
use parry3d::query::{Ray, RayCast};
use parry3d::shape::Ball;
use std::time::Instant;

const IMAGE_WIDTH: u32 = 1600;
const IMAGE_HEIGHT: u32 = 900;

type Color = Rgb<u8>;
type Buffer = ImageBuffer<Color, Vec<u8>>;

struct Camera {
    pub origin: Point<Real>,
    pub view_port_base: Vector<Real>,
    pub view_port_x_axis: Vector<Real>,
    pub view_port_y_axis: Vector<Real>,
}

impl Camera {
    fn new(aspect_ratio: f32) -> Camera {
        let view_port_width = 2.0;
        let view_port_height = view_port_width / aspect_ratio;

        Camera {
            origin: Point::origin(),
            view_port_base: Vector::new(-view_port_width / 2.0, view_port_height / 2.0, -1.0),
            view_port_x_axis: Vector::new(view_port_width, 0.0, 0.0),
            view_port_y_axis: Vector::new(0.0, -view_port_height, 0.0),
        }
    }

    fn pixel_ray(&self, buffer_dims: (u32, u32), pixel_pos: (u32, u32)) -> Ray {
        let (width, height) = buffer_dims;
        let (x, y) = pixel_pos;
        Ray {
            origin: self.origin,
            dir: self.view_port_base
                + (self.view_port_x_axis * x as f32 / width as f32)
                + (self.view_port_y_axis * y as f32 / height as f32),
        }
    }
}

struct Scene {
    camera: Camera,
    ball_radius: Real,
    ball_translation: Isometry<Real>,
}

impl Scene {
    fn new(aspect_ratio: f32) -> Self {
        Scene {
            camera: Camera::new(aspect_ratio),
            ball_radius: 1.5f32,
            ball_translation: Isometry::translation(0.0, 0.0, -4.0),
        }
    }

    fn render(&self, buffer: &mut Buffer) -> Result<()> {
        let (width, height) = buffer.dimensions();
        for x in 0..width {
            for y in 0..height {
                buffer.put_pixel(
                    x,
                    y,
                    self.ray_color(self.camera.pixel_ray((width, height), (x, y))),
                );
            }
        }
        Ok(())
    }

    fn ray_color(&self, ray: Ray) -> Color {
        if let Some(intersection) = Ball::new(self.ball_radius).cast_ray_and_get_normal(
            &self.ball_translation,
            &ray,
            100000f32,
            true,
        ) {
            Rgb([((intersection.toi - 1.7) * 120.0) as u8, 0, 0])
        } else {
            Rgb([0u8, 0, 0])
        }
    }
}

fn main() -> Result<()> {
    let mut buffer = Buffer::from_pixel(IMAGE_WIDTH, IMAGE_HEIGHT, Rgb([255u8, 255, 255]));
    let scene = Scene::new(IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32);
    let now = Instant::now();
    scene.render(&mut buffer)?;
    println!("Elapsed {}us", now.elapsed().as_micros());
    buffer.save("out.png")?;
    Ok(())
}
