/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use mireforge::prelude::*;

const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(320, 240);
const START_WINDOW_SIZE: UVec2 = UVec2::new(1280, 800);

#[derive(Debug)]
pub struct AlphaExample {
    alpha: MaterialRef,
    tick_count: u32,
}

impl AlphaExample {
    fn fun_value(&self, speed: f32, min: i16, max: i16, offset: u32) -> i16 {
        let angle = (self.tick_count + offset) as f32 * 0.1 * speed;
        let sin_value = angle.sin();

        let pos_sin = (sin_value + 1.0) / 2.0;
        let pos_sin_int = (pos_sin * (max - min) as f32) as u16;

        pos_sin_int as i16 + min
    }
}

impl Application for AlphaExample {
    fn new(assets: &mut impl Assets) -> Self {
        let alpha = assets.material_png("test_mask.alpha");

        Self {
            alpha,
            tick_count: 0,
        }
    }

    fn tick(&mut self, _assets: &mut impl Assets) {
        self.tick_count += 1;
    }

    fn render(&mut self, gfx: &mut impl Gfx) {
        let width = self.fun_value(0.15, 30, 310, self.tick_count + 25);
        let height = self.fun_value(0.05, 30, 230, self.tick_count);
    }
}

fn main() {
    run::<AlphaExample>("Alpha Mask Example", VIRTUAL_SCREEN_SIZE, START_WINDOW_SIZE);
}
