use crate::dsp::SpectralMask;
use crate::prelude::*;

use nannou::image::{ImageBuffer, Rgba};
use nannou::noise::{NoiseFn, Perlin, Seedable};
use nannou::prelude::*;
use nannou::wgpu::*;

use std::cell::RefCell;
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};

pub mod contours_gpu;

pub use contours_gpu::ContoursGPU;

/// Perlin noise contour line generator. Supports multi-threading.
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

    /// The bounding rect of the visual.
    rect: Rect,

    /// Whether each contour should be feathered or solid.
    feathering: bool,

    /// The internal image buffer (the buffer which holds pixel data).
    image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,

    thread_buffers: Vec<Arc<Mutex<Vec<Rgba<u8>>>>>,
    thread_pool: Option<ThreadPool>,
}

impl Contours {
    /// Creates a new `Contours`.
    ///
    /// This object is responsible for handling its own texture and image buffer.
    ///
    /// Uses 1 thread by default — see [`with_num_threads()`](Self::with_num_threads).
    pub fn new(device: &wgpu::Device, rect: Rect) -> Self {
        let width = rect.w().floor() as u32;
        let height = rect.h().floor() as u32;

        Self {
            noise: Arc::new(Perlin::new().set_seed(random())),
            z: 0.0,
            z_increment: 0.003,

            range: Arc::new(0.0..=0.0),
            num_contours: 1,

            texture: wgpu::TextureBuilder::new()
                .size([width, height])
                .mip_level_count(4)
                .sample_count(1)
                .format(wgpu::TextureFormat::Rgba8Unorm)
                .usage(
                    wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                )
                .build(device),
            image_buffer: ImageBuffer::from_fn(width, height, |_, _| {
                Rgba([255, 255, 255, u8::MAX])
            }),

            feathering: false,

            rect,

            thread_pool: None,
            thread_buffers: Vec::new(),
        }
    }

    /// Randomizes the internal `z` value.
    pub fn randomize(&mut self) {
        // self.noise = Arc::new(self.noise.set_seed(random()));
        self.z = random_range(-1000.0, 1000.0);
    }

