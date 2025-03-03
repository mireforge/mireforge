/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use crate::ApplicationLogic;
use fixed32::Fp;
use int_math::{URect, UVec2};
use limnus_app::prelude::{App, AppReturnValue, ApplicationExit, Plugin};
use limnus_basic_input::InputMessage;
use limnus_basic_input::prelude::MouseScrollDelta;
use limnus_default_stages::{FixedUpdate, Update};
use limnus_gamepad::{GamepadMessage, Gamepads};
use limnus_local_resource::prelude::LocalResource;
use limnus_message::MessagesIterator;
use limnus_screen::WindowMessage;
use limnus_system_params::{LoReM, Msg, Re, ReAll};
use std::cmp::{max, min};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use swamp_render_wgpu::Render;
use tracing::trace;

#[derive(LocalResource, Default)]
pub struct GameLogic<L: ApplicationLogic> {
    pub logic: L,
}

impl<L: ApplicationLogic> GameLogic<L> {
    #[must_use]
    pub fn new() -> Self {
        Self { logic: L::new() }
    }
    pub fn inputs(&mut self, iter: MessagesIterator<InputMessage>) {
        for message in iter {
            match message {
                InputMessage::KeyboardInput(button_state, key_code) => {
                    self.logic.keyboard_input(*button_state, *key_code);
                }
                InputMessage::MouseInput(button_state, button) => {
                    self.logic.mouse_input(*button_state, *button);
                }
                InputMessage::MouseWheel(scroll_delta, _touch_phase) => {
                    if let MouseScrollDelta::LineDelta(delta) = scroll_delta {
                        let game_scroll_y = (-delta.y as f32 * 120.0) as i16;
                        self.logic.mouse_wheel(game_scroll_y);
                    }
                }
            }
        }
    }

    pub fn cursor_moved(
        &mut self,
        physical_position: UVec2,
        viewport: URect,
        virtual_surface_size: UVec2,
    ) {
        let relative_x = max(
            0,
            min(
                physical_position.x as i64 - viewport.position.x as i64,
                (viewport.size.x - 1) as i64,
            ),
        );

        let relative_y = max(
            0,
            min(
                physical_position.y as i64 - viewport.position.y as i64,
                (viewport.size.y - 1) as i64,
            ),
        );

        let clamped_to_viewport: UVec2 = UVec2::new(relative_x as u16, relative_y as u16);

        let virtual_position_x =
            (clamped_to_viewport.x as u64 * virtual_surface_size.x as u64) / viewport.size.x as u64;

        let virtual_position_y =
            (clamped_to_viewport.y as u64 * virtual_surface_size.y as u64) / viewport.size.y as u64;

        let virtual_position = UVec2::new(virtual_position_x as u16, virtual_position_y as u16);
        self.logic.cursor_moved(virtual_position);
    }

    pub fn mouse_move(&mut self, iter: MessagesIterator<WindowMessage>, wgpu_render: &Render) {
        for message in iter {
            match message {
                WindowMessage::CursorMoved(position) => self.cursor_moved(
                    *position,
                    wgpu_render.viewport(),
                    wgpu_render.virtual_surface_size(),
                ),
                WindowMessage::WindowCreated() => {}
                WindowMessage::Resized(_) => {}
            }
        }
    }
}

impl<L: ApplicationLogic> Debug for GameLogic<L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameAudioRender")
    }
}

pub fn advanced_game_logic_tick<L: ApplicationLogic>(
    mut all_resources: ReAll,
    mut game_logic: LoReM<GameLogic<L>>,
) {
    game_logic.logic.tick();
    if game_logic.logic.wants_to_quit() {
        all_resources.insert(ApplicationExit {
            value: AppReturnValue::Value(0),
        });
    }
}

pub fn advanced_game_logic_keyboard_tick<L: ApplicationLogic>(
    input_messages: Msg<InputMessage>,
    mut game_logic: LoReM<GameLogic<L>>,
) {
    game_logic.inputs(input_messages.iter_previous());
}

pub fn advanced_game_logic_mouse_tick<L: ApplicationLogic>(
    wgpu_render: Re<Render>,
    window_messages: Msg<WindowMessage>,
    mut game_logic: LoReM<GameLogic<L>>,
) {
    game_logic.mouse_move(window_messages.iter_previous(), &wgpu_render);
}

pub fn advanced_gamepad_input_tick<L: ApplicationLogic>(
    mut internal_game: LoReM<GameLogic<L>>,
    gamepads: Re<Gamepads>,
    gamepad_messages: Msg<GamepadMessage>,
) {
    for gamepad_message in gamepad_messages.iter_current() {
        match gamepad_message {
            GamepadMessage::Connected(_gamepad_id, _gamepad_name) => {}
            GamepadMessage::Disconnected(gamepad_id) => {
                if let Some(gamepad) = gamepads.gamepad(*gamepad_id) {
                    if gamepad.is_active {
                        internal_game.logic.gamepad_disconnected(*gamepad_id);
                    }
                }
            }
            GamepadMessage::Activated(gamepad_id) => {
                if let Some(gamepad) = gamepads.gamepad(*gamepad_id) {
                    internal_game
                        .logic
                        .gamepad_activated(*gamepad_id, gamepad.name.as_str().to_string());
                }
            }
            GamepadMessage::ButtonChanged(gamepad_id, button, value) => {
                if let Some(gamepad) = gamepads.gamepad(*gamepad_id) {
                    if gamepad.is_active {
                        internal_game.logic.gamepad_button_changed(
                            gamepad,
                            *button,
                            Fp::from(*value),
                        );
                    }
                }
            }
            GamepadMessage::AxisChanged(gamepad_id, axis, value) => {
                if let Some(gamepad) = gamepads.gamepad(*gamepad_id) {
                    if gamepad.is_active {
                        internal_game
                            .logic
                            .gamepad_axis_changed(gamepad, *axis, Fp::from(*value));
                    }
                }
            }
        }
    }
}

// ------ Plugin

#[derive(Debug, Default)]
pub struct GameLogicPlugin<L: ApplicationLogic> {
    _phantom: PhantomData<L>,
}

impl<L: ApplicationLogic> GameLogicPlugin<L> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<L: ApplicationLogic> Plugin for GameLogicPlugin<L> {
    fn post_initialization(&self, app: &mut App) {
        trace!("GameLogicPlugin startup");
        let game_logic = GameLogic::<L>::new();
        app.insert_local_resource(game_logic);

        app.add_system(Update, advanced_gamepad_input_tick::<L>);
        app.add_system(Update, advanced_game_logic_mouse_tick::<L>);
        app.add_system(Update, advanced_game_logic_keyboard_tick::<L>);
        app.add_system(FixedUpdate, advanced_game_logic_tick::<L>);
    }
}
