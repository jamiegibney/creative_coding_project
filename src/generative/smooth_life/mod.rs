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
    image_buffer: RefCell<ImageBuffer<Rgba<u8>, Vec<u8>>>,
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
            gen: SmoothLifeGenerator::new(w as usize),
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
            image_buffer: RefCell::new(ImageBuffer::from_fn(w, h, |_, _| {
                Rgba([255, 255, 255, u8::MAX])
            })),
        }
    }

    pub fn update(&mut self) {
        self.gen.update();
        // TODO copy grid data to pxls:

        let grid = self.gen.grid();

        self.image_buffer
            .borrow_mut()
            .enumerate_pixels_mut()
            .for_each(|(x, y, pxl)| {
                //
            });
    }
}
