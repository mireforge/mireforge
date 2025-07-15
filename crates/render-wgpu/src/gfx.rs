use crate::{
    FixedAtlas, FontAndMaterial, FrameLookup, MaterialRef, NineSliceAndMaterial, QuadParams,
    SpriteParams,
};
use int_math::{URect, UVec2, Vec2, Vec3};
use mireforge_render::{AspectRatio, Color, ViewportStrategy, VirtualScale};
use monotonic_time_rs::Millis;

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

    fn set_virtual_size(&mut self, virtual_size: UVec2);
    fn quad_ex(&mut self, position: Vec3, size: UVec2, color: Color, params: QuadParams);
}
