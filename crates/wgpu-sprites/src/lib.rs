/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use bytemuck::{Pod, Zeroable};
use image::DynamicImage;
use image::GenericImageView;
use limnus_wgpu_math::{Matrix4, Vec4};
use tracing::{debug, warn};
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
use wgpu::{BufferBindingType, TextureView};

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
#[derive(Debug, Copy, Clone)]
struct AlphaMaskParams {
    offset: [f32; 2],
}

unsafe impl Pod for AlphaMaskParams {}
unsafe impl Zeroable for AlphaMaskParams {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SpriteInstanceUniform {
    //     transformed_pos = transformed_pos * vec2<f32>(instance.scale) + vec2<f32>(instance.position);
    //     let world_pos = vec4<f32>(transformed_pos, 0.0, 1.0);
    //     output.position = camera.view_proj * world_pos;
    //pub position: [i32; 2],      // Integer pixel positions
    //pub scale: [u32; 2],         // Integer pixel dimensions
    pub model: Matrix4, // Model Transformation matrix. use
    pub tex_coords_mul_add: Vec4,
    pub rotation: u32,
    pub color: Vec4,
}

unsafe impl Pod for SpriteInstanceUniform {}
unsafe impl Zeroable for SpriteInstanceUniform {}

impl SpriteInstanceUniform {
    #[must_use]
    pub const fn new(model: Matrix4, tex_coords_mul_add: Vec4, rotation: u32, color: Vec4) -> Self {
        Self {
            model,
            tex_coords_mul_add,
            rotation,
            color,
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
    pub sprite_shader_info: ShaderInfo,
    pub quad_shader_info: ShaderInfo,
    pub mask_shader_info: ShaderInfo,
    pub light_shader_info: ShaderInfo,
    pub virtual_to_screen_shader_info: ShaderInfo,

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

    pub alpha_mask_params_instance_buffer: Buffer,
}

const MAX_RENDER_SPRITE_COUNT: usize = 10_000;

#[derive(Debug)]
pub struct ShaderInfo {
    pub vertex_shader: ShaderModule,
    pub fragment_shader: ShaderModule,
    pub pipeline: RenderPipeline,
}

#[must_use]
pub fn create_shader_info(
    device: &Device,
    surface_texture_format: TextureFormat,
    camera_bind_group_layout: &BindGroupLayout,
    specific_layouts: &[&BindGroupLayout],
    vertex_source: &str,
    fragment_source: &str,
    blend_state: BlendState,
    name: &str,
) -> ShaderInfo {
    let mut layouts = Vec::new();
    layouts.push(camera_bind_group_layout);
    layouts.extend_from_slice(specific_layouts);

    create_shader_info_ex(
        device,
        surface_texture_format,
        &layouts,
        vertex_source,
        fragment_source,
        &[Vertex::desc(), SpriteInstanceUniform::desc()],
        blend_state,
        name,
    )
}

#[must_use]
pub fn create_shader_info_ex(
    device: &Device,
    surface_texture_format: TextureFormat,
    specific_layouts: &[&BindGroupLayout],
    vertex_source: &str,
    fragment_source: &str,
    buffers: &[VertexBufferLayout],
    blend_state: BlendState,
    name: &str,
) -> ShaderInfo {
    let vertex_shader =
        mireforge_wgpu::create_shader_module(device, &format!("{name} vertex"), vertex_source);
    let fragment_shader =
        mireforge_wgpu::create_shader_module(device, &format!("{name} fragment"), fragment_source);

    let custom_layout =
        create_pipeline_layout(device, specific_layouts, &format!("{name} pipeline layout"));

    let pipeline = create_pipeline_with_buffers(
        device,
        surface_texture_format,
        &custom_layout,
        &vertex_shader,
        &fragment_shader,
        buffers,
        blend_state,
        name,
    );

    ShaderInfo {
        vertex_shader,
        fragment_shader,
        pipeline,
    }
}

impl SpriteInfo {
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn new(
        device: &Device,
        surface_texture_format: TextureFormat,
        view_proj_matrix: Matrix4,
    ) -> Self {
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

        // Create normal sprite shader
        let (sprite_vertex_shader_source, sprite_fragment_shader_source) = normal_sprite_sources();

        let sprite_texture_sampler_bind_group_layout =
            create_texture_and_sampler_group_layout(device, "sprite texture and sampler layout");

        let alpha_blending = BlendState::ALPHA_BLENDING;

        let sprite_shader_info = create_shader_info(
            device,
            surface_texture_format,
            &camera_bind_group_layout,
            &[&sprite_texture_sampler_bind_group_layout],
            sprite_vertex_shader_source,
            sprite_fragment_shader_source,
            alpha_blending,
            "Sprite",
        );

        // Create quad shader
        let quad_shader_info = {
            let (vertex_shader_source, fragment_shader_source) = quad_shaders();

            create_shader_info(
                device,
                surface_texture_format,
                &camera_bind_group_layout,
                &[],
                vertex_shader_source,
                fragment_shader_source,
                alpha_blending,
                "Quad",
            )
        };

        let mask_shader_info = {
            let vertex_shader_source = masked_texture_tinted_vertex_source();
            let fragment_shader_source = masked_texture_tinted_fragment_source();

            let diffuse_texture_group =
                create_texture_and_sampler_group_layout(device, "normal diffuse texture group");

            let alpha_texture_group =
                create_texture_and_sampler_group_layout(device, "alpha texture group");

            create_shader_info(
                device,
                surface_texture_format,
                &camera_bind_group_layout,
                &[&diffuse_texture_group, &alpha_texture_group],
                vertex_shader_source,
                fragment_shader_source,
                alpha_blending,
                "AlphaMask",
            )
        };

        let virtual_to_screen_shader_info = {
            let virtual_texture_group_layout =
                create_texture_and_sampler_group_layout(device, "virtual texture group");
            create_shader_info_ex(
                device,
                surface_texture_format,
                &[&virtual_texture_group_layout],
                SCREEN_QUAD_VERTEX_SHADER,
                SCREEN_QUAD_FRAGMENT_SHADER,
                &[],
                alpha_blending,
                "VirtualToScreen",
            )
        };

        let light_shader_info = {
            let vertex_shader_source = sprite_vertex_shader_source;
            let fragment_shader_source = sprite_fragment_shader_source;

            let light_texture_group =
                create_texture_and_sampler_group_layout(device, "light texture group");

            let additive_blend = wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Zero,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            };

            create_shader_info(
                device,
                surface_texture_format,
                &camera_bind_group_layout,
                &[&light_texture_group],
                vertex_shader_source,
                fragment_shader_source,
                additive_blend,
                "Light (Additive)",
            )
        };

        let quad_matrix_and_uv_instance_buffer = create_quad_matrix_and_uv_instance_buffer(
            device,
            MAX_RENDER_SPRITE_COUNT,
            "sprite_instance buffer",
        );

        let sampler = mireforge_wgpu::create_nearest_sampler(device, "sprite nearest sampler");

        let alpha_mask_params_instance_buffer =
            create_alpha_mask_params_instance_buffer(device, 128, "alpha_mask instance buffer");

        Self {
            sprite_shader_info,
            quad_shader_info,
            mask_shader_info,
            light_shader_info,
            virtual_to_screen_shader_info,
            sampler,
            vertex_buffer,
            index_buffer,
            camera_bind_group_layout,
            camera_uniform_buffer,
            camera_bind_group,
            sprite_texture_sampler_bind_group_layout,
            quad_matrix_and_uv_instance_buffer,
            alpha_mask_params_instance_buffer,
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
        util::TextureDataOrder::LayerMajor,
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
pub fn create_texture_and_sampler_group_layout(device: &Device, label: &str) -> BindGroupLayout {
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
    create_texture_and_sampler_bind_group_ex(
        device,
        bind_group_layout,
        &texture_view,
        sampler,
        label,
    )
}

#[must_use]
pub fn create_texture_and_sampler_bind_group_ex(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    texture_view: &TextureView,
    sampler: &Sampler,
    label: &str,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(texture_view),
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

#[must_use]
pub fn create_alpha_mask_params_instance_buffer(
    device: &Device,
    max_instances: usize,
    label: &str,
) -> Buffer {
    let buffer_size = (size_of::<AlphaMaskParams>() * max_instances) as BufferAddress;

    device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size: buffer_size,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_pipeline_layout(
    device: &Device,
    layouts: &[&BindGroupLayout],
    label: &str,
) -> PipelineLayout {
    device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: layouts,
        push_constant_ranges: &[],
    })
}

fn create_pipeline_with_buffers(
    device: &Device,
    format: TextureFormat,
    pipeline_layout: &PipelineLayout,
    vertex_shader: &ShaderModule,
    fragment_shader: &ShaderModule,
    buffers: &[VertexBufferLayout],
    blend_state: BlendState,
    label: &str,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: vertex_shader,
            entry_point: Some("vs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment_shader,
            entry_point: Some("fs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(ColorTargetState {
                format,
                blend: Some(blend_state),
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

#[must_use]
pub const fn normal_sprite_sources() -> (&'static str, &'static str) {
    let vertex_shader_source = "
// Bind Group 0: Uniforms (view-projection matrix)
struct Uniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera_uniforms: Uniforms;

// Bind Group 1: Texture and Sampler (Unused in Vertex Shader but needed for consistency)
@group(1) @binding(0)
var diffuse_texture: texture_2d<f32>;

@group(1) @binding(1)
var sampler_diffuse: sampler;

// Vertex input structure
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @builtin(instance_index) instance_idx: u32,
};

// Vertex output structure to fragment shader
// Must be exactly the same as the fragment shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

// Vertex shader entry point
@vertex
fn vs_main(
    input: VertexInput,
    // Instance attributes
    @location(2) model_matrix0: vec4<f32>,
    @location(3) model_matrix1: vec4<f32>,
    @location(4) model_matrix2: vec4<f32>,
    @location(5) model_matrix3: vec4<f32>,
    @location(6) tex_multiplier: vec4<f32>,
    @location(7) rotation_step: u32,
    @location(8) color: vec4<f32>,
) -> VertexOutput {
    var output: VertexOutput;

    // Reconstruct the model matrix from the instance data
    let model_matrix = mat4x4<f32>(
        model_matrix0,
        model_matrix1,
        model_matrix2,
        model_matrix3,
    );

    // Compute world position
    let world_position = model_matrix * vec4<f32>(input.position, 1.0);

    // Apply view-projection matrix
    output.position = camera_uniforms.view_proj * world_position;

    // Decode rotation_step
    let rotation_val = rotation_step & 3u; // Bits 0-1
    let flip_x = (rotation_step & 4u) != 0u; // Bit 2
    let flip_y = (rotation_step & 8u) != 0u; // Bit 3

    // Rotate texture coordinates based on rotation_val
    var rotated_tex_coords = input.tex_coords;
    if (rotation_val == 1) {
        // 90 degrees rotation
        rotated_tex_coords = vec2<f32>(1.0 - input.tex_coords.y, input.tex_coords.x);
    } else if (rotation_val == 2) {
        // 180 degrees rotation
        rotated_tex_coords = vec2<f32>(1.0 - input.tex_coords.x, 1.0 - input.tex_coords.y);
    } else if (rotation_val == 3) {
        // 270 degrees rotation
        rotated_tex_coords = vec2<f32>(input.tex_coords.y, 1.0 - input.tex_coords.x);
    }
    // else rotation_val == Degrees0, no rotation

    // Apply flipping
    if (flip_x) {
        rotated_tex_coords.x = 1.0 - rotated_tex_coords.x;
    }
    if (flip_y) {
        rotated_tex_coords.y = 1.0 - rotated_tex_coords.y;
    }

    // Modify texture coordinates
    output.tex_coords = rotated_tex_coords * tex_multiplier.xy + tex_multiplier.zw;
    output.color = color;

    return output;
}
        ";
    //

    let fragment_shader_source = "

// Bind Group 1: Texture and Sampler
@group(1) @binding(0)
var diffuse_texture: texture_2d<f32>;

@group(1) @binding(1)
var sampler_diffuse: sampler;

// Fragment input structure from vertex shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

// Fragment shader entry point
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var final_color: vec4<f32>;

    // Sample the texture using the texture coordinates
    let texture_color = textureSample(diffuse_texture, sampler_diffuse, input.tex_coords);

    return  texture_color * input.color;;
}

";
    (vertex_shader_source, fragment_shader_source)
}

#[allow(unused)]
pub const fn masked_texture_tinted_fragment_source() -> &'static str {
    r"
// Masked Texture and tinted shader

// Bind Group 1: Texture and Sampler (Unused in Vertex Shader but needed for consistency?)
@group(1) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(1) @binding(1)
var sampler_diffuse: sampler;

// Bind Group 2: Texture and Sampler (Unused in Vertex Shader but needed for consistency?)
@group(2) @binding(0)
var alpha_texture: texture_2d<f32>;
@group(2) @binding(1)
var sampler_alpha: sampler;

// Must be the same as vertex shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) modified_tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) original_tex_coords: vec2<f32>, 
};

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color_sample = textureSample(diffuse_texture, sampler_diffuse, input.modified_tex_coords);
    let mask_alpha = textureSample(alpha_texture, sampler_alpha, input.original_tex_coords).r;

    let final_rgb = color_sample.rgb * input.color.rgb;
    let final_alpha = mask_alpha * input.color.a;

    return vec4<f32>(final_rgb, final_alpha);
}
    "
}

#[must_use]
pub const fn masked_texture_tinted_vertex_source() -> &'static str {
    "
// Bind Group 0: Uniforms (view-projection matrix)
struct Uniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera_uniforms: Uniforms;

// Bind Group 1: Texture and Sampler (Unused in Vertex Shader but needed for consistency?)
@group(1) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(1) @binding(1)
var sampler_diffuse: sampler;

// Bind Group 2: Texture and Sampler (Unused in Vertex Shader but needed for consistency?)
@group(2) @binding(0)
var alpha_texture: texture_2d<f32>;
@group(2) @binding(1)
var sampler_alpha: sampler;

// Vertex input structure
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @builtin(instance_index) instance_idx: u32,
};

// Vertex output structure to fragment shader
// Must be exactly the same as the fragment shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) modified_tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) original_tex_coords: vec2<f32>, 
};

// Vertex shader entry point
@vertex
fn vs_main(
    input: VertexInput,
    // Instance attributes
    @location(2) model_matrix0: vec4<f32>,
    @location(3) model_matrix1: vec4<f32>,
    @location(4) model_matrix2: vec4<f32>,
    @location(5) model_matrix3: vec4<f32>,
    @location(6) tex_multiplier: vec4<f32>,
    @location(7) rotation_step: u32,
    @location(8) color: vec4<f32>,
) -> VertexOutput {

    var output: VertexOutput;

    // Reconstruct the model matrix from the instance data
    let model_matrix = mat4x4<f32>(
        model_matrix0,
        model_matrix1,
        model_matrix2,
        model_matrix3,
    );

    // Compute world position
    let world_position = model_matrix * vec4<f32>(input.position, 1.0);

    // Apply view-projection matrix
    output.position = camera_uniforms.view_proj * world_position;

    // Decode rotation_step
    let rotation_val = rotation_step & 3u; // Bits 0-1
    let flip_x = (rotation_step & 4u) != 0u; // Bit 2
    let flip_y = (rotation_step & 8u) != 0u; // Bit 3

    // Rotate texture coordinates based on rotation_val
    var rotated_tex_coords = input.tex_coords;
    if (rotation_val == 1) {
        // 90 degrees rotation
        rotated_tex_coords = vec2<f32>(1.0 - input.tex_coords.y, input.tex_coords.x);
    } else if (rotation_val == 2) {
        // 180 degrees rotation
        rotated_tex_coords = vec2<f32>(1.0 - input.tex_coords.x, 1.0 - input.tex_coords.y);
    } else if (rotation_val == 3) {
        // 270 degrees rotation
        rotated_tex_coords = vec2<f32>(input.tex_coords.y, 1.0 - input.tex_coords.x);
    }
    // else rotation_val == Degrees0, no rotation

    // Apply flipping
    if (flip_x) {
        rotated_tex_coords.x = 1.0 - rotated_tex_coords.x;
    }
    if (flip_y) {
        rotated_tex_coords.y = 1.0 - rotated_tex_coords.y;
    }

    // Modify texture coordinates
    output.modified_tex_coords = rotated_tex_coords * tex_multiplier.xy + tex_multiplier.zw;
    output.original_tex_coords = input.tex_coords;
  
    output.color = color;

    return output;
}"
}

