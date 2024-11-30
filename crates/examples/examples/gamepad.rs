/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
extern crate core;

use swamp::prelude::*;

const TILE_SIZE: UVec2 = UVec2::new(128, 128);
const CHARACTER_HEIGHT: u16 = 36;
const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(320, 240);
const FACTOR: u16 = 5;
const START_WINDOW_SIZE: UVec2 = UVec2::new(
    FACTOR * VIRTUAL_SCREEN_SIZE.x,
    FACTOR * VIRTUAL_SCREEN_SIZE.y,
);

const JOTEM_SIZE: UVec2 = UVec2::new(1536, 1536);

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Action {
    Idle,
    Attacking,
}

#[derive(Debug)]
pub struct ExampleLogic {
    tick_count: usize,
    tick_count_since_attack: usize,

    character_x: Fp,
    character_x_velocity: Fp,
    character_action: Action,
    attacking_ticks: u16,
    attack_id: u8,
}

const ATTACK_START_FRAME: u16 = 12 * 7;

impl ApplicationLogic for ExampleLogic {
    fn new() -> Self {
        Self {
            tick_count: 0,
            tick_count_since_attack: 0,
            character_x: ((VIRTUAL_SCREEN_SIZE.x / 2u16 - (TILE_SIZE.x / 2u16) - 64) as i16).into(),
            character_x_velocity: Fp::zero(),
            character_action: Action::Idle,
            attacking_ticks: 0,
            attack_id: 0,
        }
    }

    fn tick(&mut self) {
        self.tick_count += 1;

        self.character_x += self.character_x_velocity;

        match self.character_action {
            Action::Attacking => {
                self.attacking_ticks += 1;
                if self.attacking_ticks > 10 {
                    self.attacking_ticks = 0;
                    self.character_action = Action::Idle;
                }
            }
            Action::Idle => {}
        }
    }

    fn gamepad_button_changed(&mut self, _gamepad: &Gamepad, button: Button, value: Fp) {
        if value > Fp::from(0.1) {
            if let Button::South = button {
                if self.character_action == Action::Idle {
                    self.tick_count_since_attack = 0;
                    self.attacking_ticks = 0;
                    self.character_action = Action::Attacking;
                    self.attack_id += 1;
                }
            }
        }
    }

    fn gamepad_axis_changed(&mut self, _gamepad: &Gamepad, axis: Axis, value: Fp) {
        if let Axis::LeftStickX = axis {
            self.character_x_velocity = value * 3;
        }
    }
}

// ----------------- Render -------------------------

#[derive(Debug)]
pub struct ExampleRender {
    old_hero_atlas: FixedAtlas,
    attack_anim: FrameAnimation,
    played_attack_id: u8,
    sleep_anim: FrameAnimation,
    bat_anim: FrameAnimation,
    bat_atlas: Option<FixedAtlas>, // Intentionally unload this later
}

impl ApplicationRender<ExampleLogic> for ExampleRender {
    fn new(assets: &mut impl Assets) -> Self {
        let now = assets.now();

        let old_hero_atlas =
            assets.frame_fixed_grid_material_png("jotem_spritesheet", TILE_SIZE, JOTEM_SIZE);

        let attack_anim_cfg = FrameAnimationConfig::new(ATTACK_START_FRAME, 10, 19);

        let sleep_anim_config = FrameAnimationConfig::new(12 * 11 + 1, 5, 5);
        let mut sleep_anim = FrameAnimation::new(sleep_anim_config);
        sleep_anim.play_repeat(now);

        let bat_atlas =
            assets.frame_fixed_grid_material_png("flying_46x30", (46, 30).into(), (322, 30).into());

        let bat_anim_config = FrameAnimationConfig::new(0, 7, 20);
        let mut bat_anim = FrameAnimation::new(bat_anim_config);
        bat_anim.play_repeat(now);

        Self {
            old_hero_atlas,
            bat_atlas: Some(bat_atlas),
            attack_anim: FrameAnimation::new(attack_anim_cfg),
            sleep_anim,
            bat_anim,
            played_attack_id: 0,
        }
    }

    fn render(&mut self, gfx: &mut impl Gfx, state: &ExampleLogic) {
        let now = gfx.now();

        if state.character_action == Action::Attacking && self.played_attack_id != state.attack_id {
            self.played_attack_id = state.attack_id;
            self.attack_anim.play(now);
        }

        self.attack_anim.update(now);
        self.sleep_anim.update(now);
        self.bat_anim.update(now);

        gfx.set_clear_color(Color::from_octet(0, 0, 0, 0));

        gfx.set_origin((0, (VIRTUAL_SCREEN_SIZE.y / 2 - CHARACTER_HEIGHT) as i16).into());

        if let Some(ref bat_atlas) = &self.bat_atlas {
            gfx.sprite_atlas_frame(
                (
                    (VIRTUAL_SCREEN_SIZE.x / 2u16 - 23) as i16,
                    (VIRTUAL_SCREEN_SIZE.y / 2u16 + CHARACTER_HEIGHT + 10) as i16,
                    1,
                )
                    .into(),
                self.bat_anim.frame(),
                bat_atlas,
            );
        }

        let spacing = 64;

        gfx.sprite_atlas_frame(
            (
                state.character_x.into(),
                (VIRTUAL_SCREEN_SIZE.y / 2u16 - CHARACTER_HEIGHT) as i16,
                0,
            )
                .into(),
            self.attack_anim.frame(),
            &self.old_hero_atlas,
        );

        gfx.sprite_atlas_frame(
            (
                (VIRTUAL_SCREEN_SIZE.x / 2u16 - (TILE_SIZE.x / 2u16) + spacing) as i16,
                (VIRTUAL_SCREEN_SIZE.y / 2u16 - CHARACTER_HEIGHT) as i16,
                0,
            )
                .into(),
            self.sleep_anim.frame(),
            &self.old_hero_atlas,
        );
    }
}

// ----------------- AUDIO -------------------------

#[derive(Debug)]
pub struct ExampleAudio {
    whoosh_sound: StereoSampleRef,
    attack_sound: Option<SoundHandle>,
    attack_sound_ticks: u16, // TODO: Check if sound is finished playing.
    attack_sound_played_id: u8,
}

impl ApplicationAudio<ExampleLogic> for ExampleAudio {
    fn new(assets: &mut impl Assets) -> Self {
        let now = assets.now();

        let whoosh_sound = assets.audio_sample_wav("qubodup_whoosh");

        Self {
            whoosh_sound,
            attack_sound: None,
            attack_sound_ticks: 0,
            attack_sound_played_id: 0,
        }
    }

    fn audio(&mut self, audio: &mut impl Audio, state: &ExampleLogic) {
        if state.character_action == Action::Attacking {
            if self.attack_sound_played_id != state.attack_id {
                self.attack_sound = Some(audio.play(&self.whoosh_sound));
                self.attack_sound_ticks = 0;
                self.attack_sound_played_id = state.attack_id;
            } else {
                self.attack_sound_ticks += 1;
                if self.attack_sound_ticks > 20 {
                    // TODO: HACK: check if sound is finished playing
                    self.attack_sound = None;
                }
            }
        } else {
            self.attack_sound = None;
        }
    }
}

fn main() {
    run_advanced::<ExampleLogic, ExampleRender, ExampleAudio>(
        "Gamepad Example",
        VIRTUAL_SCREEN_SIZE,
        START_WINDOW_SIZE,
    );
}
