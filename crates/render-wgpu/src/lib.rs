/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod plugin;
pub mod prelude;

use int_math::{URect, UVec2, Vec2, Vec3};
use limnus_assets::prelude::{Asset, Id, RawAssetId, RawWeakId, WeakId};
use limnus_assets::Assets;
use limnus_resource::prelude::Resource;
use limnus_wgpu_math::{Matrix4, OrthoInfo, Vec4};
use mireforge_font::Font;
use mireforge_font::FontRef;
use mireforge_font::WeakFontRef;
use mireforge_render::prelude::*;
use mireforge_wgpu_sprites::{SpriteInfo, SpriteInstanceUniform};
use monotonic_time_rs::Millis;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem::swap;
use std::sync::Arc;
use tracing::trace;
use wgpu::{BindGroup, BindGroupLayout, Buffer, RenderPass, RenderPipeline};

pub type MaterialRef = Id<Material>;
pub type WeakMaterialRef = WeakId<Material>;

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
    material_ref: WeakMaterialRef,

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
}

#[derive(Resource)]
pub struct Render {
    index_buffer: Buffer,  // Only indicies for a single identity quad
    vertex_buffer: Buffer, // Only one identity quad (0,0,1,1)
    sampler: wgpu::Sampler,
    pipeline: RenderPipelineRef,
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
}

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
            material_ref: (&atlas_ref.material).into(),
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
            material_ref: (&font_and_mat.material_ref).into(),
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
        let (vertex_shader_source, fragment_shader_source) = sources();

        let sprite_info = SpriteInfo::new(
            &device,
            surface_texture_format,
            vertex_shader_source,
            fragment_shader_source,
            create_view_uniform_view_projection_matrix(physical_size),
        );

        Self {
            device,
            queue,
            items: Vec::new(),
            //   fonts: Vec::new(),
            sampler: sprite_info.sampler,
            pipeline: Arc::new(sprite_info.sprite_pipeline),
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
        }
    }

    pub fn set_now(&mut self, now: Millis) {
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

    #[inline(always)]
    fn push_sprite(&mut self, position: Vec3, material: &MaterialRef, sprite: Sprite) {
        self.items.push(RenderItem {
            position,
            material_ref: material.into(),
            renderable: Renderable::Sprite(sprite),
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
            material_ref: (&nine_slice_and_material.material_ref).into(),
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

    /*
    pub fn render_sprite(&mut self, position: Vec3, material: &MaterialRef, params: SpriteParams) {
        let atlas_rect = URect::new(0, 0, material.texture_size().x, material.texture_size().y);

        self.push_sprite(position, material, Sprite { atlas_rect, params });
    }*/

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
        self.items.push(RenderItem {
            position,
            material_ref: WeakId::<Material>::new(RawWeakId::with_asset_type::<Material>(
                RawAssetId::new(0, 0),
                "nothing".into(),
            )),
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
            material_ref: material_ref.into(),
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
        let mut current_material: Option<&WeakMaterialRef> = None;

        for render_item in &self.items {
            if Some(&render_item.material_ref) != current_material {
                if !current_batch.is_empty() {
                    material_batches.push(current_batch.clone());
                    current_batch.clear();
                }
                current_material = Some(&render_item.material_ref);
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
            true,
        )
    }

    /// # Panics
    ///
    #[allow(clippy::too_many_lines)]
    pub fn prepare_render(&mut self, materials: &Assets<Material>, fonts: &Assets<Font>) {
        const FLIP_X_MASK: u32 = 0b0000_0100;
        const FLIP_Y_MASK: u32 = 0b0000_1000;

        sort_render_items_by_z_and_material(&mut self.items);

        let batches = self.order_render_items_in_batches();

        let mut quad_matrix_and_uv: Vec<SpriteInstanceUniform> = Vec::new();
        let mut batch_vertex_ranges: Vec<(WeakMaterialRef, u32, u32)> = Vec::new();

        for render_items in &batches {
            let quad_len_before = quad_matrix_and_uv.len() as u32;

            // Fix: Access material_ref through reference and copy it
            let weak_material_ref = render_items
                .first()
                .map(|item| {
                    // Force copy semantics by dereferencing the shared reference
                    let material_ref: WeakId<Material> = item.material_ref;
                    material_ref
                })
                .expect("Render items batch was empty");

            let result = materials.get_weak(weak_material_ref);
            if result.is_none() {
                // Material is not loaded yet
                continue;
            }
            let material = result.unwrap();
            let current_texture_size = material.texture_size;

            for render_item in render_items {
                match &render_item.renderable {
                    Renderable::Sprite(sprite) => {
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
                            true,
                        );
                        quad_matrix_and_uv.push(quad_instance);
                    }

                    Renderable::NineSlice(nine_slice) => {
                        Self::prepare_nine_slice(
                            nine_slice,
                            render_item.position,
                            &mut quad_matrix_and_uv,
                            current_texture_size,
                        );
                    }

                    Renderable::QuadColor(quad) => {
                        /*
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
                         */

                        let model_matrix =
                            Matrix4::from_translation(
                                render_item.position.x as f32,
                                render_item.position.y as f32,
                                0.0,
                            ) * Matrix4::from_scale(quad.size.x as f32, quad.size.y as f32, 1.0);

                        /*
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

                         */

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
                            false,
                        );
                        quad_matrix_and_uv.push(quad_instance);
                    }

                    Renderable::Text(text) => {
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
                                true,
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

                            let cell_tex_coords_mul_add = Self::calculate_texture_coords_mul_add(
                                cell_texture_area,
                                current_texture_size,
                            );

                            let quad_instance = SpriteInstanceUniform::new(
                                cell_model_matrix,
                                cell_tex_coords_mul_add,
                                0,
                                Vec4([1.0, 1.0, 1.0, 1.0]),
                                true,
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
    pub fn render(
        &mut self,
        render_pass: &mut RenderPass,
        materials: &Assets<Material>,
        fonts: &Assets<Font>,
        now: Millis,
    ) {
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

        self.prepare_render(materials, fonts);

        render_pass.set_viewport(
            self.viewport.position.x as f32,
            self.viewport.position.y as f32,
            self.viewport.size.x as f32,
            self.viewport.size.y as f32,
            0.0,
            1.0,
        );

        render_pass.set_pipeline(&self.pipeline);

        // Index and vertex buffers never change
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        // Vertex buffer is reused
        render_pass.set_vertex_buffer(1, self.quad_matrix_and_uv_instance_buffer.slice(..));

        // Camera is the same for everything
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        let num_indices = mireforge_wgpu_sprites::INDICES.len() as u32;

        for &(weak_material_ref, start, count) in &self.batch_offsets {
            let wgpu_material = materials
                .get_weak(weak_material_ref)
                .expect("no such material");

            // Bind the texture and sampler bind group (Bind Group 1)
            render_pass.set_bind_group(1, &wgpu_material.texture_and_sampler_bind_group, &[]);

            // Issue the instanced draw call for the batch
            trace!(material=%weak_material_ref, start=%start, count=%count, "draw instanced");
            render_pass.draw_indexed(0..num_indices, 0, start..(start + count));
        }

        self.items.clear();
    }

    pub fn material_from_texture(&self, texture: wgpu::Texture, label: &str) -> Material {
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

        Material {
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
        near: 1.0,
        far: -1.0,
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
    items.sort_by_key(|item| (item.position.z, item.material_ref));
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

#[derive(Debug, PartialEq, Eq, Asset)]
pub struct Material {
    pub texture_and_sampler_bind_group: BindGroup,
//    pub pipeline: RenderPipelineRef,
    pub texture_size: UVec2,
}

impl PartialOrd<Self> for Material {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.texture_and_sampler_bind_group
                .cmp(&other.texture_and_sampler_bind_group),
        )
    }
}

impl Ord for Material {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.texture_and_sampler_bind_group
            .cmp(&other.texture_and_sampler_bind_group)
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

pub type RenderPipelineRef = Arc<RenderPipeline>;

const fn sources() -> (&'static str, &'static str) {
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
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) use_texture: u32,
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
    @location(9) use_texture: u32,
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
    output.use_texture = use_texture;

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
    @location(2) use_texture: u32,
};

// Fragment shader entry point
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var final_color: vec4<f32>;

    // Sample the texture using the texture coordinates
    let texture_color = textureSample(diffuse_texture, sampler_diffuse, input.tex_coords);
    if (input.use_texture == 1u) { // Check if use_texture is true (1)
        // Apply color modulation and opacity
        final_color = texture_color * input.color;
    } else {
        final_color = input.color;
    }

    return final_color;
}

";
    (vertex_shader_source, fragment_shader_source)
}

#[allow(unused)]
const fn masked_texture_tinted() -> &'static str {
    r"
// Masked Texture and tinted shader


@group(1) @binding(0) var t_color: texture_2d<f32>;
@group(1) @binding(1) var t_mask: texture_2d<f32>;
@group(1) @binding(2) var s_sampler: sampler;
// New uniform binding for mask parameters
@group(1) @binding(3) var<uniform> mask_params: MaskParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>, // Original UVs from vertex
    @location(1) color: vec4<f32>,
};

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color_sample = textureSample(t_color, s_sampler, input.tex_coords);

    let mask_coords = input.tex_coords + mask_params.offset;

    // TODO: Scale rot and other // let mask_coords = (input.tex_coords * mask_params.scale) + mask_params.offset;

    let mask_alpha = textureSample(t_mask, s_sampler, mask_coords).r;

    let final_rgb = color_sample.rgb * input.color.rgb;
    let final_alpha = mask_alpha * input.color.a;

    return vec4<f32>(final_rgb, final_alpha);
}
    "
}