const fn quad_shaders() -> (&'static str, &'static str) {
    let vertex_shader_source = "
// Bind Group 0: Uniforms (view-projection matrix)
struct Uniforms {
    view_proj: mat4x4<f32>,
};
// Camera (view projection matrix) is always first
@group(0) @binding(0)
var<uniform> camera_uniforms: Uniforms;


// Vertex input structure
struct VertexInput {
    @location(0) position: vec3<f32>,
};

// Vertex output structure to fragment shader
// Must be exactly the same in both places
struct VertexOutput {
    @builtin(position) position: vec4<f32>, // MUST BE HERE, DO NOT REMOVE
    @location(0) color: vec4<f32>,
};

// Vertex shader entry point
@vertex
fn vs_main(
    input: VertexInput,
    // Instance attributes
    @location(2) model_matrix0: vec4<f32>, // Always fixed
    @location(3) model_matrix1: vec4<f32>, // Always fixed
    @location(4) model_matrix2: vec4<f32>, // Always fixed
    @location(5) model_matrix3: vec4<f32>, // Always fixed
    @location(8) color: vec4<f32>, //  Always fixed at position 8
) -> VertexOutput {
    var output: VertexOutput;

    // Reconstruct the model matrix from the instance data
    let model_matrix = mat4x4<f32>(
        model_matrix0,
        model_matrix1,
        model_matrix2,
        model_matrix3,
    );

    // Compute world position
    let world_position = model_matrix * vec4<f32>(input.position, 1.0);

    // Apply view-projection matrix
    output.position = camera_uniforms.view_proj * world_position;
    output.color = color;

    return output;
}
        ";
    //

    let fragment_shader_source = "

// Fragment input structure from vertex shader,
// Must be exactly the same in both places
struct VertexOutput {
    @builtin(position) position: vec4<f32>, // MUST BE HERE, DO NOT REMOVE
    @location(0) color: vec4<f32>,
};

// Fragment shader entry point
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // It is a quad, so we only use the color
    return input.color;
}

";
    (vertex_shader_source, fragment_shader_source)
}

pub const SCREEN_QUAD_VERTEX_SHADER: &str = "
// Define the output structure
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
};


@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );

    var texcoords = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.texcoord = texcoords[vertex_index];
    return output;
}
";

// Fragment shader for the screen quad
pub const SCREEN_QUAD_FRAGMENT_SHADER: &str = "
@group(0) @binding(0) var game_texture: texture_2d<f32>;
@group(0) @binding(1) var game_sampler: sampler;

@fragment
fn fs_main(@location(0) texcoord: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(game_texture, game_sampler, texcoord);
}
";
