/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod plugin;
pub mod prelude;

use int_math::{URect, UVec2, Vec2, Vec3};
use limnus_assets::Assets;
use limnus_assets::prelude::{Asset, Id, WeakId};
use limnus_resource::prelude::Resource;
use limnus_wgpu_math::{Matrix4, OrthoInfo, Vec4};
use mireforge_font::Font;
use mireforge_font::FontRef;
use mireforge_font::WeakFontRef;
use mireforge_render::prelude::*;
use mireforge_wgpu::create_nearest_sampler;
use mireforge_wgpu_sprites::{
    ShaderInfo, SpriteInfo, SpriteInstanceUniform, create_texture_and_sampler_bind_group_ex,
    create_texture_and_sampler_group_layout,
};
use monotonic_time_rs::Millis;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::mem::swap;
use std::sync::Arc;
use tracing::trace;
use wgpu::{BindGroup, BindGroupLayout, Buffer, CommandEncoder, RenderPipeline, TextureView};

pub type MaterialRef = Arc<Material>;

pub type WeakMaterialRef = Arc<Material>;

pub type TextureRef = Id<Texture>;
pub type WeakTextureRef = WeakId<Texture>;

pub trait FrameLookup {
    fn lookup(&self, frame: u16) -> (&MaterialRef, URect);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedAtlas {
    pub material: MaterialRef,
    pub texture_size: UVec2,
    pub one_cell_size: UVec2,
    pub cell_count_size: UVec2,
}

impl FixedAtlas {
    /// # Panics
    ///
    #[must_use]
    pub fn new(one_cell_size: UVec2, texture_size: UVec2, material_ref: MaterialRef) -> Self {
        let cell_count_size = UVec2::new(
            texture_size.x / one_cell_size.x,
            texture_size.y / one_cell_size.y,
        );

        assert_ne!(cell_count_size.x, 0, "illegal texture and one cell size");

        Self {
            material: material_ref,
            texture_size,
            one_cell_size,
            cell_count_size,
        }
    }
}

impl FrameLookup for FixedAtlas {
    fn lookup(&self, frame: u16) -> (&MaterialRef, URect) {
        let x = frame % self.cell_count_size.x;
        let y = frame / self.cell_count_size.x;

        (
            &self.material,
            URect::new(
                x * self.one_cell_size.x,
                y * self.one_cell_size.y,
                self.one_cell_size.x,
                self.one_cell_size.y,
            ),
        )
    }
}

#[derive(Debug)]
pub struct NineSliceAndMaterial {
    pub slices: Slices,
    pub material_ref: MaterialRef,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FontAndMaterial {
    pub font_ref: FontRef,
    pub material_ref: MaterialRef,
}

pub trait Gfx {
    fn sprite_atlas_frame(&mut self, position: Vec3, frame: u16, atlas: &impl FrameLookup);
    fn sprite_atlas(&mut self, position: Vec3, atlas_rect: URect, material_ref: &MaterialRef);
    fn draw_sprite(&mut self, position: Vec3, material_ref: &MaterialRef);
    fn draw_sprite_ex(&mut self, position: Vec3, material_ref: &MaterialRef, params: &SpriteParams);
    fn quad(&mut self, position: Vec3, size: UVec2, color: Color);
    fn draw_with_mask(
        &mut self,
        position: Vec3,
        size: UVec2,
        color: Color,
        alpha_masked: &MaterialRef,
    );

    fn nine_slice(
        &mut self,
        position: Vec3,
        size: UVec2,
        color: Color,
        nine_slice: &NineSliceAndMaterial,
    );
    fn set_origin(&mut self, position: Vec2);
    fn set_clear_color(&mut self, color: Color);

    fn tilemap_params(
        &mut self,
        position: Vec3,
        tiles: &[u16],
        width: u16,
        atlas_ref: &FixedAtlas,
        scale: u8,
    );

    fn text_draw(&mut self, position: Vec3, text: &str, font_ref: &FontAndMaterial, color: &Color);

    #[must_use]
    fn now(&self) -> Millis;

    #[must_use]
    fn physical_aspect_ratio(&self) -> AspectRatio;

    #[must_use]
    fn physical_size(&self) -> UVec2;

    fn set_viewport(&mut self, viewport_strategy: ViewportStrategy);

    #[must_use]
    fn viewport(&self) -> &ViewportStrategy;

    fn set_scale(&mut self, scale_factor: VirtualScale);
}

fn to_wgpu_color(c: Color) -> wgpu::Color {
    let f = c.to_f64();
    wgpu::Color {
        r: f.0,
        g: f.1,
        b: f.2,
        a: f.3,
    }
}

#[derive(Debug)]
struct RenderItem {
    position: Vec3,
    material_ref: MaterialRef,

    renderable: Renderable,
}

#[derive(Debug)]
pub struct Text {
    text: String,
    font_ref: WeakFontRef,
    color: Color,
}

#[derive(Debug)]
enum Renderable {
    Sprite(Sprite),
    QuadColor(QuadColor),
    NineSlice(NineSlice),
    TileMap(TileMap),
    Text(Text),
    Mask(UVec2, Color),
}

#[derive(Resource)]
pub struct Render {
    virtual_surface_texture_view: TextureView,
    virtual_surface_texture: wgpu::Texture,
    virtual_to_surface_bind_group: BindGroup,
    index_buffer: Buffer,  // Only indices for a single identity quad
    vertex_buffer: Buffer, // Only one identity quad (0,0,1,1)
    sampler: wgpu::Sampler,
    virtual_to_screen_shader_info: ShaderInfo,
    pub normal_sprite_pipeline: ShaderInfo,
    pub quad_shader_info: ShaderInfo,
    pub mask_shader_info: ShaderInfo,
    pub light_shader_info: ShaderInfo,
    physical_surface_size: UVec2,
    viewport_strategy: ViewportStrategy,
    // Group 0
    camera_bind_group: BindGroup,
    #[allow(unused)]
    camera_buffer: Buffer,

    // Group 1
    texture_sampler_bind_group_layout: BindGroupLayout,

    // Group 1
    quad_matrix_and_uv_instance_buffer: Buffer,

    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>, // Queue to talk to device

    // Internals
    items: Vec<RenderItem>,
    //fonts: Vec<FontAndMaterialRef>,
    origin: Vec2,

    // Cache
    batch_offsets: Vec<(WeakMaterialRef, u32, u32)>,
    viewport: URect,
    clear_color: wgpu::Color,
    last_render_at: Millis,
    scale: f32,

    debug_tick: u64,
}

impl Render {}

impl Debug for Render {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Render")
    }
}

impl Gfx for Render {
    fn sprite_atlas_frame(&mut self, position: Vec3, frame: u16, atlas: &impl FrameLookup) {
        self.sprite_atlas_frame(position, frame, atlas);
    }

