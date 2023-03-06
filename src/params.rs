use std::ops::Deref;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Viewport {
    pub half_viewport_x: f32,
    pub half_viewport_y: f32,
    pub ratio: f32,
}

impl Viewport {
    pub fn new(window_size: [f32; 2]) -> Self {
        Self {
            half_viewport_x: window_size[0] / 2.,
            half_viewport_y: window_size[1] / 2.,
            ratio: window_size[0] / window_size[1],
        }
    }
}

const ZOOM_FACTOR: f32 = 1.2;

pub enum Zoom {
    In,
    Out,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Coordinates {
    real: f32,
    imag: f32,
    zoom: f32,
}

impl Default for Coordinates {
    fn default() -> Self {
        Self {
            real: 0.0,
            imag: 0.0,
            zoom: 1.5,
        }
    }
}

impl Coordinates {
    pub fn get_complex(&self) -> (f32, f32) {
        (self.real, self.imag)
    }

    pub fn set_complex(mut self, complex: (f32, f32)) -> Self {
        self.real = complex.0;
        self.imag = complex.1;

        self.real = self.real.clamp(-2., 2.);
        self.imag = self.imag.clamp(-2., 2.);

        self
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(mut self, zoom: Zoom, zoom_center: Option<(f32, f32)>) -> Self {
        let old_zoom = self.zoom;

        let factor = match zoom {
            Zoom::In => 1. / ZOOM_FACTOR,
            Zoom::Out => ZOOM_FACTOR,
        };
        self.zoom *= factor;

        self.zoom = self.zoom.clamp(0.00001, 1.5);

        if let Some(zoom_center) = zoom_center {
            let old_scaled_rel_x = zoom_center.0 * old_zoom;
            let old_scaled_rel_y = zoom_center.1 * old_zoom;
            let new_scaled_rel_x = zoom_center.0 * self.zoom;
            let new_scaled_rel_y = zoom_center.1 * self.zoom;

            let new_real = self.real + (old_scaled_rel_x - new_scaled_rel_x);
            let new_imag = self.imag - (old_scaled_rel_y - new_scaled_rel_y);

            self.set_complex((new_real, new_imag))
        } else {
            self
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Iterations(i32);

impl Deref for Iterations {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Iterations {
    pub fn new(value: i32) -> Self {
        Self(value.clamp(0, 10000))
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorParams {
    hue: f32,
    saturation: f32,
    lightness: f32,
    hue_linked: i32,
    saturation_linked: i32,
    lightness_linked: i32,
}

impl ColorParams {
    pub fn new(hue: f32, saturation: f32, lightness: f32) -> Self {
        Self {
            hue,
            saturation,
            lightness,
            hue_linked: 0,
            saturation_linked: 0,
            lightness_linked: 0,
        }
    }

    pub fn get_hsl(&self) -> (f32, f32, f32) {
        (self.hue, self.saturation, self.lightness)
    }

    pub fn set_hsl(mut self, value: (f32, f32, f32)) -> Self {
        self.hue = value.0.clamp(0., 1.0);
        self.saturation = value.1.clamp(0., 1.0);
        self.lightness = value.2.clamp(0., 1.0);

        self
    }

    pub fn get_link(&self) -> (bool, bool, bool) {
        (
            self.hue_linked > 0,
            self.saturation_linked > 0,
            self.lightness_linked > 0,
        )
    }

    pub fn set_link(mut self, value: (bool, bool, bool)) -> Self {
        self.hue_linked = value.0 as i32;
        self.saturation_linked = value.1 as i32;
        self.lightness_linked = value.2 as i32;

        self
    }
}
