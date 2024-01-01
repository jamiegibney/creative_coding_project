use super::*;
use std::mem::size_of;

pub struct ComputeGeneral {
    /// The 2D grid buffer.
    pub image_buffer: wgpu::Buffer,
    /// The size of the image buffer.
    pub image_buffer_size: wgpu::BufferAddress,

    /// Buffer for each 2D point.
    pub points_buffer: wgpu::Buffer,
    /// Buffer for voronoi state.
    pub state_buffer: wgpu::Buffer,

    /// Shader bind group.
    pub bind_group: wgpu::BindGroup,

    /// Shader compute pipeline.
    pub pipeline: wgpu::ComputePipeline,
}

impl ComputeGeneral {
    pub fn new(
        w: u32,
        h: u32,
        device: &wgpu::Device,
        compute_shader: &wgpu::ShaderModule,
    ) -> Self {
        let image_buffer_size =
            (w as usize * h as usize * size_of::<f32>()) as wgpu::BufferAddress;
        let image_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("voronoi image data"),
            size: image_buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let points = Points::new();
        let points_bytes = points.as_bytes();
        let points_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("voronoi points buffer"),
            contents: points_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let state = VoronoiStateGeneral {
            active_cells: MAX_NUM_POINTS as u32,
            weight: 0.3,
            width: w,
            height: h,
        };
        let state_bytes = state.as_bytes();
        let state_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("voronoi state buffer"),
            contents: state_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
            // image
            .storage_buffer(wgpu::ShaderStages::COMPUTE, false, false)
            // points
            .uniform_buffer(wgpu::ShaderStages::COMPUTE, false)
            // state
            .uniform_buffer(wgpu::ShaderStages::COMPUTE, false)
            .build(device);

        let bind_group = wgpu::BindGroupBuilder::new()
            .buffer_bytes(
                &image_buffer,
                0,
                Some(std::num::NonZeroU64::new(image_buffer_size).unwrap()),
            )
            .buffer::<Points>(&points_buffer, 0..1)
            .buffer::<VoronoiStateGeneral>(&state_buffer, 0..1)
            .build(device, &bind_group_layout);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("voronoi pipeline"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("voronoi pipeline"),
                layout: Some(&pipeline_layout),
                module: compute_shader,
                entry_point: "main",
            });

        Self {
            image_buffer,
            image_buffer_size,

            points_buffer,
            state_buffer,

            bind_group,
            pipeline,
        }
    }
}
