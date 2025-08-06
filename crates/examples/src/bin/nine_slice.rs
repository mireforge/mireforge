/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use mireforge::prelude::*;

const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(320*2, 240*2);
const START_WINDOW_SIZE: UVec2 = UVec2::new(1280, 800);

#[derive(Debug)]
pub struct NineSliceExample {
    nine_slice: NineSliceAndMaterial,
    tick_count: u32,
}

impl NineSliceExample {
    fn fun_value(&self, speed: f32, min: i16, max: i16, offset: u32) -> i16 {
        let angle = (self.tick_count + offset) as f32 * 0.1 * speed;
        let sin_value = angle.sin();

        let pos_sin = f32::midpoint(sin_value, 1.0);
        let pos_sin_int = (pos_sin * f32::from(max - min)) as u16;

        pos_sin_int as i16 + min
    }
}

impl Application for NineSliceExample {
    fn new(assets: &mut impl Assets) -> Self {
        let nine_slice = assets.nine_slice_material_png(
            "nine_slice_debug",
            Slices {
                left: 10,
                top: 10,
                right: 13,
                bottom: 5,
            },
        );

        Self {
            nine_slice,
            tick_count: 0,
        }
    }

    fn tick(&mut self, _assets: &mut impl Assets) {
        self.tick_count += 1;
    }

    fn render(&mut self, gfx: &mut impl Gfx) {
        let width = self.fun_value(0.15, 30, 310, self.tick_count + 25);
        let height = self.fun_value(0.05, 30, 230, self.tick_count);

        gfx.nine_slice(
            Vec3::new(3, 3, 0),
            UVec2::new(width as u16, height as u16),
            Color::from_f32(1.0, 1.0, 1.0, 1.0),
            &self.nine_slice,
        );

        gfx.nine_slice_stretch(
            Vec3::new(320, 3, 0),
            UVec2::new(width as u16, height as u16),
            Color::from_f32(1.0, 1.0, 1.0, 1.0),
            &self.nine_slice,
        );
    }
}

fn main() {
    let _ = run::<NineSliceExample>("Nine Slice Example", VIRTUAL_SCREEN_SIZE, START_WINDOW_SIZE);
}