    fn sprite_atlas(&mut self, position: Vec3, atlas_rect: URect, material_ref: &MaterialRef) {
        self.sprite_atlas(position, atlas_rect, material_ref);
    }

    fn draw_sprite(&mut self, position: Vec3, material_ref: &MaterialRef) {
        self.draw_sprite(position, material_ref);
    }

    fn draw_sprite_ex(
        &mut self,
        position: Vec3,
        material_ref: &MaterialRef,
        params: &SpriteParams,
    ) {
        self.draw_sprite_ex(position, material_ref, *params);
    }

    fn quad(&mut self, position: Vec3, size: UVec2, color: Color) {
        self.draw_quad(position, size, color);
    }

    fn draw_with_mask(
        &mut self,
        position: Vec3,
        size: UVec2,
        color: Color,
        alpha_masked: &MaterialRef,
    ) {
        self.push_mask(position, size, color, alpha_masked);
    }

    fn nine_slice(
        &mut self,
        position: Vec3,
        size: UVec2,
        color: Color,
        nine_slice: &NineSliceAndMaterial,
    ) {
        self.nine_slice(position, size, color, nine_slice);
    }

    fn set_origin(&mut self, position: Vec2) {
        self.origin = position;
    }

    fn set_clear_color(&mut self, color: Color) {
        self.clear_color = to_wgpu_color(color);
    }

    fn tilemap_params(
        &mut self,
        position: Vec3,
        tiles: &[u16],
        width: u16,
        atlas_ref: &FixedAtlas,
        scale: u8,
    ) {
        self.items.push(RenderItem {
            position,
            material_ref: atlas_ref.material.clone(),
            renderable: Renderable::TileMap(TileMap {
                tiles_data_grid_size: UVec2::new(width, tiles.len() as u16 / width),
                cell_count_size: atlas_ref.cell_count_size,
                one_cell_size: atlas_ref.one_cell_size,
                tiles: Vec::from(tiles),
                scale,
            }),
        });
    }

    fn text_draw(
        &mut self,
        position: Vec3,
        text: &str,
        font_and_mat: &FontAndMaterial,
        color: &Color,
    ) {
        self.items.push(RenderItem {
            position,
            material_ref: font_and_mat.material_ref.clone(),
            renderable: Renderable::Text(Text {
                text: text.to_string(),
                font_ref: (&font_and_mat.font_ref).into(),
                color: *color,
            }),
        });
    }

    fn now(&self) -> Millis {
        self.last_render_at
    }

    fn physical_aspect_ratio(&self) -> AspectRatio {
        self.physical_surface_size.into()
    }

    fn physical_size(&self) -> UVec2 {
        self.physical_surface_size
    }

    fn set_viewport(&mut self, viewport_strategy: ViewportStrategy) {
        self.viewport_strategy = viewport_strategy;
    }

    fn viewport(&self) -> &ViewportStrategy {
        &self.viewport_strategy
    }

