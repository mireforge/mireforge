/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
extern crate core;

pub mod prelude;

use int_math::{URect, UVec2, Vec2};

use fixed32::Fp;
use limnus_app::prelude::{App, AppReturnValue, ApplicationExit, Plugin};
use limnus_audio_mixer::{AudioMixer, StereoSample};
use limnus_basic_input::InputMessage;
use limnus_basic_input::prelude::{
    ButtonState, KeyCode, MouseButton, MouseScrollDelta, TouchPhase,
};
use limnus_default_stages::{FixedUpdate, RenderUpdate, Update};
use limnus_gamepad::{Axis, Button, GamePadId, Gamepad, GamepadMessage, Gamepads};
use limnus_local_resource::prelude::LocalResource;
use limnus_message::MessagesIterator;
use limnus_resource::ResourceStorage;
use limnus_resource::prelude::Resource;
use limnus_screen::WindowMessage;
use limnus_system_params::{LoReM, Msg, Re, ReAll, ReM};
use mireforge_game_assets::{Assets, GameAssets};
use mireforge_game_audio::{Audio, GameAudio};
use mireforge_render_wgpu::prelude::{Gfx, Render};
use monotonic_time_rs::{InstantMonotonicClock, Millis, MonotonicClock};
use std::cmp::{max, min};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use tracing::debug;

pub trait Application: Sized + 'static {
    fn new(assets: &mut impl Assets) -> Self;
    fn tick(&mut self, assets: &mut impl Assets);
    fn render(&mut self, gfx: &mut impl Gfx);
    fn audio(&mut self, _audio: &mut impl Audio) {}

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

    fn touch(&mut self, _position: UVec2, _touch_phase: &TouchPhase) {}

    fn mouse_input(&mut self, _state: ButtonState, _button: MouseButton) {}

    fn mouse_wheel(&mut self, _delta_y: i16) {}

    fn mouse_motion(&mut self, _delta: Vec2) {}

    fn gamepad_activated(&mut self, _gamepad_id: GamePadId, _name: String) {}
    fn gamepad_button_changed(&mut self, _gamepad: &Gamepad, _button: Button, _value: Fp) {}
    fn gamepad_axis_changed(&mut self, _gamepad: &Gamepad, _axis: Axis, _value: Fp) {}
    fn gamepad_disconnected(&mut self, _gamepad_id: GamePadId) {}

    fn scale_factor_changed(&mut self, _scale_factor: f64) -> Option<UVec2> {
        None
    }
}

#[derive(Debug, Resource)]
pub struct GameSettings {
    pub virtual_size: UVec2,
}

#[derive(LocalResource)]
pub struct Game<G: Application> {
    game: G,
    clock: InstantMonotonicClock,
}

impl<G: Application> Debug for Game<G> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "WgpuGame")
    }
}

impl<G: Application> Game<G> {
    #[must_use]
    pub fn new(all_resources: &mut ResourceStorage) -> Self {
        let clock = InstantMonotonicClock::new();
        let mut assets = GameAssets::new(all_resources, clock.now());
        let game = G::new(&mut assets);

        Self { game, clock }
    }

    pub fn inputs(&mut self, iter: MessagesIterator<InputMessage>) {
        for message in iter {
            match message {
                InputMessage::KeyboardInput(button_state, key_code) => {
                    self.game.keyboard_input(*button_state, *key_code);
                }
                InputMessage::MouseInput(button_state, button) => {
                    self.game.mouse_input(*button_state, *button);
                }
                InputMessage::MouseWheel(scroll_delta, _touch_phase) => {
                    if let MouseScrollDelta::LineDelta(delta) = scroll_delta {
                        let game_scroll_y = (f32::from(-delta.y) * 120.0) as i16;
                        self.game.mouse_wheel(game_scroll_y);
                    }
                }
            }
        }
    }

    #[must_use] pub fn virtual_position_from_physical(
        physical_position: UVec2,
        viewport: URect,
        virtual_surface_size: UVec2,
    ) -> UVec2 {
        let relative_x = max(
            0,
            min(
                i64::from(physical_position.x) - i64::from(viewport.position.x),
                i64::from(viewport.size.x - 1),
            ),
        );

        let relative_y = max(
            0,
            min(
                i64::from(physical_position.y) - i64::from(viewport.position.y),
                i64::from(viewport.size.y - 1),
            ),
        );

        let clamped_to_viewport: UVec2 = UVec2::new(relative_x as u16, relative_y as u16);

        let virtual_position_x =
            (u64::from(clamped_to_viewport.x) * u64::from(virtual_surface_size.x)) / u64::from(viewport.size.x);

        let virtual_position_y =
            (u64::from(clamped_to_viewport.y) * u64::from(virtual_surface_size.y)) / u64::from(viewport.size.y);

        UVec2::new(virtual_position_x as u16, virtual_position_y as u16)
    }

    pub fn cursor_moved(
        &mut self,
        physical_position: UVec2,
        viewport: URect,
        virtual_surface_size: UVec2,
    ) {
        let virtual_position =
            Self::virtual_position_from_physical(physical_position, viewport, virtual_surface_size);
        self.game.cursor_moved(virtual_position);
    }

    pub fn touch(
        &mut self,
        physical_position: UVec2,
        touch_phase: &TouchPhase,
        viewport: URect,
        virtual_surface_size: UVec2,
    ) {
        let virtual_position =
            Self::virtual_position_from_physical(physical_position, viewport, virtual_surface_size);
        self.game.touch(virtual_position, touch_phase);
    }

    pub fn mouse_move(&mut self, iter: MessagesIterator<WindowMessage>, wgpu_render: &Render) {
        for message in iter {
            match message {
                WindowMessage::CursorMoved(position) => self.cursor_moved(
                    *position,
                    wgpu_render.viewport(),
                    wgpu_render.virtual_surface_size_with_scaling(),
                ),
                WindowMessage::Touch(position, touch_phase) => self.touch(
                    *position,
                    touch_phase,
                    wgpu_render.viewport(),
                    wgpu_render.virtual_surface_size_with_scaling(),
                ),
                WindowMessage::WindowCreated() => {}
                WindowMessage::Resized(_) => {}
            }
        }
    }

    pub fn tick(&mut self, storage: &mut ResourceStorage, now: Millis) {
        // This is a quick operation, we basically wrap storage
        let mut assets = GameAssets::new(storage, now);

        self.game.tick(&mut assets);
    }

    pub fn render(&mut self, wgpu_render: &mut Render, now: Millis) {
        wgpu_render.set_now(now);
        self.game.render(wgpu_render);
    }
}

