use parry3d::math::{Point, Real, Vector};
use parry3d::query::Ray;
pub struct Camera {
    origin: Point<Real>,
    view_port_base: Vector<Real>,
    view_port_x_axis: Vector<Real>,
    view_port_y_axis: Vector<Real>,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Camera {
        let view_port_width = 2.0;
        let view_port_height = view_port_width / aspect_ratio;

        Camera {
            origin: Point::origin(),
            view_port_base: Vector::new(-view_port_width / 2.0, view_port_height / 2.0, -1.0),
            view_port_x_axis: Vector::new(view_port_width, 0.0, 0.0),
            view_port_y_axis: Vector::new(0.0, -view_port_height, 0.0),
        }
    }

    pub fn pixel_ray(
        &self,
        buffer_dims: (u32, u32),
        pixel_pos: (u32, u32),
        subpixel_offsets: (f32, f32),
    ) -> Ray {
        let (width, height) = buffer_dims;
        let (x, y) = pixel_pos;
        let (subpixel_x, subpixel_y) = subpixel_offsets;
        Ray {
            origin: self.origin,
            dir: self.view_port_base
                + (self.view_port_x_axis * (x as f32 + subpixel_x) / width as f32)
                + (self.view_port_y_axis * (y as f32 + subpixel_y) / height as f32),
        }
    }
}
