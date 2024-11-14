/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use crate::prelude::Glyph;
use int_math::{UVec2, Vec2};
use monotonic_time_rs::Millis;
use swamp_assets::prelude::{AssetName, Id};
use swamp_basic_input::prelude::*;
use swamp_render_wgpu::prelude::Font;
use swamp_render_wgpu::{FixedAtlas, FontAndMaterial, Gfx, MaterialRef};

pub trait Application: Send + Sync + Sized + 'static {
    fn new(assets: &mut impl Assets) -> Self;
    fn tick(&mut self, assets: &mut impl Assets);
    fn render(&mut self, gfx: &mut impl Gfx);

    fn wants_to_quit(&self) -> bool {
        false
    }

    fn wants_cursor_visible(&self) -> bool {
        true
    }

    fn keyboard_input(&mut self, _state: ButtonState, _key_code: KeyCode) {}

    fn cursor_entered(&mut self) {}

    fn cursor_left(&mut self) {}

    fn cursor_moved(&mut self, _position: UVec2) {}

    fn mouse_input(&mut self, _state: ButtonState, _button: MouseButton) {}

    fn mouse_wheel(&mut self, _delta_y: i16) {}

    fn mouse_motion(&mut self, _delta: Vec2) {}

    fn scale_factor_changed(&mut self, _scale_factor: f64) -> Option<UVec2> {
        None
    }
}

pub trait Assets {
    fn material_png(&mut self, name: impl Into<AssetName>) -> MaterialRef;
    fn frame_fixed_grid_material_png(
        &mut self,
        name: impl Into<AssetName>,
        grid_size: UVec2,
        texture_size: UVec2,
    ) -> FixedAtlas;
    fn bm_font(&mut self, name: impl Into<AssetName>) -> FontAndMaterial;

    fn font(&self, font_ref: &Id<Font>) -> Option<&Font>;
    fn text_glyphs(&self, text: &str, font_ref: &FontAndMaterial) -> Option<Vec<Glyph>>;

    fn now(&self) -> Millis;
}

/*
pub trait Gfx {
    // Physical surface and viewport
    fn physical_aspect_ratio(&self) -> AspectRatio;
    fn physical_size(&self) -> UVec2;
    fn set_viewport(&mut self, viewport_strategy: ViewportStrategy);
    fn viewport(&self) -> &ViewportStrategy;

    // "Camera" (Project and view matrix)
    fn set_scale(&mut self, scale_factor: VirtualScale);
    fn set_origin(&mut self, position: Vec2);

    // Other
    fn set_clear_color(&mut self, color: Color);

    // Sprite
    fn sprite_atlas_frame(&mut self, position: Vec3, frame: u16, atlas: &impl FrameLookup);
    fn sprite_atlas(&mut self, position: Vec3, atlas_rect: URect, material: &MaterialRef);

    // Text
    fn text_draw(&mut self, position: Vec3, text: &str, font_ref: &FontAndMaterialRef);
    fn text_glyphs(&self, position: Vec2, text: &str, font_ref: &FontAndMaterialRef) -> Vec<Glyph>;

    // Tilemap
    fn tilemap(&mut self, position: Vec3, tiles: &[u16], width: u16, atlas: &FixedAtlas);
    fn tilemap_params(
        &mut self,
        position: Vec3,
        tiles: &[u16],
        width: u16,
        atlas: &FixedAtlas,
        scale: u8,
    );
    fn now(&self) -> Millis;
}
*/
