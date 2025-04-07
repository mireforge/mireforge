use crate::gfx::Gfx;
use crate::{
    FixedAtlas, FontAndMaterial, FrameLookup, MaterialRef, NineSliceAndMaterial, Render,
    RenderItem, Renderable, SpriteParams, Text, TileMap, to_wgpu_color,
};
use int_math::{URect, UVec2, Vec2, Vec3};
use mireforge_render::{AspectRatio, Color, ViewportStrategy, VirtualScale};
use monotonic_time_rs::Millis;

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

    fn set_virtual_size(&mut self, virtual_size: UVec2) {
        self.resize_virtual(virtual_size);
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
