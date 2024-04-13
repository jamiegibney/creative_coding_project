//! SmoothLife generator run on the GPU.

use crate::app::SmoothLifePreset;
use crate::dsp::SpectralMask;
use crate::prelude::*;
use atomic::Atomic;
use atomic_float::AtomicF32;
use nannou::image::{ImageBuffer, Rgba};
use nannou::prelude::*;
use nannou::wgpu;
use nannou_audio::stream::input;
use std::mem::size_of;
use std::sync::{
    atomic::{AtomicBool, AtomicU32},
    Arc, Mutex,
};
use std::time::Duration;

mod state;
use state::*;

const SMOOTHLIFE_SHADER_X_THREADS: u32 = 16;
const SMOOTHLIFE_SHADER_Y_THREADS: u32 = 16;

struct Compute {
    image_buffer: wgpu::Buffer,
    image_buffer_size: wgpu::BufferAddress,

    diff_buffer: wgpu::Buffer,

    state_buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup,

    pipeline: wgpu::ComputePipeline,
}

impl Compute {
    pub fn new(
        w: u32,
        h: u32,
        device: &wgpu::Device,
        compute_shader: &wgpu::ShaderModule,
    ) -> Self {
        // image buffer
        let image_buf_size =
            (w as usize * h as usize * size_of::<f32>()) as wgpu::BufferAddress;
        let image_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("smoothlife image data"),
            size: image_buf_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let diff_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("smoothlife diff buffer"),
            size: image_buf_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        // state buffer
        let sml_state = SmoothLifeState::slime(w, h);
        let sml_state_bytes = sml_state.as_bytes();
        let state_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("smoothlife data buffer"),
            contents: sml_state_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // bind group
        let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            // dynamic storage? / readonly storage?
            .storage_buffer(wgpu::ShaderStages::COMPUTE, false, false)
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
            .buffer_bytes(
                &diff_buffer,
                0,
                Some(std::num::NonZeroU64::new(image_buf_size).unwrap()),
            )
            .buffer::<SmoothLifeState>(&state_buffer, 0..1)
            .build(device, &bind_group_layout);

        // pipeline
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("smoothlife pipeline"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("smoothlife pipeline"),
                layout: Some(&pipeline_layout),
                module: compute_shader,
                entry_point: "main",
            });

        Self {
            image_buffer,
            image_buffer_size: image_buf_size,
            diff_buffer,
            state_buffer,
            bind_group,
            pipeline,
        }
    }
}

pub struct SmoothLifeGPU {
    state_gpu: SmoothLifeState,
    state_atomic: SmoothLifeStateAtomic,

    compute: Compute,

    rect: Rect,

    image_buf: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>,
    texture: wgpu::Texture,
}

impl SmoothLifeGPU {
    pub fn new(app: &App, rect: Rect) -> Self {
        let w = rect.w() as u32;
        let h = rect.h() as u32;
        let win = app.main_window();
        let device = win.device();

        let cs_desc = wgpu::include_wgsl!("./smooth_life.wgsl");
        let cs_mod = unsafe { device.create_shader_module_unchecked(&cs_desc) };

        Self {
            state_gpu: SmoothLifeState::slime(w, h),
            state_atomic: SmoothLifeStateAtomic::default(),
            compute: Compute::new(w, h, device, &cs_mod),
            rect,
            image_buf: Arc::new(Mutex::new(ImageBuffer::from_fn(
                w,
                h,
                |_, _| Rgba([0, 0, 0, u8::MAX]),
            ))),
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
        }
    }

    pub fn set_speed(&self, speed: f32) {
        self.state_atomic.speed.sr(speed);
    }

    pub fn set_outer_radius(&self, radius: f32) {
        debug_assert!((0.0..=32.0).contains(&radius));

        self.state_atomic.ra.sr(radius);
    }

    pub fn randomize(&self) {
        self.state_atomic.should_reset.sr(true);
    }

    pub fn set_preset(&self, preset: SmoothLifePreset) {
        self.state_atomic.preset.sr(preset);
    }

    pub fn preset_arc(&self) -> Arc<Atomic<SmoothLifePreset>> {
        Arc::clone(&self.state_atomic.preset)
    }

