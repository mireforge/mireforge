/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use mireforge::prelude::*;

const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(320 * 2, 240 * 2);
const START_WINDOW_SIZE: UVec2 = UVec2::new(VIRTUAL_SCREEN_SIZE.x * 2, VIRTUAL_SCREEN_SIZE.y * 2);

#[derive(Debug)]
pub struct FontGlyphExample {
    font: FontAndMaterial,
    offset: u16,
    glyphs: Option<GlyphDraw>,
}
const REALLY_LONG_STRING: &str = "\
... it will start soon...         Hello, can you see this text? If so, it seems to be working. \
Grab a snack, sit back, and enjoy the scrolling... it’s mesmerizing, isn’t it? \
So cool to bring back the scroll text from the 1980s. #retro                 ";

impl Application for FontGlyphExample {
    fn new(assets: &mut impl Assets) -> Self {
        Self {
            font: assets.bm_font("menu"),
            offset: 0,
            glyphs: None,
        }
    }

    fn tick(&mut self, assets: &mut impl Assets) {
        self.offset += 4;
        self.offset %= VIRTUAL_SCREEN_SIZE.x * 12;
        if self.glyphs.is_none() {
            self.glyphs = assets.text_glyphs(REALLY_LONG_STRING, &self.font);
        }
    }

    fn render(&mut self, gfx: &mut impl Gfx) {
        const MAX_AMPLITUDE: i16 = 32;
        let time_angle = f32::from(self.offset) * 0.02;

        if let Some(glyph_draw) = &self.glyphs {
            let start_x = (VIRTUAL_SCREEN_SIZE.x as i16) - (self.offset as i16);
            for glyph in &glyph_draw.glyphs {
                let local_angle = f32::from(glyph.relative_position.x) * 0.002;
                let amplitude = ((time_angle + local_angle).sin() * f32::from(MAX_AMPLITUDE)) as i16;
                gfx.sprite_atlas(
                    Vec3::new(
                        start_x + glyph.relative_position.x,
                        (VIRTUAL_SCREEN_SIZE.y / 2) as i16 + glyph.relative_position.y + amplitude,
                        1,
                    ),
                    glyph.texture_rectangle,
                    &self.font.material_ref,
                );
            }
        }
    }
}

fn main() {
    run::<FontGlyphExample>(
        "Font Glyphs Example",
        VIRTUAL_SCREEN_SIZE,
        START_WINDOW_SIZE,
    );
}
