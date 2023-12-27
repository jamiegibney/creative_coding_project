use crate::prelude::*;
use nannou::image::{ImageBuffer, Rgba};
use nannou::prelude::*;
use std::ops::{Add, Rem};

mod grid;
mod process;
mod process_async;
mod state;
mod gpu;

pub use grid::{random_f64, Grid};
pub use process::SmoothLifeGenerator;
pub use process_async::SmoothLifeGeneratorAsync;
pub use state::SLState;
pub use gpu::SmoothLifeGPU;

pub struct SmoothLife {
    generator: SmoothLifeGeneratorAsync,
    rect: Rect,
    texture: wgpu::Texture,
    image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    use_bilinear: bool,
}

impl SmoothLife {
    pub fn new(device: &wgpu::Device, rect: Rect, resolution: usize) -> Self {
        let w = rect.w().floor() as u32;
        let h = rect.h().floor() as u32;

        let mut generator = SmoothLifeGeneratorAsync::new(resolution);
        generator.set_speed(2.0);
        generator.set_state(SLState::fluid());
        generator.set_outer_radius(14.0);

        Self {
            generator,
            rect,
            texture: wgpu::TextureBuilder::new()
                .size([w, h])
                .mip_level_count(4)
                .sample_count(1)
                .format(wgpu::TextureFormat::Rgba8Unorm)
                .usage(
                    wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                )
                .build(device),
            image_buffer: ImageBuffer::from_fn(w, h, |_, _| {
                Rgba([0, 0, 0, u8::MAX])
            }),
            use_bilinear: false,
        }
    }

    /// This function may allocate, and may be an expensive operation.
    pub fn set_resolution(&mut self, device: &wgpu::Device, resolution: usize) {
        let w = self.rect.w() as u32;
        let h = self.rect.h() as u32;

        self.texture = wgpu::TextureBuilder::new()
            .size([w, h])
            .mip_level_count(4)
            .sample_count(1)
            .format(wgpu::TextureFormat::Rgba8Unorm)
            .usage(
                wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .build(device);

        self.image_buffer =
            ImageBuffer::from_fn(w, h, |_, _| Rgba([0, 0, 0, u8::MAX]));
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.generator.set_speed(speed);
    }

    pub fn set_outer_radius(&mut self, ra: f64) {
        self.generator.set_outer_radius(ra);
    }

    pub fn use_bilinear(&mut self, use_bilinear: bool) {
        self.use_bilinear = use_bilinear;
    }

    pub fn is_using_bilinear(&self) -> bool {
        self.use_bilinear
    }

    pub fn reset(&mut self) {
        self.generator.reset();
    }

    fn update_image_buffer(&mut self) {
        let w = self.image_buffer.width() as f64;
        let h = self.image_buffer.height() as f64;

        self.image_buffer
            .enumerate_pixels_mut()
            .for_each(|(x, y, pxl)| {
                let xn = x as f64 / w;
                let yn = y as f64 / h;

                let br = if self.use_bilinear {
                    (self.generator.get_value_bilinear(xn, yn) * 255.0) as u8
                }
                else {
                    (self.generator.get_value_nn(xn, yn) * 255.0) as u8
                };

                pxl.0 = [br, br, br, u8::MAX];
            });
    }
}

impl UIDraw for SmoothLife {
    fn update(&mut self, _: &App, input_data: &InputData) {
        self.generator.update(input_data.delta_time);
        self.update_image_buffer();
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        self.texture.upload_data(
            app.main_window().device(),
            &mut frame.command_encoder(),
            self.image_buffer.as_flat_samples().as_slice(),
        );

        draw.texture(&self.texture)
            .xy(self.rect.xy())
            .wh(self.rect.wh());
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}

impl DrawMask for SmoothLife {
    // fn update(&mut self, delta_time: f64) {
    //     self.generator.update(delta_time);
    //     self.update_image_buffer();
    // }
    //
    // fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
    //     self.texture.upload_data(
    //         app.main_window().device(),
    //         &mut frame.command_encoder(),
    //         self.image_buffer.as_flat_samples().as_slice(),
    //     );
    //
    //     draw.texture(&self.texture)
    //         .xy(self.rect.xy())
    //         .wh(self.rect.wh());
    // }

    fn column_to_mask(&self, mask: &mut crate::dsp::SpectralMask, x: f64) {
        if !(0.0..=1.0).contains(&x) {
            return;
        }

        let sr = unsafe { SAMPLE_RATE };
        let num_bins = mask.len();

        for i in 1..num_bins {
            let bin_freq = mask.bin_freq(i, sr);
            let y = 1.0 - freq_log_norm(bin_freq, 20.0, sr);

            mask[i] = self.generator.get_value_bilinear(x, y);
        }
    }
}