    fn set_scale(&mut self, scale_factor: VirtualScale) {
        match scale_factor {
            VirtualScale::IntScale(scale) => self.scale = scale as f32,
            VirtualScale::FloatScale(scale) => self.scale = scale,
        }
    }
}

impl Render {
    #[must_use]
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>, // Queue to talk to device
        surface_texture_format: wgpu::TextureFormat,
        physical_size: UVec2,
        virtual_surface_size: UVec2,
        now: Millis,
    ) -> Self {
        let sprite_info = SpriteInfo::new(
            &device,
            surface_texture_format,
            create_view_uniform_view_projection_matrix(physical_size),
        );

        // Create a texture at your virtual resolution (e.g., 320x240)
        let virtual_surface_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Texture"),
            size: wgpu::Extent3d {
                width: virtual_surface_size.x as u32,
                height: virtual_surface_size.y as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_texture_format, // TODO: Check: Should probably always be same as swap chain format?
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let virtual_surface_texture_view =
            virtual_surface_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let virtual_to_screen_sampler =
            create_nearest_sampler(&device, "nearest sampler for virtual to screen");
        let virtual_to_screen_layout =
            create_texture_and_sampler_group_layout(&device, "virtual to screen layout");
        let virtual_to_surface_bind_group = create_texture_and_sampler_bind_group_ex(
            &device,
            &virtual_to_screen_layout,
            &virtual_surface_texture_view,
            &virtual_to_screen_sampler,
            "virtual to screen bind group",
        );

        Self {
            device,
            queue,
            items: Vec::new(),
            //   fonts: Vec::new(),
            virtual_to_screen_shader_info: sprite_info.virtual_to_screen_shader_info,
            virtual_surface_texture,
            virtual_surface_texture_view,
            virtual_to_surface_bind_group,
            sampler: sprite_info.sampler,
            normal_sprite_pipeline: sprite_info.sprite_shader_info,
            quad_shader_info: sprite_info.quad_shader_info,
            mask_shader_info: sprite_info.mask_shader_info,
            light_shader_info: sprite_info.light_shader_info,
            texture_sampler_bind_group_layout: sprite_info.sprite_texture_sampler_bind_group_layout,
            index_buffer: sprite_info.index_buffer,
            vertex_buffer: sprite_info.vertex_buffer,
            quad_matrix_and_uv_instance_buffer: sprite_info.quad_matrix_and_uv_instance_buffer,
            camera_bind_group: sprite_info.camera_bind_group,
            batch_offsets: Vec::new(),
            camera_buffer: sprite_info.camera_uniform_buffer,
            viewport: Self::viewport_from_integer_scale(physical_size, virtual_surface_size),
            clear_color: to_wgpu_color(Color::from_f32(0.008, 0.015, 0.008, 1.0)),
            origin: Vec2::new(0, 0),
            last_render_at: now,
            physical_surface_size: physical_size,
            viewport_strategy: ViewportStrategy::FitIntegerScaling(virtual_surface_size),
            scale: 1.0,
            debug_tick: 0,
        }
    }

    pub const fn set_now(&mut self, now: Millis) {
        self.last_render_at = now;
    }

    pub const fn virtual_surface_size(&self) -> UVec2 {
        match self.viewport_strategy {
            ViewportStrategy::FitIntegerScaling(virtual_size)
            | ViewportStrategy::FitFloatScaling(virtual_size) => virtual_size,
            ViewportStrategy::MatchPhysicalSize => self.physical_surface_size,
        }
    }

    pub const fn physical_surface_size(&self) -> UVec2 {
        self.physical_surface_size
    }

    pub const fn viewport(&self) -> URect {
        self.viewport
    }

    #[inline]
    fn push_sprite(&mut self, position: Vec3, material: &MaterialRef, sprite: Sprite) {
        self.items.push(RenderItem {
            position,
            material_ref: material.clone(),
            renderable: Renderable::Sprite(sprite),
        });
    }

    pub fn push_mask(
        &mut self,
        position: Vec3,
        size: UVec2,
        color: Color,
        alpha_masked: &MaterialRef,
    ) {
        self.items.push(RenderItem {
            position,
            material_ref: alpha_masked.clone(),
            renderable: Renderable::Mask(size, color),
        });
    }

    pub fn push_nine_slice(
        &mut self,
        position: Vec3,
        size: UVec2,
        color: Color,
        nine_slice_and_material: &NineSliceAndMaterial,
    ) {
        let nine_slice_info = NineSlice {
            size,
            slices: nine_slice_and_material.slices,
            color,
            origin_in_atlas: UVec2::new(0, 0),
            size_inside_atlas: None,
        };

        self.items.push(RenderItem {
            position,
            material_ref: nine_slice_and_material.material_ref.clone(),
            renderable: Renderable::NineSlice(nine_slice_info),
        });
    }

    #[must_use]
    pub fn viewport_from_integer_scale(physical_size: UVec2, virtual_size: UVec2) -> URect {
        let window_aspect = physical_size.x as f32 / physical_size.y as f32;
        let virtual_aspect = virtual_size.x as f32 / virtual_size.y as f32;

        if physical_size.x < virtual_size.x || physical_size.y < virtual_size.y {
            return URect::new(0, 0, physical_size.x, physical_size.y);
        }

        let mut integer_scale = if window_aspect > virtual_aspect {
            physical_size.y / virtual_size.y
        } else {
            physical_size.x / virtual_size.x
        };

        if integer_scale < 1 {
            integer_scale = 1;
        }

        let viewport_actual_size = UVec2::new(
            virtual_size.x * integer_scale,
            virtual_size.y * integer_scale,
        );

        let border_size = physical_size - viewport_actual_size;

        let offset = border_size / 2;

        URect::new(
            offset.x,
            offset.y,
            viewport_actual_size.x,
            viewport_actual_size.y,
        )
    }

    #[must_use]
    pub fn viewport_from_float_scale(physical_size: UVec2, virtual_size: UVec2) -> URect {
        let window_aspect = physical_size.x as f32 / physical_size.y as f32;
        let virtual_aspect = virtual_size.x as f32 / virtual_size.y as f32;

        if physical_size.x < virtual_size.x || physical_size.y < virtual_size.y {
            return URect::new(0, 0, physical_size.x, physical_size.y);
        }

        let mut float_scale = if window_aspect > virtual_aspect {
            physical_size.y as f32 / virtual_size.y as f32
        } else {
            physical_size.x as f32 / virtual_size.x as f32
        };

        if float_scale < 0.01 {
            float_scale = 0.01;
        }

        let viewport_actual_size = UVec2::new(
            (virtual_size.x as f32 * float_scale) as u16,
            (virtual_size.y as f32 * float_scale) as u16,
        );

        let border_size = physical_size - viewport_actual_size;

        let offset = border_size / 2;

        URect::new(
            offset.x,
            offset.y,
            viewport_actual_size.x,
            viewport_actual_size.y,
        )
    }

    pub fn resize(&mut self, physical_size: UVec2) {
        self.physical_surface_size = physical_size;
    }

    pub fn sprite_atlas(&mut self, position: Vec3, atlas_rect: URect, material_ref: &MaterialRef) {
        self.push_sprite(
            position,
            material_ref,
            Sprite {
                params: SpriteParams {
                    texture_pos: atlas_rect.position,
                    texture_size: atlas_rect.size,
                    ..Default::default()
                },
            },
        );
    }

    pub fn sprite_atlas_frame(&mut self, position: Vec3, frame: u16, atlas: &impl FrameLookup) {
        let (material_ref, atlas_rect) = atlas.lookup(frame);
        self.push_sprite(
            position,
            material_ref,
            Sprite {
                params: SpriteParams {
                    texture_pos: atlas_rect.position,
                    texture_size: atlas_rect.size,
                    ..Default::default()
                },
            },
        );
    }

    pub fn sprite_atlas_frame_ex(
        &mut self,
        position: Vec3,
        frame: u16,
        atlas: &impl FrameLookup,
        mut params: SpriteParams,
    ) {
        let (material_ref, atlas_rect) = atlas.lookup(frame);
        params.texture_pos = atlas_rect.position;
        params.texture_size = atlas_rect.size;
        self.push_sprite(position, material_ref, Sprite { params });
    }

    pub fn draw_sprite(&mut self, position: Vec3, material: &MaterialRef) {
        self.push_sprite(
            position,
            material,
            Sprite {
                params: SpriteParams::default(),
            },
        );
    }

    pub fn draw_sprite_ex(&mut self, position: Vec3, material: &MaterialRef, params: SpriteParams) {
        self.push_sprite(position, material, Sprite { params });
    }

    pub fn nine_slice(
        &mut self,
        position: Vec3,
        size: UVec2,
        color: Color,
        nine_slice_and_material: &NineSliceAndMaterial,
    ) {
        self.push_nine_slice(position, size, color, nine_slice_and_material);
    }

    pub fn draw_quad(&mut self, position: Vec3, size: UVec2, color: Color) {
        let material = Material {
            base: MaterialBase {
                //pipeline: self.normal_sprite_pipeline.clone(),
            },
            kind: MaterialKind::Quad,
        };

        //debug!(?position, ?size, ?color, "draw quad");

        self.items.push(RenderItem {
            position,
            material_ref: MaterialRef::from(material),
            renderable: Renderable::QuadColor(QuadColor { size, color }),
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_nine_slice(
        &mut self,
        position: Vec3,
        size: UVec2,
        slices: Slices,
        material_ref: &MaterialRef,
        color: Color,
    ) {
        self.items.push(RenderItem {
            position,
            material_ref: material_ref.clone(),
            renderable: Renderable::NineSlice(NineSlice {
                size,
                slices,
                color,
                origin_in_atlas: UVec2::new(0, 0),
                size_inside_atlas: None,
            }),
        });
    }

    // TODO: Not done yet
    /*
    #[allow(clippy::too_many_arguments)]
    pub fn draw_nine_slice_atlas(
        &mut self,
        position: Vec3,
        size: UVec2,
        corner_size: UVec2,
        texture_window_size: UVec2,
        material_ref: &MaterialRef,
        atlas_offset: UVec2,
        color: Color,
    ) {
        self.items.push(RenderItem {
            position,
            material_ref: material_ref.into(),
            renderable: Renderable::NineSlice(NineSlice {
                corner_size,
                texture_window_size,
                size,
                atlas_offset,
                color,
            }),
        });
    }

     */

    pub const fn clear_color(&self) -> wgpu::Color {
        self.clear_color
    }

    // first two is multiplier and second pair is offset
    fn calculate_texture_coords_mul_add(atlas_rect: URect, texture_size: UVec2) -> Vec4 {
        let x = atlas_rect.position.x as f32 / texture_size.x as f32;
        let y = atlas_rect.position.y as f32 / texture_size.y as f32;
        let width = atlas_rect.size.x as f32 / texture_size.x as f32;
        let height = atlas_rect.size.y as f32 / texture_size.y as f32;
        Vec4([width, height, x, y])
    }

    fn order_render_items_in_batches(&self) -> Vec<Vec<&RenderItem>> {
        let mut material_batches: Vec<Vec<&RenderItem>> = Vec::new();
        let mut current_batch: Vec<&RenderItem> = Vec::new();
        let mut current_material: Option<MaterialRef> = None;

        for render_item in &self.items {
            if Some(&render_item.material_ref) != current_material.as_ref() {
                if !current_batch.is_empty() {
                    material_batches.push(current_batch.clone());
                    current_batch.clear();
                }
                current_material = Some(render_item.material_ref.clone());
            }
            current_batch.push(render_item);
        }

        if !current_batch.is_empty() {
            material_batches.push(current_batch);
        }

        material_batches
    }

    #[must_use]
    pub fn quad_helper_uniform(
        position: Vec3,
        quad_size: UVec2,
        render_atlas: URect,
        color: Color,

        current_texture_size: UVec2,
    ) -> SpriteInstanceUniform {
        let model_matrix = Matrix4::from_translation(position.x as f32, position.y as f32, 0.0)
            * Matrix4::from_scale(quad_size.x as f32, quad_size.y as f32, 1.0);

        let tex_coords_mul_add =
            Self::calculate_texture_coords_mul_add(render_atlas, current_texture_size);

        let rotation_value = 0;

        SpriteInstanceUniform::new(
            model_matrix,
            tex_coords_mul_add,
            rotation_value,
            Vec4(color.to_f32_slice()),
        )
    }

    /// # Panics
    ///
    #[allow(clippy::too_many_lines)]
    pub fn prepare_render(&mut self, textures: &Assets<Texture>, fonts: &Assets<Font>) {
        const FLIP_X_MASK: u32 = 0b0000_0100;
        const FLIP_Y_MASK: u32 = 0b0000_1000;

        sort_render_items_by_z_and_material(&mut self.items);

        let batches = self.order_render_items_in_batches();

        let mut quad_matrix_and_uv: Vec<SpriteInstanceUniform> = Vec::new();
        let mut batch_vertex_ranges: Vec<(MaterialRef, u32, u32)> = Vec::new();

        for render_items in &batches {
            let quad_len_before = quad_matrix_and_uv.len() as u32;

            // Fix: Access material_ref through reference and copy it
            let weak_material_ref = render_items
                .first()
                .map(|item| {
                    // Force copy semantics by dereferencing the shared reference
                    let material_ref: MaterialRef = item.material_ref.clone();
                    material_ref
                })
                .expect("Render items batch was empty");

            if !weak_material_ref.is_complete(textures) {
                // Material is not loaded yet
                trace!(?weak_material_ref, "material is not complete yet");
                continue;
            }
            let material = weak_material_ref.clone();

            let maybe_texture_ref = material.primary_texture();
            let maybe_texture = maybe_texture_ref.and_then(|found_primary_texture_ref| {
                let found_primary_texture = textures.get(&found_primary_texture_ref);
                found_primary_texture
            });

            for render_item in render_items {
                match &render_item.renderable {
                    Renderable::Sprite(sprite) => {
                        let current_texture_size = maybe_texture.unwrap().texture_size;

                        let params = &sprite.params;
                        let mut size = params.texture_size;
                        if size.x == 0 && size.y == 0 {
                            size = current_texture_size;
                        }

                        let render_atlas = URect {
                            position: params.texture_pos,
                            size,
                        };

                        match params.rotation {
                            Rotation::Degrees90 | Rotation::Degrees270 => {
                                swap(&mut size.x, &mut size.y);
                            }
                            _ => {}
                        }

                        let model_matrix = Matrix4::from_translation(
                            render_item.position.x as f32,
                            render_item.position.y as f32,
                            0.0,
                        ) * Matrix4::from_scale(
                            (size.x * params.scale as u16) as f32,
                            (size.y * params.scale as u16) as f32,
                            1.0,
                        );

                        let tex_coords_mul_add = Self::calculate_texture_coords_mul_add(
                            render_atlas,
                            current_texture_size,
                        );

                        let mut rotation_value = match params.rotation {
                            Rotation::Degrees0 => 0,
                            Rotation::Degrees90 => 1,
                            Rotation::Degrees180 => 2,
                            Rotation::Degrees270 => 3,
                        };

                        if params.flip_x {
                            rotation_value |= FLIP_X_MASK;
                        }
                        if params.flip_y {
                            rotation_value |= FLIP_Y_MASK;
                        }

                        let quad_instance = SpriteInstanceUniform::new(
                            model_matrix,
                            tex_coords_mul_add,
                            rotation_value,
                            Vec4(params.color.to_f32_slice()),
                        );
                        quad_matrix_and_uv.push(quad_instance);
                    }

                    Renderable::Mask(_size, color) => {
                        let current_texture_size = maybe_texture.unwrap().texture_size;
                        let params = SpriteParams {
                            texture_size: current_texture_size,
                            texture_pos: UVec2 { x: 0, y: 0 },
                            scale: 1,
                            rotation: Rotation::default(),
                            flip_x: false,
                            flip_y: false,
                            pivot: Vec2 { x: 0, y: 0 },
                            color: *color,
                        };

                        let mut size = params.texture_size;
                        if size.x == 0 && size.y == 0 {
                            size = current_texture_size;
                        }

                        let render_atlas = URect {
                            position: params.texture_pos,
                            size,
                        };

                        let model_matrix = Matrix4::from_translation(
                            render_item.position.x as f32,
                            render_item.position.y as f32,
                            0.0,
                        ) * Matrix4::from_scale(
                            (size.x * params.scale as u16) as f32,
                            (size.y * params.scale as u16) as f32,
                            1.0,
                        );

                        let tex_coords_mul_add = Self::calculate_texture_coords_mul_add(
                            render_atlas,
                            current_texture_size,
                        );

                        let mut rotation_value = match params.rotation {
                            Rotation::Degrees0 => 0,
                            Rotation::Degrees90 => 1,
                            Rotation::Degrees180 => 2,
                            Rotation::Degrees270 => 3,
                        };

                        if params.flip_x {
                            rotation_value |= FLIP_X_MASK;
                        }
                        if params.flip_y {
                            rotation_value |= FLIP_Y_MASK;
                        }

                        let quad_instance = SpriteInstanceUniform::new(
                            model_matrix,
                            tex_coords_mul_add,
                            rotation_value,
                            Vec4(params.color.to_f32_slice()),
                        );
                        quad_matrix_and_uv.push(quad_instance);
                    }

                    Renderable::NineSlice(nine_slice) => {
                        let current_texture_size = maybe_texture.unwrap().texture_size;
                        Self::prepare_nine_slice(
                            nine_slice,
                            render_item.position,
                            &mut quad_matrix_and_uv,
                            current_texture_size,
                        );
                    }

                    Renderable::QuadColor(quad) => {
                        let model_matrix =
                            Matrix4::from_translation(
                                render_item.position.x as f32,
                                render_item.position.y as f32,
                                0.0,
                            ) * Matrix4::from_scale(quad.size.x as f32, quad.size.y as f32, 1.0);

                        let tex_coords_mul_add = Vec4([
                            0.0, //x
                            0.0, //y
                            0.0, 0.0,
                        ]);
                        let rotation_value = 0;

                        let quad_instance = SpriteInstanceUniform::new(
                            model_matrix,
                            tex_coords_mul_add,
                            rotation_value,
                            Vec4(quad.color.to_f32_slice()),
                        );
                        quad_matrix_and_uv.push(quad_instance);
                    }

                    Renderable::Text(text) => {
                        let current_texture_size = maybe_texture.unwrap().texture_size;
                        let result = fonts.get_weak(text.font_ref);
                        if result.is_none() {
                            continue;
                        }
                        let font = result.unwrap();

                        let glyphs = font.draw(&text.text);
                        for glyph in glyphs {
                            let pos = render_item.position + Vec3::from(glyph.relative_position);
                            let texture_size = glyph.texture_rectangle.size;
                            let model_matrix =
                                Matrix4::from_translation(pos.x as f32, pos.y as f32, 0.0)
                                    * Matrix4::from_scale(
                                        texture_size.x as f32,
                                        texture_size.y as f32,
                                        1.0,
                                    );
                            let tex_coords_mul_add = Self::calculate_texture_coords_mul_add(
                                glyph.texture_rectangle,
                                current_texture_size,
                            );

                            let quad_instance = SpriteInstanceUniform::new(
                                model_matrix,
                                tex_coords_mul_add,
                                0,
                                Vec4(text.color.to_f32_slice()),
                            );
                            quad_matrix_and_uv.push(quad_instance);
                        }
                    }

                    Renderable::TileMap(tile_map) => {
                        for (index, tile) in tile_map.tiles.iter().enumerate() {
                            let cell_pos_x = (index as u16 % tile_map.tiles_data_grid_size.x)
                                * tile_map.one_cell_size.x
                                * tile_map.scale as u16;
                            let cell_pos_y = (index as u16 / tile_map.tiles_data_grid_size.x)
                                * tile_map.one_cell_size.y
                                * tile_map.scale as u16;
                            let cell_x = *tile % tile_map.cell_count_size.x;
                            let cell_y = *tile / tile_map.cell_count_size.x;

                            let tex_x = cell_x * tile_map.one_cell_size.x;
                            let tex_y = cell_y * tile_map.one_cell_size.x;

                            let cell_texture_area = URect::new(
                                tex_x,
                                tex_y,
                                tile_map.one_cell_size.x,
                                tile_map.one_cell_size.y,
                            );

                            let cell_model_matrix = Matrix4::from_translation(
                                (render_item.position.x + cell_pos_x as i16) as f32,
                                (render_item.position.y + cell_pos_y as i16) as f32,
                                0.0,
                            ) * Matrix4::from_scale(
                                (tile_map.one_cell_size.x * tile_map.scale as u16) as f32,
                                (tile_map.one_cell_size.y * tile_map.scale as u16) as f32,
                                1.0,
                            );

                            let current_texture_size = maybe_texture.unwrap().texture_size;
                            let cell_tex_coords_mul_add = Self::calculate_texture_coords_mul_add(
                                cell_texture_area,
                                current_texture_size,
                            );

                            let quad_instance = SpriteInstanceUniform::new(
                                cell_model_matrix,
                                cell_tex_coords_mul_add,
                                0,
                                Vec4([1.0, 1.0, 1.0, 1.0]),
                            );
                            quad_matrix_and_uv.push(quad_instance);
                        }
                    }
                }
            }

            let quad_count = quad_matrix_and_uv.len() as u32 - quad_len_before;
            batch_vertex_ranges.push((weak_material_ref, quad_len_before, quad_count));
        }

        // write all model_matrix and uv_coords to instance buffer once, before the render pass
        self.queue.write_buffer(
            &self.quad_matrix_and_uv_instance_buffer,
            0,
            bytemuck::cast_slice(&quad_matrix_and_uv),
        );

        self.batch_offsets = batch_vertex_ranges;
    }

    #[allow(clippy::too_many_lines)]
    #[inline]
    pub fn prepare_nine_slice(
        nine_slice: &NineSlice,
        position_offset: Vec3,
        quad_matrix_and_uv: &mut Vec<SpriteInstanceUniform>,
        current_texture_size: UVec2,
    ) {
        let color = nine_slice.color;
        let world_window_size = nine_slice.size;
        let slices = &nine_slice.slices;
        let atlas_origin = nine_slice.origin_in_atlas;
        let texture_window_size = nine_slice.size_inside_atlas.unwrap_or(current_texture_size);

        let world_edge_width = nine_slice.size.x - slices.left - slices.right;
        let world_edge_height = nine_slice.size.y - slices.top - slices.bottom;
        let texture_edge_width = texture_window_size.x - slices.left - slices.right;
        let texture_edge_height = texture_window_size.y - slices.top - slices.bottom;

        // Lower left Corner
        // Y goes up, X goes to the right, right-handed coordinate system
        let lower_left_pos = Vec3::new(position_offset.x, position_offset.y, 0);
        let corner_size = UVec2::new(slices.left, slices.bottom);
        // it should be pixel perfect so it is the same size as the texture cut out
        let lower_left_quad_size = UVec2::new(corner_size.x, corner_size.y);
        let lower_left_atlas = URect::new(
            atlas_origin.x,
            atlas_origin.y + texture_window_size.y - slices.bottom, // Bottom of texture minus bottom slice height
            corner_size.x,
            corner_size.y,
        );
        let lower_left_quad = Self::quad_helper_uniform(
            lower_left_pos,
            lower_left_quad_size,
            lower_left_atlas,
            color,
            current_texture_size,
        );
        quad_matrix_and_uv.push(lower_left_quad);

        // Lower edge
        let lower_side_position =
            Vec3::new(position_offset.x + slices.left as i16, position_offset.y, 0);
        // World quad size is potentially wider than the texture,
        // that is fine, since the texture will be repeated.
        let lower_side_world_quad_size = UVec2::new(world_edge_width, slices.bottom);
        let lower_side_texture_size = UVec2::new(texture_edge_width, slices.bottom);
        // Lower edge
        let lower_side_atlas = URect::new(
            atlas_origin.x + slices.left,
            atlas_origin.y + texture_window_size.y - slices.bottom, // Bottom of texture minus bottom slice height
            lower_side_texture_size.x,
            lower_side_texture_size.y,
        );
        let lower_side_quad = Self::quad_helper_uniform(
            lower_side_position,
            lower_side_world_quad_size,
            lower_side_atlas,
            color,
            current_texture_size,
        );
        quad_matrix_and_uv.push(lower_side_quad);

        // Lower right corner
        let lower_right_pos = Vec3::new(
            position_offset.x + (world_window_size.x - slices.right) as i16,
            position_offset.y,
            0,
        );
        let lower_right_corner_size = UVec2::new(slices.right, slices.bottom);
        let lower_right_atlas = URect::new(
            atlas_origin.x + texture_window_size.x - slices.right,
            atlas_origin.y + texture_window_size.y - slices.bottom, // Bottom of texture minus bottom slice height
            lower_right_corner_size.x,
            lower_right_corner_size.y,
        );
        let lower_right_quad = Self::quad_helper_uniform(
            lower_right_pos,
            lower_right_corner_size,
            lower_right_atlas,
            color,
            current_texture_size,
        );
        quad_matrix_and_uv.push(lower_right_quad);

        // Left edge
        let left_edge_pos = Vec3::new(
            position_offset.x,
            position_offset.y + slices.bottom as i16,
            0,
        );
        let left_edge_world_quad_size = UVec2::new(slices.left, world_edge_height);
        let left_edge_texture_size = UVec2::new(slices.left, texture_edge_height);
        let left_edge_atlas = URect::new(
            atlas_origin.x,
            atlas_origin.y + slices.top, // Skip top slice
            left_edge_texture_size.x,
            left_edge_texture_size.y,
        );
        let left_edge_quad = Self::quad_helper_uniform(
            left_edge_pos,
            left_edge_world_quad_size,
            left_edge_atlas,
            color,
            current_texture_size,
        );
        quad_matrix_and_uv.push(left_edge_quad);

        // CENTER IS COMPLICATED
        // This was pretty tricky to get going, but @catnipped wanted it :)

        // For the center region, we have to create multiple quads, since we want
        // it pixel perfect and not stretch it
        let base_center_x = atlas_origin.x + slices.left;
        let base_center_y = atlas_origin.y + slices.top;

        // Calculate how many repetitions (quads) we need in each direction
        let repeat_x_count = (world_edge_width as f32 / texture_edge_width as f32).ceil() as usize;
        let repeat_y_count =
            (world_edge_height as f32 / texture_edge_height as f32).ceil() as usize;

        for y in 0..repeat_y_count {
            for x in 0..repeat_x_count {
                let this_quad_width =
                    if x == repeat_x_count - 1 && world_edge_width % texture_edge_width != 0 {
                        world_edge_width % texture_edge_width
                    } else {
                        texture_edge_width
                    };

                let this_quad_height =
                    if y == repeat_y_count - 1 && world_edge_height % texture_edge_height != 0 {
                        world_edge_height % texture_edge_height
                    } else {
                        texture_edge_height
                    };

                let quad_pos = Vec3::new(
                    position_offset.x + slices.left as i16 + (x as u16 * texture_edge_width) as i16,
                    position_offset.y
                        + slices.bottom as i16
                        + (y as u16 * texture_edge_height) as i16,
                    0,
                );

                let texture_x = base_center_x;

                let texture_y = if y == repeat_y_count - 1 && this_quad_height < texture_edge_height
                {
                    base_center_y + (texture_edge_height - this_quad_height)
                } else {
                    base_center_y
                };

                let this_texture_region =
                    URect::new(texture_x, texture_y, this_quad_width, this_quad_height);

                let center_quad = Self::quad_helper_uniform(
                    quad_pos,
                    UVec2::new(this_quad_width, this_quad_height),
                    this_texture_region,
                    color,
                    current_texture_size,
                );

                quad_matrix_and_uv.push(center_quad);
            }
        }
        // CENTER IS DONE ---------

        // Right edge
        let right_edge_pos = Vec3::new(
            position_offset.x + (world_window_size.x - slices.right) as i16,
            position_offset.y + slices.bottom as i16,
            0,
        );
        let right_edge_world_quad_size = UVec2::new(slices.right, world_edge_height);
        let right_edge_texture_size = UVec2::new(slices.right, texture_edge_height);
        let right_edge_atlas = URect::new(
            atlas_origin.x + texture_window_size.x - slices.right,
            atlas_origin.y + slices.top, // Skip top slice
            right_edge_texture_size.x,
            right_edge_texture_size.y,
        );

        let right_edge_quad = Self::quad_helper_uniform(
            right_edge_pos,
            right_edge_world_quad_size,
            right_edge_atlas,
            color,
            current_texture_size,
        );
        quad_matrix_and_uv.push(right_edge_quad);

        // Top left corner
        let top_left_pos = Vec3::new(
            position_offset.x,
            position_offset.y + (world_window_size.y - slices.top) as i16,
            0,
        );
        let top_left_corner_size = UVec2::new(slices.left, slices.top);
        let top_left_atlas = URect::new(
            atlas_origin.x,
            atlas_origin.y, // Top of texture
            top_left_corner_size.x,
            top_left_corner_size.y,
        );
        let top_left_quad = Self::quad_helper_uniform(
            top_left_pos,
            top_left_corner_size,
            top_left_atlas,
            color,
            current_texture_size,
        );
        quad_matrix_and_uv.push(top_left_quad);

        // Top edge
        let top_edge_pos = Vec3::new(
            position_offset.x + slices.left as i16,
            position_offset.y + (world_window_size.y - slices.top) as i16,
            0,
        );
        let top_edge_world_quad_size = UVec2::new(world_edge_width, slices.top);
        let top_edge_texture_size = UVec2::new(texture_edge_width, slices.top);
        let top_edge_atlas = URect::new(
            atlas_origin.x + slices.left,
            atlas_origin.y, // Top of texture
            top_edge_texture_size.x,
            top_edge_texture_size.y,
        );
        let top_edge_quad = Self::quad_helper_uniform(
            top_edge_pos,
            top_edge_world_quad_size,
            top_edge_atlas,
            color,
            current_texture_size,
        );
        quad_matrix_and_uv.push(top_edge_quad);

        // Top right corner
        let top_right_pos = Vec3::new(
            position_offset.x + (world_window_size.x - slices.right) as i16,
            position_offset.y + (world_window_size.y - slices.top) as i16,
            0,
        );
        let top_right_corner_size = UVec2::new(slices.right, slices.top);
        let top_right_atlas = URect::new(
            atlas_origin.x + texture_window_size.x - slices.right,
            atlas_origin.y, // Top of texture
            top_right_corner_size.x,
            top_right_corner_size.y,
        );
        let top_right_quad = Self::quad_helper_uniform(
            top_right_pos,
            top_right_corner_size,
            top_right_atlas,
            color,
            current_texture_size,
        );
        quad_matrix_and_uv.push(top_right_quad);
    }

    /// # Panics
    ///
    #[allow(clippy::too_many_lines)]
    pub fn render(
        &mut self,
        command_encoder: &mut CommandEncoder,
        display_surface_texture_view: &TextureView,
        //        materials: &Assets<Material>,
        textures: &Assets<Texture>,
        fonts: &Assets<Font>,
        now: Millis,
    ) {
        self.debug_tick += 1;
        trace!("start render()");
        self.last_render_at = now;

        self.viewport = match self.viewport_strategy {
            ViewportStrategy::FitIntegerScaling(virtual_surface_size) => {
                Self::viewport_from_integer_scale(self.physical_surface_size, virtual_surface_size)
            }
            ViewportStrategy::FitFloatScaling(virtual_surface_size) => {
                Self::viewport_from_float_scale(self.physical_surface_size, virtual_surface_size)
            }
            ViewportStrategy::MatchPhysicalSize => URect::new(
                0,
                0,
                self.physical_surface_size.x,
                self.physical_surface_size.y,
            ),
        };

        let view_proj_matrix = match self.viewport_strategy {
            ViewportStrategy::MatchPhysicalSize => {
                create_view_uniform_view_projection_matrix(self.physical_surface_size)
            }
            ViewportStrategy::FitFloatScaling(virtual_surface_size)
            | ViewportStrategy::FitIntegerScaling(virtual_surface_size) => {
                create_view_projection_matrix_from_virtual(
                    virtual_surface_size.x,
                    virtual_surface_size.y,
                )
            }
        };

        let scale_matrix = Matrix4::from_scale(self.scale, self.scale, 0.0);
        let origin_translation_matrix =
            Matrix4::from_translation(-self.origin.x as f32, -self.origin.y as f32, 0.0);

        let total_matrix = scale_matrix * view_proj_matrix * origin_translation_matrix;

        // write all model_matrix and uv_coords to instance buffer once, before the render pass
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[total_matrix]),
        );

        self.prepare_render(textures, fonts);

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.virtual_surface_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_viewport(
                0.0,
                0.0,
                self.virtual_surface_size().x as f32,
                self.virtual_surface_size().y as f32,
                0.0,
                1.0,
            );

            // Index and vertex buffers never change
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // Vertex buffer is reused
            render_pass.set_vertex_buffer(1, self.quad_matrix_and_uv_instance_buffer.slice(..));

            let num_indices = mireforge_wgpu_sprites::INDICES.len() as u32;

            let mut current_pipeline: Option<&MaterialKind> = None;

            for &(ref weak_material_ref, start, count) in &self.batch_offsets {
                let wgpu_material = weak_material_ref;

                let pipeline_kind = &wgpu_material.kind;

                if current_pipeline != Some(pipeline_kind) {
                    let pipeline = match pipeline_kind {
                        MaterialKind::NormalSprite { .. } => &self.normal_sprite_pipeline.pipeline,
                        MaterialKind::Quad => &self.quad_shader_info.pipeline,
                        MaterialKind::AlphaMasker { .. } => &self.mask_shader_info.pipeline,
                        MaterialKind::LightAdd { .. } => &self.light_shader_info.pipeline,
                    };
                    //trace!(%pipeline_kind, ?pipeline, "setting pipeline");
                    render_pass.set_pipeline(pipeline);
                    // Apparently after setting pipeline,
                    // you must set all bind groups again
                    current_pipeline = Some(pipeline_kind);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                }

                match &wgpu_material.kind {
                    MaterialKind::NormalSprite { primary_texture }
                    | MaterialKind::LightAdd { primary_texture } => {
                        let texture = textures.get(primary_texture).unwrap();
                        // Bind the texture and sampler bind group (Bind Group 1)
                        render_pass.set_bind_group(1, &texture.texture_and_sampler_bind_group, &[]);
                    }
                    MaterialKind::AlphaMasker {
                        primary_texture,
                        alpha_texture,
                    } => {
                        let real_diffuse_texture = textures.get(primary_texture).unwrap();
                        let alpha_texture = textures.get(alpha_texture).unwrap();
                        render_pass.set_bind_group(
                            1,
                            &real_diffuse_texture.texture_and_sampler_bind_group,
                            &[],
                        );
                        render_pass.set_bind_group(
                            2,
                            &alpha_texture.texture_and_sampler_bind_group,
                            &[],
                        );
                    }
                    MaterialKind::Quad => {
                        trace!("set quad material");
                        // Intentionally do nothing
                    }
                }

                // Issue the instanced draw call for the batch
                trace!(material=%weak_material_ref, start=%start, count=%count, %num_indices, "draw instanced");
                render_pass.draw_indexed(0..num_indices, 0, start..(start + count));
            }
        }

        self.items.clear();

        self.render_virtual_texture_to_display(command_encoder, display_surface_texture_view);
    }

    pub fn render_virtual_texture_to_display(
        &mut self,
        command_encoder: &mut CommandEncoder,
        display_surface_texture_view: &TextureView,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Screen Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: display_surface_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        /*
        let scale_x = window_width as f32 / VIRTUAL_WIDTH as f32;
        let scale_y = window_height as f32 / VIRTUAL_HEIGHT as f32;
        let scale = scale_x.min(scale_y).floor(); // Use integer scaling

        let viewport_width = VIRTUAL_WIDTH as f32 * scale;
        let viewport_height = VIRTUAL_HEIGHT as f32 * scale;
        let viewport_x = (window_width as f32 - viewport_width) / 2.0;
        let viewport_y = (window_height as f32 - viewport_height) / 2.0;
         */

        render_pass.set_viewport(
            0f32,
            0f32,
            self.viewport.size.x as f32,
            self.viewport.size.y as f32,
            0.0,
            1.0,
        );

        // Draw the render texture to the screen
        render_pass.set_pipeline(&self.virtual_to_screen_shader_info.pipeline);
        render_pass.set_bind_group(0, &self.virtual_to_surface_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        render_pass.draw(0..6, 0..1);
    }

    pub fn texture_resource_from_texture(&self, texture: &wgpu::Texture, label: &str) -> Texture {
        trace!("load texture from memory with name: '{label}'");
        let size = &texture.size();
        let texture_and_sampler_bind_group =
            mireforge_wgpu_sprites::create_sprite_texture_and_sampler_bind_group(
                &self.device,
                &self.texture_sampler_bind_group_layout,
                &texture,
                &self.sampler,
                label,
            );

        let texture_size = UVec2::new(size.width as u16, size.height as u16);

        Texture {
            texture_and_sampler_bind_group,
            //pipeline: Arc::clone(&self.pipeline),
            texture_size,
        }
    }
}

//fn set_view_projection(&mut self) {}

fn create_view_projection_matrix_from_virtual(virtual_width: u16, virtual_height: u16) -> Matrix4 {
    OrthoInfo {
        left: 0.0,
        right: virtual_width as f32,
        bottom: 0.0,
        top: virtual_height as f32,
        near: 1.0, // Maybe flipped? -1.0
        far: -1.0, // maybe flipped? 1.0 or 0.0
    }
    .into()
}

fn create_view_uniform_view_projection_matrix(viewport_size: UVec2) -> Matrix4 {
    let viewport_width = viewport_size.x as f32;
    let viewport_height = viewport_size.y as f32;

    let viewport_aspect_ratio = viewport_width / viewport_height;

    let scale_x = 1.0;
    let scale_y = viewport_aspect_ratio; // scaling Y probably gives the best precision?

    let view_projection_matrix = [
        [scale_x, 0.0, 0.0, 0.0],
        [0.0, scale_y, 0.0, 0.0],
        [0.0, 0.0, -1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    view_projection_matrix.into()
}

fn sort_render_items_by_z_and_material(items: &mut [RenderItem]) {
    items.sort_by_key(|item| (item.position.z, item.material_ref.clone()));
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Rotation {
    #[default]
    Degrees0,
    Degrees90,
    Degrees180,
    Degrees270,
}

#[derive(Debug, Copy, Clone)]
pub struct SpriteParams {
    pub texture_size: UVec2,
    pub texture_pos: UVec2,
    pub scale: u8,
    pub rotation: Rotation,
    pub flip_x: bool,
    pub flip_y: bool,
    pub pivot: Vec2,
    pub color: Color,
}

impl Default for SpriteParams {
    fn default() -> Self {
        Self {
            texture_size: UVec2::new(0, 0),
            texture_pos: UVec2::new(0, 0),
            pivot: Vec2::new(0, 0),
            flip_x: false,
            flip_y: false,
            color: Color::from_octet(255, 255, 255, 255),
            scale: 1,
            rotation: Rotation::Degrees0,
        }
    }
}

pub type BindGroupRef = Arc<BindGroup>;

#[derive(Debug, PartialEq, Eq, Asset)]
pub struct Texture {
    pub texture_and_sampler_bind_group: BindGroup,
    //    pub pipeline: RenderPipelineRef,
    pub texture_size: UVec2,
}

impl Display for Texture {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.texture_size)
    }
}

impl PartialOrd<Self> for Texture {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.texture_and_sampler_bind_group
                .cmp(&other.texture_and_sampler_bind_group),
        )
    }
}

impl Ord for Texture {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.texture_and_sampler_bind_group
            .cmp(&other.texture_and_sampler_bind_group)
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq)]
pub struct MaterialBase {
    //pub pipeline: PipelineRef,
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq)]
pub struct Material {
    pub base: MaterialBase,
    pub kind: MaterialKind,
}

