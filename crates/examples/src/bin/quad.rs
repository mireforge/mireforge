/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use mireforge::prelude::*;

const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(320, 240);
const START_WINDOW_SIZE: UVec2 = UVec2::new(1280, 800);

#[derive(Debug)]
pub struct QuadExample {
    tick_count: u32,
    #[allow(unused)]
    alpha: MaterialRef,
}

impl QuadExample {
    fn fun_value(&self, speed: f32, min: i16, max: i16, offset: u32) -> i16 {
        let angle = (self.tick_count + offset) as f32 * 0.1 * speed;
        let sin_value = angle.sin();

        let pos_sin = f32::midpoint(sin_value, 1.0);
        let pos_sin_int = (pos_sin * f32::from(max - min)) as u16;

        pos_sin_int as i16 + min
    }
}

impl Application for QuadExample {
    fn new(assets: &mut impl Assets) -> Self {
        let alpha = assets.material_png("test_mask.alpha");
        Self {
            tick_count: 0,
            alpha,
        }
    }

    fn tick(&mut self, _assets: &mut impl Assets) {
        self.tick_count += 1;
    }

    fn render(&mut self, gfx: &mut impl Gfx) {
        let x = self.fun_value(0.09, 100, 200, self.tick_count);
        let y = self.fun_value(0.08, 80, 100, self.tick_count);
        let base_angle = self.tick_count as f32 / 30.0 % std::f32::consts::TAU;

        for i in 0..100 {
            let angle = (i as f32 / 50.0).mul_add(std::f32::consts::TAU, base_angle);
            let distance = i as f32 * 0.8;
            let size_distance = i as f32 * 1.5;

            let pos_x = x + (angle.cos() * distance) as i16;
            let pos_y = y + (angle.sin() * distance) as i16;

            let size = (50.0 - (size_distance / 3.0).min(45.0)) as u16;

            let percentage = f64::from(i) / 100.0;
            let hue = percentage;
            let saturation = 1.0;
            let value = 1.0;
            let color = hsv_to_rgb(hue as f32, saturation, value, percentage as f32);

            gfx.quad((pos_x, pos_y, 0).into(), (size, size).into(), color);
        }
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32, a: f32) -> Color {
    let h = h * 6.0;
    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * f.mul_add(-s, 1.0);
    let t = v * (1.0 - f).mul_add(-s, 1.0);

    let (r, g, b) = match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    Color::from_f32(r, g, b, a)
}

fn main() {
    let _ = run::<QuadExample>("Quad Example", VIRTUAL_SCREEN_SIZE, START_WINDOW_SIZE);
}
