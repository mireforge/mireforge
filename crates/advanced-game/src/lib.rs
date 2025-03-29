/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
extern crate core;

pub mod audio;
pub mod logic;
pub mod render;

use fixed32::Fp;
use int_math::{UVec2, Vec2};
use limnus_basic_input::prelude::{ButtonState, KeyCode, MouseButton};
use limnus_gamepad::{Axis, Button, GamePadId, Gamepad};
use mireforge_game_assets::Assets;
use mireforge_game_audio::Audio;
use mireforge_render_wgpu::Gfx;

pub trait ApplicationLogic: Sized + 'static {
    fn new() -> Self;

    fn tick(&mut self);

    fn wants_to_quit(&self) -> bool {
        false
    }

    fn keyboard_input(&mut self, _state: ButtonState, _key_code: KeyCode) {}

    fn cursor_entered(&mut self) {}

    fn cursor_left(&mut self) {}

    fn cursor_moved(&mut self, _position: UVec2) {}

    fn mouse_input(&mut self, _state: ButtonState, _button: MouseButton) {}

    fn mouse_wheel(&mut self, _delta_y: i16) {}

    fn mouse_motion(&mut self, _delta: Vec2) {}

    fn gamepad_activated(&mut self, _gamepad_id: GamePadId, _name: String) {}
    fn gamepad_button_changed(&mut self, _gamepad: &Gamepad, _button: Button, _value: Fp) {}
    fn gamepad_axis_changed(&mut self, _gamepad: &Gamepad, _axis: Axis, _value: Fp) {}
    fn gamepad_disconnected(&mut self, _gamepad_id: GamePadId) {}
}

pub trait ApplicationAudio<L: ApplicationLogic>: Sized + 'static {
    fn new(assets: &mut impl Assets) -> Self;
    fn audio(&mut self, _audio: &mut impl Audio, _logic: &L) {}
}

pub trait ApplicationRender<L: ApplicationLogic>: Sized + 'static {
    fn new(assets: &mut impl Assets) -> Self;
    fn render(&mut self, gfx: &mut impl Gfx, logic: &L);

    fn wants_cursor_visible(&self) -> bool {
        true
    }
    fn scale_factor_changed(&mut self, _scale_factor: f64) -> Option<UVec2> {
        None
    }
}