impl Material {
    #[inline]
    #[must_use]
    pub fn primary_texture(&self) -> Option<TextureRef> {
        self.kind.primary_texture()
    }

    #[inline]
    #[must_use]
    pub fn is_complete(&self, textures: &Assets<Texture>) -> bool {
        self.kind.is_complete(textures)
    }
}

impl Display for Material {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq)]
pub enum MaterialKind {
    NormalSprite {
        primary_texture: Id<Texture>,
    },
    AlphaMasker {
        primary_texture: Id<Texture>,
        alpha_texture: Id<Texture>,
    },
    Quad,
    LightAdd {
        primary_texture: Id<Texture>,
    },
}

impl MaterialKind {}

impl MaterialKind {
    pub fn primary_texture(&self) -> Option<Id<Texture>> {
        match &self {
            Self::NormalSprite {
                primary_texture, ..
            }
            | Self::LightAdd { primary_texture }
            | Self::AlphaMasker {
                primary_texture, ..
            } => Some(primary_texture.clone()),
            Self::Quad => None,
        }
    }

    pub(crate) fn is_complete(&self, textures: &Assets<Texture>) -> bool {
        match &self {
            Self::NormalSprite { primary_texture } | Self::LightAdd { primary_texture } => {
                textures.contains(primary_texture)
            }
            Self::AlphaMasker {
                primary_texture,
                alpha_texture,
            } => textures.contains(primary_texture) && textures.contains(alpha_texture),
            Self::Quad => true,
        }
    }
}

