/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use bytemuck::{Pod, Zeroable};
use image::DynamicImage;
use image::GenericImageView;
use limnus_wgpu_math::{Matrix4, Vec4};
use tracing::{debug, warn};
use wgpu::BufferBindingType;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferDescriptor, BufferUsages,
    Device, Extent3d, PipelineLayout, PipelineLayoutDescriptor, Queue, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerBindingType, ShaderModule, ShaderStages, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexStepMode,
};
use wgpu::{BindingResource, PipelineCompilationOptions};
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, Face, FrontFace, MultisampleState, PolygonMode,
    PrimitiveState, PrimitiveTopology, util,
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],   // 2D position of the vertex
    tex_coords: [f32; 2], // Texture coordinates
}

// Implement Zeroable manually
unsafe impl Zeroable for Vertex {}

// Implement Pod manually
unsafe impl Pod for Vertex {}

impl Vertex {
    const ATTRIBUTES: [VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    pub const fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// Buffer that stores a model and texture coordinates for one sprite
// model: Mx4
// tex_coords: V4
#[must_use]
pub fn create_sprite_uniform_buffer(device: &Device, label: &str) -> Buffer {
    device.create_buffer_init(&util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&[SpriteInstanceUniform {
            model: Matrix4::identity(),
            tex_coords_mul_add: Vec4([0.0, 0.0, 1.0, 1.0]),
            rotation: 0,
            color: Vec4([1.0, 0.0, 1.0, 1.0]),
            use_texture: 0,
        }]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CameraUniform {
    pub view_proj: Matrix4,
}

unsafe impl Pod for CameraUniform {}
unsafe impl Zeroable for CameraUniform {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SpriteInstanceUniform {
    pub model: Matrix4, // Model Transformation matrix
    pub tex_coords_mul_add: Vec4,
    pub rotation: u32,
    pub color: Vec4,
    pub use_texture: u32,
}

unsafe impl Pod for SpriteInstanceUniform {}
unsafe impl Zeroable for SpriteInstanceUniform {}

impl SpriteInstanceUniform {
    #[must_use]
    pub const fn new(
        model: Matrix4,
        tex_coords_mul_add: Vec4,
        rotation: u32,
        color: Vec4,
        use_texture: bool,
    ) -> Self {
        Self {
            model,
            tex_coords_mul_add,
            rotation,
            color,
            use_texture: use_texture as u32,
        }
    }
}

impl SpriteInstanceUniform {
    const fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                // Model Matrix. There is unfortunately no Matrix type, so you have to define it as 4 attributes of Float32x4.
                VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: 16,
                    shader_location: 3,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: 32,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: 48,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                // Texture multiplier and add
                VertexAttribute {
                    offset: 64,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                // Rotation
                VertexAttribute {
                    offset: 80,
                    shader_location: 7,
                    format: VertexFormat::Uint32,
                },
                // color (RGBA)
                VertexAttribute {
                    offset: 84,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
                // use_texture (bool)
                VertexAttribute {
                    offset: 84 + 4 * 4,
                    shader_location: 9,
                    format: VertexFormat::Uint32,
                },
            ],
        }
    }
}

// wgpu has, for very unknown reasons, put coordinate texture origo at top-left(!)
const RIGHT: f32 = 1.0;
const DOWN: f32 = 1.0;

const IDENTITY_QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0],
        tex_coords: [0.0, DOWN],
    }, // Bottom left
    Vertex {
        position: [1.0, 0.0],
        tex_coords: [RIGHT, DOWN],
    }, // Bottom right
    Vertex {
        position: [1.0, 1.0],
        tex_coords: [RIGHT, 0.0],
    }, // Top right
    Vertex {
        position: [0.0, 1.0],
        tex_coords: [0.0, 0.0],
    }, // Top left
];

// u16 is the smallest index buffer supported by wgpu // IndexFormat
pub const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[derive(Debug)]
pub struct SpriteInfo {
    pub sprite_pipeline: RenderPipeline,

    pub sampler: Sampler,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,

    // Camera - Group 0
    pub camera_bind_group_layout: BindGroupLayout,

