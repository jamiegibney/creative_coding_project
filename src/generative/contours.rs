use crate::dsp::SpectralMask;
use crate::prelude::*;
use crate::util::thread_pool::ThreadPool;
use nannou::image::{ImageBuffer, Rgba};
use nannou::noise::{NoiseFn, Perlin, Seedable};
use nannou::prelude::*;
use std::cell::RefCell;
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};

/// The number of threads to use for computing the contour lines.
const CONTOUR_NUM_THREADS: usize = 16;

/// Perlin noise contour line generator. Runs asynchronously via 16 threads.
// TODO there must be a way of storing the threads rather than spawning them
// each frame? thread pool?
// TODO configurable number of threads (even just when constructing)
pub struct Contours {
    /// Noise generator.
    noise: Arc<Perlin>,
    /// The 3D z value for progressing through the noise field.
    z: f64,
    z_increment: f64,

    /// Contour generation range.
    range: Arc<RangeInclusive<f64>>,
    /// Number of contours to generate.
    num_contours: u32,

    /// A handle to a texture on the GPU.
    texture: wgpu::Texture,

    /// The internal image buffer (the buffer which holds pixel data).
    image_buffer: RefCell<ImageBuffer<Rgba<u8>, Vec<u8>>>,

    column: Vec<Rgba<u8>>,
    prev_col_idx: Option<usize>,

    scratch_buffers: Vec<Arc<Mutex<Vec<Rgba<u8>>>>>,
    thread_pool: ThreadPool,
}

impl Contours {
    /// Creates a new `Contours`.
    ///
    /// This object is responsible for handling its own texture and image buffer.
    pub fn new(device: &wgpu::Device, win_rect: &Rect) -> Self {
        assert!(CONTOUR_NUM_THREADS.is_power_of_two());
        let height_px = win_rect.h() as usize;

        let total_px = win_rect.w() as usize * win_rect.h() as usize;
        let thread_px = total_px / CONTOUR_NUM_THREADS;

        Self {
            noise: Arc::new(Perlin::new().set_seed(random())),
            z: 0.0,
            z_increment: 0.003,

            range: Arc::new(0.0..=0.0),
            num_contours: 1,

            texture: wgpu::TextureBuilder::new()
                .size([win_rect.w() as u32, win_rect.h() as u32])
                .mip_level_count(4)
                .sample_count(1)
                .format(wgpu::TextureFormat::Rgba8Unorm)
                .usage(
                    wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                )
                .build(device),
            image_buffer: RefCell::new(ImageBuffer::from_fn(
                win_rect.w().floor() as u32,
                win_rect.h().floor() as u32,
                |_, _| Rgba([255, 255, 255, u8::MAX]),
            )),

            column: vec![Rgba([255, 255, 255, u8::MAX]); height_px],
            prev_col_idx: None,

            scratch_buffers: {
                let mut v = Vec::with_capacity(CONTOUR_NUM_THREADS);
                (0..CONTOUR_NUM_THREADS).for_each(|_| {
                    v.push(Arc::new(Mutex::new(vec![
                        Rgba([
                            255,
                            255,
                            255,
                            u8::MAX
                        ]);
                        thread_px
                    ])));
                });
                v
            },

            thread_pool: ThreadPool::build(CONTOUR_NUM_THREADS).unwrap(),
        }
    }

    /// Updates the internal image buffer and noise generator.
    pub fn update(&mut self) {
        // synchronous version:
        /* self.image_buffer
        .borrow_mut()
        .enumerate_pixels_mut()
        .for_each(|(x, y, pxl)| {
            pxl.0 = [255, 255, 255, u8::MAX];

            let x = x as f64 / self.win_rect.w() as f64;
            let y = y as f64 / self.win_rect.h() as f64;

            let noise = self.noise.get([x, y, self.z]);
            let mapped = ((noise + 1.0) / 2.0) * self.num_contours as f64;
            let px = mod1(mapped);

            // let min = self.range.start();
            // let max = self.range.end();
            // let mid = ((max - min) / 2.0) + min;
            //
            // let brightness = if (*min..=mid).contains(&px) {
            //     map_range(px, *min, mid, 1.0, 0.0)
            // }
            // else if (mid..=*max).contains(&px) {
            //     map_range(px, mid, *max, 0.0, 1.0)
            // }
            // else {
            //     1.0
            // };
            //
            // let pxl_br = (brightness * u8::MAX as f64) as u8;
            //
            // pxl.0 = [pxl_br, pxl_br, pxl_br, u8::MAX];

            if self.range.contains(&px) {
                pxl.0 = [0, 0, 0, u8::MAX];
            }
        }); */

        // important to update this first, as it ensures the generated image
        // matches what is sampled from the noise generator before this method
        // is called again
        self.update_z();

        // generate pixel information
        self.process_async();

        let pxl_per_thread = self.rows_per_thread() * self.width_px();

        // copy generated information to the image buffer
        self.image_buffer
            .borrow_mut()
            .pixels_mut()
            .enumerate()
            .for_each(|(i, pxl)| {
                if let Ok(guard) =
                    self.scratch_buffers[i / pxl_per_thread].lock()
                {
                    if pxl_per_thread != 0 {
                        *pxl = guard[i % pxl_per_thread];
                    }
                }
            });
    }

    /// Uploads the internal image buffer to a texture, and passes it to `Draw`.
    ///
    /// `device` and `frame` are used to upload the texture.
    pub fn draw(&self, device: &wgpu::Device, draw: &Draw, frame: &Frame) {
        self.texture.upload_data(
            device,
            &mut frame.command_encoder(),
            self.image_buffer.borrow().as_flat_samples().as_slice(),
        );

        draw.texture(&self.texture);
    }

