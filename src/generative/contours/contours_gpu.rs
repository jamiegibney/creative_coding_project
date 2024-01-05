//! Pseudo-perlin noise contour line generator run on the GPU.

use crate::dsp::SpectralMask;
use crate::prelude::*;
use atomic_float::AtomicF32;
use nannou::image::{ImageBuffer, Rgba};
use nannou::prelude::*;
use nannou::wgpu;
use std::mem::size_of;
use std::ops::RangeInclusive;
use std::sync::{atomic::AtomicU32, Arc, Mutex};

const CONTOUR_SHADER_X_THREADS: u32 = 16;
const CONTOUR_SHADER_Y_THREADS: u32 = 16;

#[repr(C)]
#[derive(Clone, Copy)]
struct ContoursParams {
    /// Number of contour lines to draw.
    num_contours: u32,
    /// Upper bound of the contour range.
    upper: f32,
    /// The current z-value for the perlin noise.
    z: f32,
    /// The horizontal resolution.
    width: u32,
    /// The vertical resolution.
    height: u32,
}

/// Atomic params so we don't need a Mutex!
struct ContoursParamsAtomic {
    num_contours: Arc<AtomicU32>,
    upper: Arc<AtomicF32>,
    z: AtomicF32,
}

impl ContoursParams {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { wgpu::bytes::from(self) }
    }
}

/// Data used for the GPU compute shader.
struct Compute {
    image_buffer: wgpu::Buffer,
    image_buffer_size: wgpu::BufferAddress,

    data_buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup,

    pipeline: wgpu::ComputePipeline,
}

impl Compute {
    pub fn new(
        w: u32,
        h: u32,
        device: &wgpu::Device,
        cs_mod: &wgpu::ShaderModule,
    ) -> Self {
        let image_buf_size =
            (w as usize * h as usize * size_of::<f32>()) as wgpu::BufferAddress;
        let image_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("contours image data"),
            size: image_buf_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let ctr_data = ContoursParams {
            num_contours: 1,
            upper: 1.0,
            z: 0.0,
            width: w,
            height: h,
        };
        let ctr_data_bytes = ctr_data.as_bytes();
        let data_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("contours data buffer"),
            contents: ctr_data_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            // dynamic storage? / readonly storage?
            .storage_buffer(wgpu::ShaderStages::COMPUTE, false, false)
            // dynamic uniform?
            .uniform_buffer(wgpu::ShaderStages::COMPUTE, false)
            .build(device);
        let bind_group = wgpu::BindGroupBuilder::new()
            .buffer_bytes(
                &image_buffer,
                0,
                Some(std::num::NonZeroU64::new(image_buf_size).unwrap()),
            )
            // .texture_view(&texture_view)
            .buffer::<ContoursParams>(&data_buffer, 0..1)
            .build(device, &bind_group_layout);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("contours pipeline"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("contours pipeline"),
                layout: Some(&pipeline_layout),
                module: cs_mod,
                entry_point: "main",
            });

        Self {
            image_buffer,
            image_buffer_size: image_buf_size,

            data_buffer,

            bind_group,

            pipeline,
        }
    }
}

/// A pseudo-perlin noise contour line generator computed on the GPU.
pub struct ContoursGPU {
    rect: Rect,

    compute: Compute,
    z_increment: Arc<AtomicF32>,

    params: ContoursParams,
    atomic_params: ContoursParamsAtomic,

    texture: wgpu::Texture,

    image_buffer: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>,
}

impl ContoursGPU {
    pub fn new(app: &App, rect: Rect) -> Self {
        let w = rect.w().floor() as u32;
        let h = rect.h().floor() as u32;
        let win = app.main_window();
        let device = win.device();

        let cs_desc = wgpu::include_wgsl!("./contours.wgsl");
        let cs_mod = device.create_shader_module(&cs_desc);

        Self {
            rect,

            compute: Compute::new(w, h, device, &cs_mod),

            z_increment: Arc::new(AtomicF32::new(0.2)),

            params: ContoursParams {
                num_contours: 8,
                upper: 0.6,
                z: 0.0,
                width: w,
                height: h,
            },

            atomic_params: ContoursParamsAtomic {
                num_contours: Arc::new(AtomicU32::new(8)),
                upper: Arc::new(AtomicF32::new(0.6)),
                z: AtomicF32::new(0.0),
            },

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

            image_buffer: Arc::new(Mutex::new(ImageBuffer::from_fn(
                w,
                h,
                |_, _| Rgba([0, 0, 0, u8::MAX]),
            ))),
        }
    }

    pub fn with_z_increment(mut self, increment: f32) -> Self {
        self.z_increment.sr(increment);
        self.update_gpu_params(0.0);
        self
    }

    pub fn with_num_contours(mut self, num_contours: u32) -> Self {
        self.atomic_params.num_contours.sr(num_contours);
        self.update_gpu_params(0.0);
        self
    }

