use crate::prelude::*;
use nannou::image::{ImageBuffer, Rgba};
use nannou::prelude::*;
use std::ops::{Add, Rem};

use std::cell::RefCell;

mod grid;
mod process;
mod state;

use grid::Grid;
use process::SmoothLifeGenerator;
use state::SLState;

pub struct SmoothLife {
    gen: SmoothLifeGenerator,
    texture: wgpu::Texture,
    image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl SmoothLife {
    /// # Panics
    ///
    /// Panics if `rect.w().floor() != rect.h().floor()`.
    pub fn new(device: &wgpu::Device, rect: Rect) -> Self {
        let w = rect.w().floor() as u32;
        let h = rect.h().floor() as u32;
        assert!(epsilon_eq(w as f64, h as f64));

        Self {
            gen: SmoothLifeGenerator::new(100),
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
                Rgba([255, 255, 255, u8::MAX])
            }),
        }
    }
}

impl DrawMask for SmoothLife {
    fn update(&mut self, delta_time: f64) {
        self.gen.update(delta_time);

        let width = self.image_buffer.width() as f64;
        let height = self.image_buffer.height() as f64;

        self.image_buffer
            .enumerate_pixels_mut()
            .for_each(|(x, y, pxl)| {
                let x = x as f64 / width;
                let y = y as f64 / height;
                let value = self.gen.get_value(x, y);

                let br = (value * 255.0) as u8;

                pxl.0 = [br, br, br, u8::MAX];
            });
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        self.texture.upload_data(
            app.main_window().device(),
            &mut frame.command_encoder(),
            self.image_buffer.as_flat_samples().as_slice(),
        );

        draw.texture(&self.texture);
    }

    fn column_to_mask(&mut self, mask: &mut crate::dsp::SpectralMask, x: f64) {
        if !(0.0..=1.0).contains(&x) {
            return;
        }

        let sr = unsafe { SAMPLE_RATE };
        let num_bins = mask.len();

        for i in 1..num_bins {
            let bin_freq = mask.bin_freq(i, sr);
            let y = 1.0 - freq_log_norm(bin_freq, 20.0, sr);

            mask[i] = self.gen.get_value(x, y);
        }
    }
}