    /// Sets the internal noise seed to a random value.
    pub fn reset_seed(&mut self) {
        self.noise = Arc::new(self.noise.set_seed(random()));
    }

    /// Returns a reference to a slice of pixels of column `idx`.
    ///
    /// This function copies the pixel data into an internal slice for convenience,
    /// (unless the same index was just requested via this method).
    ///
    /// Returns `None` if `idx` is outside the bounds of the image buffer.
    pub fn column(&mut self, idx: usize) -> Option<&[Rgba<u8>]> {
        if idx > self.image_buffer.borrow().len() {
            return None;
        }
        if let Some(prev) = self.prev_col_idx {
            if prev == idx {
                return Some(&self.column);
            }
        }

        for (i, mut x) in self.image_buffer.borrow().rows().enumerate() {
            if let Some(pxl) = x.nth(idx) {
                self.column[i] = *pxl;
            }
        }

        Some(&self.column)
    }

    pub fn column_direct(&mut self, mask: &mut SpectralMask, col_idx: usize) {
        let buf = self.image_buffer.borrow();
        if col_idx > buf.width() as usize {
            return;
        }

        let sr = unsafe { SAMPLE_RATE };
        let num_bins = mask.len();
        let width = buf.width() as f64;

        for i in 1..num_bins {
            let bin_freq = mask.bin_freq(i, sr);

            let x = col_idx as f64 / width;
            let y = 1.0 - freq_log_norm(bin_freq, 20.0, sr);

            let noise = self.noise.get([x, y, self.z]);
            let mapped = ((noise + 1.0) / 2.0) * self.num_contours as f64;

            if self.range.contains(&mod1(mapped)) {
                mask[i] = 1.0;
            }
            else {
                mask[i] = 0.0;
            }
        }
    }

    /// Adds the provided range to `self`.
    ///
    /// This is the range in which pixels are drawn for each contour.
    ///
    /// Has no effect if the range is outside `0 <= x <= 1`.
    ///
    /// Consuming method.
    pub fn with_contour_range(self, range: RangeInclusive<f64>) -> Self {
        if *range.start() < 0.0 || *range.end() > 1.0 {
            self
        }
        else {
            Self { range: Arc::new(range), ..self }
        }
    }

    /// Sets the range in which pixels are drawn for each contour.
    ///
    /// Has no effect if the range is outside `0 <= x <= 1`.
    pub fn set_contour_range(&mut self, range: RangeInclusive<f64>) {
        if 0.0 <= *range.start() && *range.end() <= 1.0 {
            self.range = Arc::new(range);
        }
    }

    /// Sets how many contours to compute.
    ///
    /// Consuming method.
    pub fn with_num_contours(self, num_contours: u32) -> Self {
        Self { num_contours, ..self }
    }

    /// Sets how many contours to compute.
    pub fn set_num_contours(&mut self, num_contours: u32) {
        self.num_contours = num_contours;
    }

    /// Sets how much to increment the z value per frame to transition through
    /// a third noise dimension.
    ///
    /// Consuming method.
    pub fn with_z_increment(self, z_increment: f64) -> Self {
        Self { z_increment, ..self }
    }

    /// Asynchronously processes the contour lines on multiple threads, where
    /// each thread is responsible for a portion of the rows.
    fn process_async(&mut self) {
        let rows_per_thread = self.rows_per_thread();
        let width = self.width_px();
        let height = self.height_px();
        let z = self.z;

        for i in 0..CONTOUR_NUM_THREADS {
            let num_contours = self.num_contours;
            let noise = Arc::clone(&self.noise);
            let range = Arc::clone(&self.range);
            let buf = Arc::clone(&self.scratch_buffers[i]);

            let start_row = i * rows_per_thread;

            // TODO there must be a better way to handle this than spawning threads
            // each time - thread pool?
            self.thread_pool.execute(move || {
                let mut buf = buf.lock().unwrap();

                for x in 0..width {
                    for y in 0..rows_per_thread {
                        let actual_y = start_row + y;
                        let x_norm = x as f64 / width as f64;
                        let y_norm = actual_y as f64 / height as f64;

                        let noise = noise.get([x_norm, y_norm, z]);
                        let mapped =
                            ((noise + 1.0) / 2.0) * num_contours as f64;
                        let px = mod1(mapped);

                        buf[y * width + x] = if range.contains(&px) {
                            Rgba([0, 0, 0, u8::MAX])
                        }
                        else {
                            Rgba([255, 255, 255, u8::MAX])
                        }
                    }
                }
            });
        }
    }

    /// The width of the image buffer in pixels.
    pub fn width_px(&self) -> usize {
        self.image_buffer.borrow().width() as usize
    }

    /// The height of the image buffer in pixels.
    pub fn height_px(&self) -> usize {
        self.image_buffer.borrow().width() as usize
    }

    /// The number of rows allocated to each thread.
    fn rows_per_thread(&self) -> usize {
        self.image_buffer.borrow().height() as usize / CONTOUR_NUM_THREADS
    }

    fn update_z(&mut self) {
        self.z += self.z_increment;
        // to maintain floating-point precision, just bounce the z value back
        // and forth within this range
        if self.z < -1_000_000.0 || 1_000_000.0 < self.z {
            self.z_increment *= -1.0;
        }
    }
}

/// Calculates the modulo-1 value of a floating-point value.
fn mod1(x: f64) -> f64 {
    x - x.floor()
}