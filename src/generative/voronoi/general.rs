use super::compute::*;
use super::*;
use std::mem::size_of;
use std::sync::{Arc, Mutex};

pub struct VoronoiGPU {
    rect: Rect,
    points: Points,
    state: VoronoiStateGeneral,
    compute: ComputeGeneral,

    image_buf: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>>,
    texture: wgpu::Texture,
}

impl VoronoiGPU {
    pub fn new(app: &App, rect: Rect) -> Self {
        let w = rect.w() as u32;
        let h = rect.h() as u32;
        let win = app.main_window();
        let device = win.device();

        let cs_desc = wgpu::include_wgsl!("./voronoi.wgsl");
        let cs_mod = device.create_shader_module(&cs_desc);

        Self {
            rect,
            points: Points::new(),
            state: VoronoiStateGeneral {
                active_cells: MAX_NUM_POINTS as u32,
                width: w,
                height: h,
            },
            compute: ComputeGeneral::new(w, h, device, &cs_mod),

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
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST,
                )
                .build(device),
        }
    }

    /// Copy the 2D points from `Vectors` into the `Voronoi` generator.
    pub fn copy_from_vectors(&mut self, vectors: &Vectors) {
        self.state.active_cells = vectors.num_active_points as u32;

        self.points.copy_from_vectors(vectors);
    }
}

impl UIDraw for VoronoiGPU {
    fn update(&mut self, app: &App, input_data: &InputData) {
        let window = app.main_window();
        let device = window.device();
        let rect = self.rect();

        let read_image_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("read image"),
            size: self.compute.image_buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let points_len = size_of::<Points>() as wgpu::BufferAddress;
        let points_bytes = self.points.as_bytes();
        let points_buffer =
            device.create_buffer_init(&wgpu::BufferInitDescriptor {
                label: Some("voronoi points transfer"),
                contents: points_bytes,
                usage: wgpu::BufferUsages::COPY_SRC,
            });

        let state_len = size_of::<VoronoiStateGeneral>() as wgpu::BufferAddress;
        let state_bytes = self.state.as_bytes();
        let state_buffer =
            device.create_buffer_init(&wgpu::BufferInitDescriptor {
                label: Some("voronoi state transfer"),
                contents: state_bytes,
                usage: wgpu::BufferUsages::COPY_SRC,
            });

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("voronoi command encoder"),
            });

        encoder.copy_buffer_to_buffer(
            &points_buffer, 0, &self.compute.points_buffer, 0, points_len,
        );
        encoder.copy_buffer_to_buffer(
            &state_buffer, 0, &self.compute.state_buffer, 0, state_len,
        );

        let mut compute_pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("voronoi compute pass"),
            });
        compute_pass.set_pipeline(&self.compute.pipeline);
        compute_pass.set_bind_group(0, &self.compute.bind_group, &[]);
        compute_pass.dispatch(
            self.rect.w() as u32 / VORONOI_SHADER_X_THREADS,
            self.rect.h() as u32 / VORONOI_SHADER_Y_THREADS,
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
                    let bytes = &image_slice.get_mapped_range();

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

// principle:
// - first, iterate through each point and find the closest point to it, and store the indices somewhere
// - for each pixel, iterate through all points and find the closest
//  - store the distance (dot product)
//  - store the relative vector (i.e. current pixel - point, making the pixel the "origin")
// - then, get the closest point to that point
//  - if the dot product of (first relative - second relative) is over a threshold output the minimum of:
//      - the first min distance, or
//      - the dot product of half the relative vector (min_r - r) and the normalised vector (norm(r - min_r))
//  - otherwise, output the first distance value
