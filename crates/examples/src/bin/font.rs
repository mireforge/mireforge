/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use swamp::prelude::*;

const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(320 * 2, 240 * 2);
const START_WINDOW_SIZE: UVec2 = UVec2::new(VIRTUAL_SCREEN_SIZE.x * 2, VIRTUAL_SCREEN_SIZE.y * 2);

#[derive(Debug)]
pub struct FontExample {
    font: FontAndMaterial,
    offset: u16,
}

const REALLY_LONG_STRING: &str = "\
... it will start soon...         Hello, can you see this text? If so, it seems to be working. \
Grab a snack, sit back, and enjoy the scrolling... it’s mesmerizing, isn’t it? \
So cool to bring back the scroll text from the 1980s. #retro";

impl Application for FontExample {
    fn new(assets: &mut impl Assets) -> Self {
        Self {
            font: assets.bm_font("menu"),
            offset: 0,
        }
    }

    fn tick(&mut self, _assets: &mut impl Assets) {
        self.offset += 4;
        self.offset %= VIRTUAL_SCREEN_SIZE.x * 12;
    }

    fn render(&mut self, gfx: &mut impl Gfx) {
        gfx.text_draw(
            (
                VIRTUAL_SCREEN_SIZE.x as i16 - (self.offset as i16),
                VIRTUAL_SCREEN_SIZE.y as i16 / 2i16,
                1,
            )
                .into(),
            REALLY_LONG_STRING,
            &self.font,
        );
    }
}

fn main() {
    run::<FontExample>("Font Example", VIRTUAL_SCREEN_SIZE, START_WINDOW_SIZE);
}