    pub fn with_contour_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.atomic_params.upper.sr(*range.end());
        self.update_gpu_params(0.0);
        self
    }

    pub fn set_num_contours(&self, num_contours: u32) {
        self.atomic_params.num_contours.sr(num_contours);
    }

    pub fn set_contour_range(&self, range: RangeInclusive<f32>) {
        let upper = *range.end();

        if upper <= 1.0 {
            self.atomic_params.upper.sr(upper);
        }
    }

    pub fn set_z_increment(&self, increment: f32) {
        self.z_increment.sr(increment);
    }

    pub fn randomize(&self) {
        self.atomic_params.z.sr(random_range(-1000.0, 1000.0));
    }

    pub fn num_contours_arc(&self) -> Arc<AtomicU32> {
        Arc::clone(&self.atomic_params.num_contours)
    }

    pub fn upper_arc(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.atomic_params.upper)
    }

    pub fn z_increment_arc(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.z_increment)
    }

    fn update_gpu_params(&mut self, delta_time: f32) {
        self.atomic_params.z.fetch_add(
            self.z_increment.lr() * delta_time,
            atomic::Ordering::Relaxed,
        );

        self.params = ContoursParams {
            num_contours: self.atomic_params.num_contours.lr(),
            upper: self.atomic_params.upper.lr(),
            z: self.atomic_params.z.lr(),
            ..self.params
        };
    }

    fn get_value_bilinear(&self, mut x: f64, mut y: f64) -> f64 {
        let width = self.rect.w() as f64;
        let height = self.rect.h() as f64;

        x = x.clamp(0.0, 1.0) * (width - 1.0);
        y = y.clamp(0.0, 1.0) * (height - 1.0);

        let xt = x.fract();
        let yt = y.fract();

        let x1 = x.floor() as u32;
        let x2 = (x1 + 1).min(255);
        let y1 = y.floor() as u32;
        let y2 = (y1 + 1).min(255);

        let guard = self
            .image_buffer
            .lock()
            .expect("failed to acquire lock on contours image buffer");

        let tl = guard.get_pixel(x1, y1).0[0] as f64 / 255.0;
        let tr = guard.get_pixel(x2, y1).0[0] as f64 / 255.0;
        let bl = guard.get_pixel(x1, y2).0[0] as f64 / 255.0;
        let br = guard.get_pixel(x2, y2).0[0] as f64 / 255.0;

        drop(guard);

        let top = lerp(tl, tr, xt);
        let bottom = lerp(bl, br, xt);

        lerp(top, bottom, yt)
    }
}

impl UIDraw for ContoursGPU {
    #[allow(clippy::cast_ptr_alignment)]
    fn update(&mut self, app: &App, input_data: &InputData) {
        self.update_gpu_params(input_data.delta_time as f32);

        let window = app.main_window();
        let device = window.device();
        let rect = self.rect;

        // TODO: extract this buffer to self?
        // allocate a GPU buffer for the image data
        let read_image_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read image"),
            size: self.compute.image_buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // TODO: extract this buffer to self?
        // load the local data into a gpu buffer
        let data_len = size_of::<ContoursParams>() as wgpu::BufferAddress;
        let data = self.params.as_bytes();
        let data_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("contours data transfer"),
            contents: data,
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        // create the encoder
        let desc =
            wgpu::CommandEncoderDescriptor { label: Some("contours compute") };
        let mut encoder = device.create_command_encoder(&desc);
        // copy data into the compute buffer
        encoder.copy_buffer_to_buffer(
            &data_buf, 0, &self.compute.data_buffer, 0, data_len,
        );

        // dispatch the compute shader
        let mut compute_pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("contours compute pass"),
            });
        compute_pass.set_pipeline(&self.compute.pipeline);
        compute_pass.set_bind_group(0, &self.compute.bind_group, &[]);
        compute_pass.dispatch(
            rect.w() as u32 / CONTOUR_SHADER_X_THREADS,
            rect.h() as u32 / CONTOUR_SHADER_Y_THREADS,
            1,
        );

        // needed to unborrow the encoder so it can be consumed
        drop(compute_pass);

        // copy the image buffer over
        encoder.copy_buffer_to_buffer(
            &self.compute.image_buffer, 0, &read_image_buffer, 0,
            self.compute.image_buffer_size,
        );

        window.queue().submit(Some(encoder.finish()));

        // copy from the GPU buffer to local mem
        let image_buffer = Arc::clone(&self.image_buffer);
        let future = async move {
            let image_slice = read_image_buffer.slice(..);

            if image_slice.map_async(wgpu::MapMode::Read).await.is_ok() {
                if let Ok(mut guard) = image_buffer.lock() {
                    let bytes = &image_slice.get_mapped_range();

                    let floats = unsafe {
                        std::slice::from_raw_parts(
                            bytes.as_ptr().cast::<f32>(),
                            bytes.len() / size_of::<f32>(),
                        )
                    };

                    for (px, &br) in guard.pixels_mut().zip(floats.iter()) {
                        let br = (br * 255.0) as u8;
                        *px = Rgba([br, br, br, u8::MAX]);
                    }
                }
            }
        };

        async_std::task::spawn(future);

        // not required here, as nannou calls this for the main window
        // after the view() callback.
        // device.poll(wgpu::Maintain::Poll);
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        if let Ok(guard) = self.image_buffer.lock() {
            self.texture.upload_data(
                app.main_window().device(),
                &mut frame.command_encoder(),
                guard.as_flat_samples().as_slice(),
            );
        }

        draw.texture(&self.texture)
            .xy(self.rect.xy())
            .wh(self.rect.wh());
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}

impl DrawMask for ContoursGPU {
    fn column_to_mask(
        &self,
        mask: &mut crate::dsp::SpectralMask,
        mask_len: usize,
        x: f64,
    ) {
        if !(0.0..=1.0).contains(&x) {
            return;
        }

        let sr = unsafe { SAMPLE_RATE };

        for i in 1..mask_len {
            let bin_hz = SpectralMask::bin_freq(i, mask_len, sr);
            if bin_hz < 20.0 {
                mask[i] = 0.0;
                continue;
            }
            let y = 1.0 - freq_log_norm(bin_hz, 20.0, sr);

            let br = self.get_value_bilinear(x, y);

            mask[i] = br;
        }
    }
}