impl Display for MaterialKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let texture_name = self
            .primary_texture()
            .map_or_else(String::new, |x| x.to_string());

        let kind_name = match self {
            Self::NormalSprite { .. } => "NormalSprite",
            Self::LightAdd { .. } => "Light (Add)",
            Self::Quad => "Quad",
            Self::AlphaMasker { .. } => "AlphaMasker",
        };

        write!(f, "{kind_name} texture {texture_name}")
    }
}

#[derive(Debug)]
pub struct Sprite {
    pub params: SpriteParams,
}

#[derive(Debug)]
pub struct QuadColor {
    pub size: UVec2,
    pub color: Color,
}

#[derive(Debug, Copy, Clone)]
pub struct Slices {
    pub left: u16,
    pub top: u16,
    pub right: u16,  // how many pixels from the right side of the texture and going in
    pub bottom: u16, // how much to take from bottom of slice
}

#[derive(Debug)]
pub struct NineSlice {
    pub size: UVec2, // size of whole "window"
    pub slices: Slices,
    pub color: Color, // color tint
    pub origin_in_atlas: UVec2,
    pub size_inside_atlas: Option<UVec2>,
}

#[derive(Debug)]
pub struct TileMap {
    pub tiles_data_grid_size: UVec2,
    pub cell_count_size: UVec2,
    pub one_cell_size: UVec2,
    pub tiles: Vec<u16>,
    pub scale: u8,
}

#[derive(PartialEq, Debug, Eq, Ord, PartialOrd)]
pub struct Pipeline {
    name: String,
    render_pipeline: RenderPipeline,
}

impl Display for Pipeline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "pipeline: {}", self.name)
    }
}

pub type PipelineRef = Arc<Pipeline>;
