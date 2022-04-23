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
            origin: Point::new(1.0, 0.3, 0.0),
            view_port_base: Vector::new(-view_port_width / 2.0, view_port_height / 2.0, -1.0),
            view_port_x_axis: Vector::new(view_port_width, 0.0, 0.0),
            view_port_y_axis: Vector::new(0.0, -view_port_height, 0.0),
        }
    }

    pub fn pixel_ray(&self, x_offset: f32, y_offset: f32) -> Ray {
        Ray {
            origin: self.origin,
            dir: self.view_port_base
                + (self.view_port_x_axis * x_offset)
                + (self.view_port_y_axis * y_offset),
        }
    }
}
