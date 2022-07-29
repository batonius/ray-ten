use crate::math::{Point, Points, Rays, Real, Reals};

pub struct Camera {
    origin: Point,
    view_port_base: Point,
    view_port_x_axis: Point,
    view_port_y_axis: Point,
}

impl Camera {
    pub fn new(aspect_ratio: f32, width: f32) -> Camera {
        let view_port_width = width;
        let view_port_height = view_port_width / aspect_ratio;

        Camera {
            origin: Point::new(1.0, -1.0, 0.0),
            view_port_base: Point::new(-view_port_width / 2.0, view_port_height / 2.0, -1.0),
            view_port_x_axis: Point::new(view_port_width, 0.0, 0.0),
            view_port_y_axis: Point::new(0.0, -view_port_height, 0.0),
        }
    }

    pub fn move_origin(&mut self, delta_x: Real, delta_y: Real) {
        *self.origin.x_mut() += delta_x;
        *self.origin.y_mut() += delta_y;
    }

    pub fn pixel_rays(&self, x_offsets: Reals, y_offsets: Reals) -> Rays {
        let mut dirs = Points::from_single(self.view_port_base);
        dirs += Points::from_single(self.view_port_x_axis) * x_offsets;
        dirs += Points::from_single(self.view_port_y_axis) * y_offsets;
        Rays {
            origins: Points::from_single(self.origin),
            dirs,
        }
    }
}