    pub camera_uniform_buffer: Buffer,
    pub camera_bind_group: BindGroup,

    // Texture and Sampler - Group 1
    pub sprite_texture_sampler_bind_group_layout: BindGroupLayout,

    // Vertex Instances - Group 1
    pub quad_matrix_and_uv_instance_buffer: Buffer,
}

const MAX_RENDER_SPRITE_COUNT: usize = 10_000;

impl SpriteInfo {
    #[must_use]
    pub fn new(
        device: &Device,
        surface_texture_format: TextureFormat,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
        view_proj_matrix: Matrix4,
    ) -> Self {
        let vertex_shader =
            mireforge_wgpu::create_shader_module(device, "sprite vertex", vertex_shader_source);
        let fragment_shader =
            mireforge_wgpu::create_shader_module(device, "sprite fragment", fragment_shader_source);

        let index_buffer = create_sprite_index_buffer(device, "identity quad index buffer");
        let vertex_buffer = create_sprite_vertex_buffer(device, "identity quad vertex buffer");

        // ------------------------------- Camera View Projection Matrix in Group 0 --------------------------
        let camera_uniform_buffer = create_camera_uniform_buffer(
            device,
            view_proj_matrix,
            "view and projection matrix (camera)",
        );

        let camera_bind_group_layout =
            create_camera_uniform_bind_group_layout(device, "camera bind group layout");

        let camera_bind_group = create_camera_uniform_bind_group(
            device,
            &camera_bind_group_layout,
            &camera_uniform_buffer,
            "camera matrix",
        );

        // -------------------------- Texture and Sampler in Group 1 -----------------------------------------------
        let sprite_texture_sampler_bind_group_layout = create_sprite_texture_sampler_group_layout(
            device,
            "texture and sampler bind group layout",
        );

        // -------------------------- Sprite Instance in Group 2 -----------------------------------------------
        let quad_matrix_and_uv_instance_buffer = create_quad_matrix_and_uv_instance_buffer(
            device,
            MAX_RENDER_SPRITE_COUNT,
            "sprite_instance buffer",
        );

        let sprite_pipeline_layout = create_sprite_pipeline_layout(
            device,
            &camera_bind_group_layout,
            &sprite_texture_sampler_bind_group_layout,
            "sprite pipeline layout",
        );

        let sprite_pipeline = create_sprite_pipeline(
            device,
            surface_texture_format,
            &sprite_pipeline_layout,
            &vertex_shader,
            &fragment_shader,
        );

        let sampler = mireforge_wgpu::create_nearest_sampler(device, "sprite nearest sampler");

        Self {
            sprite_pipeline,
            sampler,
            vertex_buffer,
            index_buffer,
            camera_bind_group_layout,
            camera_uniform_buffer,
            camera_bind_group,
            sprite_texture_sampler_bind_group_layout,
            quad_matrix_and_uv_instance_buffer,
        }
    }
}

/// Creates the view - projection matrix (Camera)
fn create_camera_uniform_buffer(device: &Device, view_proj: Matrix4, label: &str) -> Buffer {
    let camera_uniform = CameraUniform { view_proj };

    device.create_buffer_init(&util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

/// Camera is just one binding, the view projection camera matrix
fn create_camera_uniform_bind_group_layout(device: &Device, label: &str) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn create_camera_uniform_bind_group(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    uniform_buffer: &Buffer,
    label: &str,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout: bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    })
}

#[must_use]
pub fn load_texture_from_memory(
    device: &Device,
    queue: &Queue,
    img: DynamicImage,
    label: &str,
) -> Texture {
    let (width, height) = img.dimensions();
    let texture_size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let (texture_format, texture_data): (TextureFormat, Vec<u8>) = match img {
        DynamicImage::ImageLuma8(buffer) => {
            debug!(
                ?label,
                "Detected Luma8 image. Using texture format R8Unorm."
            );
            if !label.contains(".alpha") {
                warn!("it is recommended that filename includes '.alpha' for luma8 textures");
            }
            (TextureFormat::R8Unorm, buffer.into_raw())
        }
        DynamicImage::ImageLumaA8(buffer) => {
            warn!(
                ?label,
                "Detected LumaA8 image. Discarding alpha channel and using Luma for alpha R8Unorm. Please do not use this format, since half of it is discarded."
            );
            if !label.contains(".alpha") {
                warn!("it is recommended that filename includes '.alpha' for luma8 textures");
            }
            // Extract only the Luma channel
            let luma_data = buffer
                .pixels()
                .flat_map(|p| [p[0]]) // p[0] is Luma, p[1] is Alpha
                .collect();
            (TextureFormat::R8Unorm, luma_data)
        }
        DynamicImage::ImageRgba8(buffer) => {
            debug!(
                ?label,
                "Detected Rgba8 image. Using texture format Rgba8UnormSrgb."
            );
            (TextureFormat::Rgba8UnormSrgb, buffer.into_raw())
        }
        DynamicImage::ImageRgb8(buffer) => {
            warn!(
                ?label,
                "Detected Rgb8 image. Converting to Rgba8. Using texture format Rgba8UnormSrgb."
            );
            let rgba_buffer = buffer.pixels().fold(
                Vec::with_capacity((width * height * 4) as usize),
                |mut acc, rgb| {
                    acc.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255u8]);
                    acc
                },
            );
            (TextureFormat::Rgba8UnormSrgb, rgba_buffer)
        }
        _ => {
            warn!(
                ?label,
                "Detected unknown format. Converting to Rgba8. Using texture format Rgba8UnormSrgb."
            );
            let rgba_buffer = img.clone().into_rgba8();
            (TextureFormat::Rgba8UnormSrgb, rgba_buffer.into_raw())
        }
    };

    let texture_descriptor = TextureDescriptor {
        label: Some(label),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: texture_format, // Use the detected format
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[texture_format],
    };

    device.create_texture_with_data(
        queue,
        &texture_descriptor,
        wgpu::util::TextureDataOrder::LayerMajor,
        &texture_data,
    )
}

