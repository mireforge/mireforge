/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use mireforge::prelude::*;

const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(640, 360);
const START_WINDOW_SIZE: UVec2 = UVec2::new(1280, 800);

#[derive(Debug)]
pub struct LightExample {
    #[allow(unused)]
    light: MaterialRef,
    background: MaterialRef,
    tick_count: u32,
}

impl LightExample {
    fn fun_value(&self, speed: f32, min: i16, max: i16, offset: u32) -> i16 {
        let angle = (self.tick_count + offset) as f32 * 0.1 * speed;
        let sin_value = angle.sin();

        let pos_sin = (sin_value + 1.0) / 2.0;
        let pos_sin_int = (pos_sin * (max - min) as f32) as u16;

        pos_sin_int as i16 + min
    }
}

impl Application for LightExample {
    fn new(assets: &mut impl Assets) -> Self {
        let background = assets.material_png("underwater");
        let light = assets.light_material_png("spot_light");

        Self {
            background,
            light,
            tick_count: 0,
        }
    }

    fn tick(&mut self, _assets: &mut impl Assets) {
        self.tick_count += 1;
    }

    fn render(&mut self, gfx: &mut impl Gfx) {
        let factor = 255;
        let background_color = Color::from_octet(factor, factor, factor, 255);
        let background_sprite_params = SpriteParams {
            texture_size: UVec2 { x: 0, y: 0 },
            texture_pos: UVec2 { x: 0, y: 0 },
            scale: 1,
            rotation: Rotation::default(),
            flip_x: false,
            flip_y: false,
            pivot: Vec2 { y: 0, x: 0 },
            color: background_color,
        };

        gfx.draw_sprite_ex(
            (0, 0, 0).into(),
            &self.background,
            &background_sprite_params,
        );

        for i in 0..2 {
            let x = self.fun_value(0.06, 0, 200, self.tick_count + i * 43 * 4);
            let y = self.fun_value(0.13, 0, 100, self.tick_count + i * 77 * 4) - 80;

            let strength = 0 + self.fun_value(0.03, 10, 64, self.tick_count + i * 97) as u8;
            let color = Color::from_octet(strength, strength, strength, 255);
            let sprite_params = SpriteParams {
                texture_size: UVec2 { x: 0, y: 0 },
                texture_pos: UVec2 { x: 0, y: 0 },
                scale: 1,
                rotation: Rotation::default(),
                flip_x: false,
                flip_y: false,
                pivot: Vec2 { x: 0, y: 0 },
                color,
            };
            gfx.draw_sprite_ex((x, y, 0).into(), &self.light, &sprite_params);
        }
    }
}

fn main() {
    run::<LightExample>("Alpha Mask Example", VIRTUAL_SCREEN_SIZE, START_WINDOW_SIZE);
}
