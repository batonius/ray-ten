use crate::simd::{Points, Rays, Reals};

pub struct Camera {
    origin: Points,
    view_port_base: Points,
    view_port_x_axis: Points,
    view_port_y_axis: Points,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Camera {
        let view_port_width = 2.0;
        let view_port_height = view_port_width / aspect_ratio;

        Camera {
            origin: Points::splat(0.0, 0.0, 0.0),
            view_port_base: Points::splat(-view_port_width / 2.0, view_port_height / 2.0, -1.0),
            view_port_x_axis: Points::splat(view_port_width, 0.0, 0.0),
            view_port_y_axis: Points::splat(0.0, -view_port_height, 0.0),
        }
    }

    pub fn pixel_rays(&self, x_offsets: &Reals, y_offsets: &Reals) -> Rays {
        let mut dirs = self.view_port_base.clone();
        dirs += &(&self.view_port_x_axis * x_offsets);
        dirs += &(&self.view_port_y_axis * y_offsets);
        Rays {
            origins: self.origin.clone(),
            dirs,
        }
    }
}