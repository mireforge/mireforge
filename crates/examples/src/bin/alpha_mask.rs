/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use mireforge::prelude::*;

const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(320, 240);
const START_WINDOW_SIZE: UVec2 = UVec2::new(1280, 800);

#[derive(Debug)]
pub struct AlphaExample {
    #[allow(unused)]
    alpha_masked: MaterialRef,
    before: MaterialRef,
    tick_count: u32,
}

impl AlphaExample {
    /*
    fn fun_value(&self, speed: f32, min: i16, max: i16, offset: u32) -> i16 {
        let angle = (self.tick_count + offset) as f32 * 0.1 * speed;
        let sin_value = angle.sin();

        let pos_sin = (sin_value + 1.0) / 2.0;
        let pos_sin_int = (pos_sin * (max - min) as f32) as u16;

        pos_sin_int as i16 + min
    }

     */
}

impl Application for AlphaExample {
    fn new(assets: &mut impl Assets) -> Self {
        let before = assets.material_png("nine_slice_debug");
        let alpha_masked = assets.material_alpha_mask("nine_slice_debug", "test_mask.alpha");

        Self {
            before,
            alpha_masked,
            tick_count: 0,
        }
    }

    fn tick(&mut self, _assets: &mut impl Assets) {
        self.tick_count += 1;
    }

    fn render(&mut self, gfx: &mut impl Gfx) {
        gfx.draw_sprite((20, 0, 0).into(), &self.before);

        let color = Color::from_f32(1.0, 0.3, 1.0, 1.0);
        gfx.draw_with_mask(
            (100, 0, 0).into(),
            (200, 200).into(),
            color,
            &self.alpha_masked,
        );
    }
}

fn main() {
    let _ = run::<AlphaExample>("Alpha Mask Example", VIRTUAL_SCREEN_SIZE, START_WINDOW_SIZE);
}