    pub fn speed_arc(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.state_atomic.speed)
    }

    fn update_gpu_state(&mut self, delta_time: f32) {
        let w = self.rect.w() as u32;
        let h = self.rect.h() as u32;

        if self.state_atomic.should_update_preset.lr() {
            self.state_gpu = match self.state_atomic.preset.lr() {
                SmoothLifePreset::Jitter => {
                    self.state_atomic.ra.sr(15.0);
                    self.state_atomic.speed.sr(9.0);
                    SmoothLifeState::flow(w, h)
                }
                SmoothLifePreset::Slime => {
                    self.state_atomic.ra.sr(26.0);
                    self.state_atomic.speed.sr(5.0);
                    SmoothLifeState::slime(w, h)
                }
                SmoothLifePreset::Corrupt => {
                    self.state_atomic.ra.sr(40.0);
                    self.state_atomic.speed.sr(12.0);
                    SmoothLifeState::corrupt(w, h)
                }
            }
        }

        self.state_gpu.should_randomize = 0;

        if self.state_atomic.should_reset.lr() {
            self.state_gpu.should_randomize = 1;
            self.state_atomic.should_reset.sr(false);
        }

        let ra = self.state_atomic.ra.lr();
        self.state_gpu = SmoothLifeState {
            ra,
            ri: ra / 3.0,
            dt: self.state_atomic.speed.lr(),
            delta_time,
            ..self.state_gpu
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
            .image_buf
            .lock()
            .expect("failed to acquire lock on smoothlife image buffer");

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

impl UIDraw for SmoothLifeGPU {
    fn update(&mut self, app: &App, input_data: &InputData) {
        self.update_gpu_state(input_data.delta_time as f32);

        let window = app.main_window();
        let device = window.device();
        let rect = self.rect;

        let read_image_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read image"),
            size: self.compute.image_buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let state_len = size_of::<SmoothLifeState>() as wgpu::BufferAddress;
        let state_bytes = self.state_gpu.as_bytes();
        let state_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("smoothlife data transfer"),
            contents: state_bytes,
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        // encoder
        let desc = wgpu::CommandEncoderDescriptor {
            label: Some("smoothlife compute"),
        };
        let mut encoder = device.create_command_encoder(&desc);
        encoder.copy_buffer_to_buffer(
            &state_buf, 0, &self.compute.state_buffer, 0, state_len,
        );

        let mut compute_pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("smoothlife compute pass"),
            });
        compute_pass.set_pipeline(&self.compute.pipeline);
        compute_pass.set_bind_group(0, &self.compute.bind_group, &[]);
        compute_pass.dispatch(
            self.rect.w() as u32 / SMOOTHLIFE_SHADER_X_THREADS,
            self.rect.h() as u32 / SMOOTHLIFE_SHADER_Y_THREADS,
            1,
        );

        drop(compute_pass);

        encoder.copy_buffer_to_buffer(
            &self.compute.image_buffer, 0, &read_image_buffer, 0,
            self.compute.image_buffer_size,
        );

        window.queue().submit(Some(encoder.finish()));

        let image_buffer = Arc::clone(&self.image_buf);

        let future = async move {
            let image_slice = read_image_buffer.slice(..);

            if image_slice.map_async(wgpu::MapMode::Read).await.is_ok() {
                if let Ok(mut guard) = image_buffer.lock() {
                    let bytes = &image_slice.get_mapped_range()[..];

                    let floats = unsafe {
                        std::slice::from_raw_parts(
                            bytes.as_ptr().cast::<f32>(),
                            bytes.len() / size_of::<f32>(),
                        )
                    };

                    for (pxl, &br) in guard.pixels_mut().zip(floats.iter()) {
                        let br = (br * 255.0) as u8;
                        *pxl = Rgba([br, br, br, u8::MAX]);
                    }
                }
            }
        };

        async_std::task::spawn(future);

        // device.poll(wgpu::Maintain::Poll);
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        if let Ok(guard) = self.image_buf.lock() {
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

impl DrawMask for SmoothLifeGPU {
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
            let y = 1.0 - freq_log_norm(bin_hz, 20.0, sr);

            let br = self.get_value_bilinear(x, y);

            mask[i] = br;
        }
    }
}