    /// Adds the provided range to `self`.
    ///
    /// This is the range in which pixels are drawn for each contour.
    ///
    /// Has no effect if the range is outside `0 <= x <= 1`.
    ///
    /// Consuming method.
    pub fn with_contour_range(mut self, range: RangeInclusive<f64>) -> Self {
        self.set_contour_range(range);
        self
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
    pub fn with_num_contours(mut self, num_contours: u32) -> Self {
        self.set_num_contours(num_contours);
        self
    }

    /// Sets how many contours to compute.
    pub fn set_num_contours(&mut self, num_contours: u32) {
        self.num_contours = num_contours;
    }

    /// Sets how much to increment the z value per frame to transition through
    /// a third noise dimension.
    ///
    /// Consuming method.
    pub fn with_z_increment(mut self, z_increment: f64) -> Self {
        self.set_z_increment(z_increment);
        self
    }

    /// Sets how much to increment the z value per frame to transition through
    /// a third noise dimension.
    pub fn set_z_increment(&mut self, z_increment: f64) {
        self.z_increment = z_increment;
    }

    /// Sets the number of threads to use for computing the noise contours.
    ///
    /// This method will allocate if the threads are spawned.
    ///
    /// Consuming method. Returns `Some` if the threads were successfully spawned,
    /// and `None` otherwise.
    ///
    /// # Panics
    ///
    /// Panics if `num_threads` is not a power-of-two value (required to efficiently
    /// divide the image buffer up).
    pub fn with_num_threads(mut self, num_threads: usize) -> Option<Self> {
        // TODO: this should return Result<Self>.
        if self.set_num_threads(num_threads) {
            Some(self)
        }
        else {
            None
        }
    }

    /// Sets the number of threads to use for computing the noise contours.
    ///
    /// Returns `true` if the threads were successfully spawned, and false otherwise.
    ///
    /// This method will allocate if new threads are successfully spawned.
    ///
    /// # Panics
    ///
    /// Panics if `num_threads` is not a power-of-two value (required to efficiently
    /// divide the image buffer up).
    pub fn set_num_threads(&mut self, num_threads: usize) -> bool {
        assert!(num_threads.is_power_of_two());

        let total_px = self.width_px() * self.height_px();
        let px_per_thread = total_px / num_threads;

        if let Ok(pool) = ThreadPool::build(num_threads) {
            self.thread_pool = Some(pool);
            self.thread_buffers = {
                let mut v = Vec::with_capacity(num_threads);
                (0..num_threads).for_each(|_| {
                    v.push(Arc::new(Mutex::new(vec![
                        Rgba([
                            255,
                            255,
                            255,
                            u8::MAX,
                        ]);
                        px_per_thread
                    ])));
                });
                v
            };

            true
        }
        else {
            false
        }
    }

    /// Controls whether the contour lines should be feathered or not.
    ///
    /// Consuming method.
    pub fn with_feathering(mut self, should_feather: bool) -> Self {
        self.feathering = should_feather;
        self
    }

    /// Controls whether the contour lines should be feathered or not.
    pub fn set_feathering(&mut self, should_feather: bool) {
        self.feathering = should_feather;
    }

    /// The width of the image buffer in pixels.
    pub fn width_px(&self) -> usize {
        self.image_buffer.width() as usize
    }

    /// The height of the image buffer in pixels.
    pub fn height_px(&self) -> usize {
        self.image_buffer.width() as usize
    }

    /// Synchronously processes the contour lines on one thread.
    fn compute(&mut self) {
        let width = self.width_px() as f64;
        let height = self.height_px() as f64;
        self.image_buffer
            .enumerate_pixels_mut()
            .for_each(|(x, y, pxl)| {
                pxl.0 = [255, 255, 255, u8::MAX];

                let x = x as f64 / width;
                let y = y as f64 / height;

                let noise = self.noise.get([x, y, self.z]);
                let mapped = ((noise + 1.0) / 2.0) * self.num_contours as f64;
                let px = mapped.fract();

                if self.range.contains(&px) {
                    pxl.0 = [0, 0, 0, u8::MAX];
                }
            });
    }

    /// Asynchronously processes the contour lines on multiple threads, where
    /// each thread is responsible for a portion of the rows.
    fn compute_async(&mut self) {
        let rows_per_thread = self.rows_per_thread();
        let width = self.width_px();
        let height = self.height_px();
        let z = self.z;

        if let Some(pool) = &self.thread_pool {
            let num_threads = pool.num_threads();

            for i in 0..num_threads {
                let num_contours = self.num_contours;
                let range = Arc::clone(&self.range);
                let noise = Arc::clone(&self.noise);
                let buf = Arc::clone(&self.thread_buffers[i]);
                let feathered = self.feathering;

                let start_row = i * rows_per_thread;

                pool.execute(move || {
                    let mut buf = buf.lock().unwrap();

                    for x in 0..width {
                        for y in 0..rows_per_thread {
                            let actual_y = start_row + y;
                            let x_norm = x as f64 / width as f64;
                            let y_norm = actual_y as f64 / height as f64;

                            let noise = noise.get([x_norm, y_norm, z]);
                            let br = (Self::contour_brightness(
                                num_contours, &range, noise, feathered,
                            ) * 255.0)
                                as u8;
                            buf[y * width + x] = Rgba([br, br, br, u8::MAX]);
                        }
                    }
                });
            }

            pool.wait_until_done();
        }

        let pxl_per_thread = rows_per_thread * width;

        // copy generated information to the image buffer
        self.image_buffer
            .pixels_mut()
            .enumerate()
            .for_each(|(i, pxl)| {
                if let Ok(guard) =
                    self.thread_buffers[i / pxl_per_thread].lock()
                {
                    if pxl_per_thread != 0 {
                        *pxl = guard[i % pxl_per_thread];
                    }
                }
            });
    }

    /// The contouring method for the generator.
    fn contour(&self, value: f64) -> f64 {
        if self.range.contains(&value) {
            0.0
        }
        else {
            1.0
        }
    }

    /// The number of rows allocated to each thread.
    fn rows_per_thread(&self) -> usize {
        // SAFETY: this function is only called if the thread pool exists, so
        // unwrapping it is fine.
        let num_threads = self.thread_pool.as_ref().unwrap().num_threads();
        self.image_buffer.height() as usize / num_threads
    }

    /// Updates the internal z value used for the noise field's third dimension.
    fn update_z(&mut self, delta_time: f64) {
        self.z += self.z_increment * delta_time;
        // to maintain floating-point precision, just bounce the z value back
        // and forth within this range
        if self.z < -1_000_000.0 || 1_000_000.0 < self.z {
            self.z_increment *= -1.0;
        }
    }

    #[allow(clippy::similar_names)]
    fn contour_brightness(
        num_contours: u32,
        range: &RangeInclusive<f64>,
        noise_value: f64,
        feathered: bool,
    ) -> f64 {
        let mapped = ((noise_value + 1.0) / 2.0) * num_contours as f64;
        let px = mapped.fract();

        let min = *range.start();
        let max = *range.end();
        let mid = lerp(min, max, 0.5);

        if feathered {
            if (min..=mid).contains(&px) {
                xfer::s_curve(normalise(px, min, mid), 0.1)
            }
            else if (mid..=max).contains(&px) {
                (xfer::s_curve(1.0 - normalise(px, mid, max), 0.1))
            }
            else {
                0.0
            }
        }
        else if (min..=mid).contains(&px) {
            xfer::s_curve_round(normalise(px, min, mid), 0.97)
        }
        else if (mid..=max).contains(&px) {
            (1.0 - xfer::s_curve_round(normalise(px, mid, max), -0.97))
        }
        else {
            0.0
        }
    }
}

impl UIDraw for Contours {
    fn update(&mut self, _: &App, input_data: &InputData) {
        // important to update this first, as it ensures the generated image
        // matches what is sampled from the noise generator before this method
        // is called again
        self.update_z(input_data.delta_time);

        if self.thread_pool.is_some() {
            self.compute_async();
        }
        else {
            self.compute();
        }
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

impl DrawMask for Contours {
    /// Updates the internal image buffer and noise generator.
    // fn update(&mut self, delta_time: f64) {
    //     // important to update this first, as it ensures the generated image
    //     // matches what is sampled from the noise generator before this method
    //     // is called again
    //     self.update_z(delta_time);
    //
    //     if self.thread_pool.is_some() {
    //         self.compute_async();
    //     }
    //     else {
    //         self.compute();
    //     }
    // }

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

    /// Directly mutates a `SpectralMask`, placing the contour information at
    /// `x` in it. `x` is expected to be between `0.0` and `1.0`, where `0.0` is
    /// the far-left and vice versa.
    ///
    /// If `x < 0.0 || 1.0 < x`, this method has no effect.
    fn column_to_mask(&self, mask: &mut SpectralMask, mask_len: usize, x: f64) {
        if !(0.0..=1.0).contains(&x) {
            return;
        }

        let sr = unsafe { SAMPLE_RATE };

        // start at 1 to skip the 0 Hz component
        for i in 1..mask_len {
            // get the frequency of the current bin
            let bin_freq = SpectralMask::bin_freq(i, mask_len, sr);

            // get a logarithmically scaled, normalised frequency value
            let y = 1.0 - freq_log_norm(bin_freq, 20.0, sr);

            // get the noise value at the bin's position
            let noise = self.noise.get([x, y, self.z]);
            // let mapped = ((noise + 1.0) / 2.0) * self.num_contours as f64;

            // apply the contouring method
            mask[i] = Self::contour_brightness(
                self.num_contours, &self.range, noise, self.feathering,
            );
        }
    }
}