#[must_use]
pub fn create_sprite_vertex_buffer(device: &Device, label: &str) -> Buffer {
    device.create_buffer_init(&util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(IDENTITY_QUAD_VERTICES),
        usage: BufferUsages::VERTEX,
    })
}

#[must_use]
pub fn create_sprite_index_buffer(device: &Device, label: &str) -> Buffer {
    device.create_buffer_init(&util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(INDICES),
        usage: BufferUsages::INDEX,
    })
}

/// Binding0: Texture
/// Binding1: Sampler
#[must_use]
pub fn create_sprite_texture_sampler_group_layout(device: &Device, label: &str) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

#[must_use]
pub fn create_sprite_texture_and_sampler_bind_group(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    texture: &Texture,
    sampler: &Sampler,
    label: &str,
) -> BindGroup {
    let texture_view = texture.create_view(&TextureViewDescriptor::default());
    device.create_bind_group(&BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(sampler),
            },
        ],
        label: Some(label),
    })
}

#[must_use]
pub fn create_quad_matrix_and_uv_instance_buffer(
    device: &Device,
    max_instances: usize,
    label: &str,
) -> Buffer {
    let buffer_size = (size_of::<SpriteInstanceUniform>() * max_instances) as BufferAddress;

    device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size: buffer_size,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_sprite_pipeline_layout(
    device: &Device,
    camera_bind_group_layout: &BindGroupLayout,
    texture_sampler_group_layout: &BindGroupLayout,
    label: &str,
) -> PipelineLayout {
    device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[camera_bind_group_layout, texture_sampler_group_layout],
        push_constant_ranges: &[],
    })
}

fn create_sprite_pipeline(
    device: &Device,
    format: TextureFormat,
    pipeline_layout: &PipelineLayout,
    vertex_shader: &ShaderModule,
    fragment_shader: &ShaderModule,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("sprite alpha blend pipeline"),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: vertex_shader,
            entry_point: Some("vs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            buffers: &[Vertex::desc(), SpriteInstanceUniform::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment_shader,
            entry_point: Some("fs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(ColorTargetState {
                format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },

        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