pub struct GamePlugin<G: Application> {
    pub phantom_data: PhantomData<G>,
}
impl<G: Application> Default for GamePlugin<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Application> GamePlugin<G> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

pub fn mouse_input_tick<G: Application>(
    mut internal_game: LoReM<Game<G>>,
    window_messages: Msg<WindowMessage>,
    wgpu_render: Re<Render>,
) {
    internal_game.mouse_move(window_messages.iter_previous(), &wgpu_render);
}

pub fn keyboard_input_tick<G: Application>(
    mut internal_game: LoReM<Game<G>>,
    input_messages: Msg<InputMessage>,
) {
    internal_game.inputs(input_messages.iter_previous());
}

pub fn audio_tick<G: Application>(
    mut internal_game: LoReM<Game<G>>,
    stereo_samples: Re<limnus_assets::Assets<StereoSample>>,
    mut audio_mixer: LoReM<AudioMixer>,
) {
    let mut game_audio = GameAudio::new(&mut audio_mixer, &stereo_samples);
    internal_game.game.audio(&mut game_audio);
}

pub fn logic_tick<G: Application>(mut internal_game: LoReM<Game<G>>, mut all_resources: ReAll) {
    let now = internal_game.clock.now();

    internal_game.tick(&mut all_resources, now);
    if internal_game.game.wants_to_quit() {
        all_resources.insert(ApplicationExit {
            value: AppReturnValue::Value(0),
        });
    }
}

pub fn render_tick<G: Application>(
    mut internal_game: LoReM<Game<G>>,
    mut wgpu_render: ReM<Render>,
) {
    let now = internal_game.clock.now();

    internal_game.render(&mut wgpu_render, now);
}

pub fn gamepad_input_tick<G: Application>(
    mut internal_game: LoReM<Game<G>>,
    gamepads: Re<Gamepads>,
    gamepad_messages: Msg<GamepadMessage>,
) {
    for gamepad_message in gamepad_messages.iter_current() {
        match gamepad_message {
            GamepadMessage::Connected(_gamepad_id, _gamepad_name) => {}
            GamepadMessage::Disconnected(gamepad_id) => {
                if let Some(gamepad) = gamepads.gamepad(*gamepad_id)
                    && gamepad.is_active {
                        internal_game.game.gamepad_disconnected(*gamepad_id);
                    }
            }
            GamepadMessage::Activated(gamepad_id) => {
                if let Some(gamepad) = gamepads.gamepad(*gamepad_id) {
                    internal_game
                        .game
                        .gamepad_activated(*gamepad_id, gamepad.name.as_str().to_string());
                }
            }
            GamepadMessage::ButtonChanged(gamepad_id, button, value) => {
                if let Some(gamepad) = gamepads.gamepad(*gamepad_id)
                    && gamepad.is_active {
                        internal_game.game.gamepad_button_changed(
                            gamepad,
                            *button,
                            Fp::from(*value),
                        );
                    }
            }
            GamepadMessage::AxisChanged(gamepad_id, axis, value) => {
                if let Some(gamepad) = gamepads.gamepad(*gamepad_id)
                    && gamepad.is_active {
                        internal_game
                            .game
                            .gamepad_axis_changed(gamepad, *axis, Fp::from(*value));
                    }
            }
        }
    }
}

impl<G: Application> Plugin for GamePlugin<G> {
    fn post_initialization(&self, app: &mut App) {
        debug!("calling WgpuGame::new()");

        let all_resources = app.resources_mut();
        let internal_game = Game::<G>::new(all_resources);
        app.insert_local_resource(internal_game);

        app.add_system(Update, gamepad_input_tick::<G>);
        app.add_system(Update, keyboard_input_tick::<G>);
        app.add_system(Update, mouse_input_tick::<G>);
        app.add_system(Update, audio_tick::<G>);
        app.add_system(FixedUpdate, logic_tick::<G>);
        app.add_system(RenderUpdate, render_tick::<G>);
    }
}
