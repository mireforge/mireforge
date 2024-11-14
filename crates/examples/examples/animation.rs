/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use swamp_boot_game::prelude::*;

const TILE_SIZE: UVec2 = UVec2::new(128, 128);
const CHARACTER_HEIGHT: u16 = 36;
const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(320, 240);
const FACTOR: u16 = 5;
const START_WINDOW_SIZE: UVec2 = UVec2::new(
    FACTOR * VIRTUAL_SCREEN_SIZE.x,
    FACTOR * VIRTUAL_SCREEN_SIZE.y,
);

const JOTEM_SIZE: UVec2 = UVec2::new(1536, 1536);

#[derive(Debug)]
pub struct AnimationExample {
    old_hero_atlas: FixedAtlas,
    attack_anim: FrameAnimation,
    sleep_anim: FrameAnimation,
    bat_anim: FrameAnimation,
    bat_atlas: Option<FixedAtlas>, // Intentionally unload this later
    tick_count: usize,
}

impl AnimationExample {}

impl Application for AnimationExample {
    fn new(assets: &mut impl Assets) -> Self {
        let now = assets.now();

        let old_hero_atlas =
            assets.frame_fixed_grid_material_png("jotem_spritesheet", TILE_SIZE, JOTEM_SIZE);

        let attack_anim = FrameAnimation::new(12 * 7, 10, 12, now);
        let sleep_anim = FrameAnimation::new(12 * 11 + 1, 5, 5, now);

        let bat_atlas =
            assets.frame_fixed_grid_material_png("flying_46x30", (46, 30).into(), (322, 30).into());

        let bat_anim = FrameAnimation::new(0, 7, 12, now);

        Self {
            old_hero_atlas,
            bat_atlas: Some(bat_atlas),
            attack_anim,
            sleep_anim,
            bat_anim,
            tick_count: 0,
        }
    }

    fn tick(&mut self, _assets: &mut impl Assets) {
        self.tick_count += 1;
        if self.tick_count > 60 && self.bat_atlas.is_some() {
            info!("intentionally unload bat atlas");
            self.bat_atlas.take();
        }
    }

    fn render(&mut self, gfx: &mut impl Gfx) {
        let now = gfx.now();

        gfx.set_clear_color(Color::from_octet(0, 0, 0, 0));

        gfx.set_origin((0, (VIRTUAL_SCREEN_SIZE.y / 2 - CHARACTER_HEIGHT) as i16).into());
        self.attack_anim.update(now);
        self.sleep_anim.update(now);
        self.bat_anim.update(now);

        if let Some(ref mut bat_atlas) = &mut self.bat_atlas {
            gfx.sprite_atlas_frame(
                (
                    (VIRTUAL_SCREEN_SIZE.x / 2u16 - (TILE_SIZE.x / 2u16)) as i16,
                    (VIRTUAL_SCREEN_SIZE.y / 2u16 - CHARACTER_HEIGHT) as i16,
                    0,
                )
                    .into(),
                self.bat_anim.frame(),
                bat_atlas,
            );
        }

        let spacing = 64;

        gfx.sprite_atlas_frame(
            (
                (VIRTUAL_SCREEN_SIZE.x / 2u16 - (TILE_SIZE.x / 2u16) - spacing) as i16,
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

fn main() {
    run::<AnimationExample>("Animation Example", VIRTUAL_SCREEN_SIZE, START_WINDOW_SIZE);
}
