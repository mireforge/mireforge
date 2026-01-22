/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use mireforge::prelude::*;

const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(320 * 2, 240 * 2);
const START_WINDOW_SIZE: UVec2 = UVec2::new(VIRTUAL_SCREEN_SIZE.x * 2, VIRTUAL_SCREEN_SIZE.y * 2);

#[derive(Debug)]
pub struct ViewportPhysicalSizeExample {
    tile_map_atlas: FixedAtlas,
    tick_count: u32,
    tiles: Vec<u16>,
    scroll_wheel_zoom: i16,
    quit: bool,
}

const GRID_WIDTH: u16 = 33u16;
const DOOR: u16 = 3 * GRID_WIDTH + 10;
const FILLED_WALL: u16 = 9 * GRID_WIDTH + 10;
const FLOOR_A: u16 = 6 * GRID_WIDTH + 1;
const FLOOR_B: u16 = 7 * GRID_WIDTH + 1;
const BROKEN_WALL: u16 = 9 * GRID_WIDTH + 11;
const CHEST: u16 = 3 * GRID_WIDTH + 18;

const CELL_SIZE: u16 = 10;

const SCROLL_WHEEL_MAX: u16 = 20_000; // can differ a lot between platforms

const TEXTURE_SIZE: UVec2 = UVec2::new(330, 460);

fn fun_value(tick_count: u32, speed: f32, min: i16, max: i16, offset: u32) -> i16 {
    let angle = (tick_count + offset) as f32 * 0.1 * speed;
    let sin_value = angle.sin();

    let pos_sin = f32::midpoint(sin_value, 1.0);
    let pos_sin_int = (pos_sin * f32::from(max - min)) as u16;

    pos_sin_int as i16 + min
}
impl ViewportPhysicalSizeExample {
    fn converted_zoom(scroll_wheel_zoom: i16) -> f32 {
        const MAX_ZOOM: f32 = 0.05;

        let mut converted_zoom = f32::from(scroll_wheel_zoom) / f32::from(SCROLL_WHEEL_MAX);
        converted_zoom *= MAX_ZOOM;

        converted_zoom.clamp(0.001, MAX_ZOOM)
    }
}

impl Application for ViewportPhysicalSizeExample {
    fn new(assets: &mut impl Assets) -> Self {
        let tiles: &[u16] = &[
            // Note: Rows are upside down
            FILLED_WALL,
            BROKEN_WALL,
            DOOR,
            FILLED_WALL,
            FILLED_WALL,
            FILLED_WALL,
            FLOOR_A,
            FLOOR_B,
            FLOOR_A,
            FILLED_WALL,
            FILLED_WALL,
            FLOOR_A,
            FLOOR_B,
            CHEST,
            FILLED_WALL,
            FILLED_WALL,
            BROKEN_WALL,
            BROKEN_WALL,
            FILLED_WALL,
            FILLED_WALL,
        ];

        Self {
            tile_map_atlas: assets.frame_fixed_grid_material_png(
                "bountiful_bits",
                (CELL_SIZE, CELL_SIZE).into(),
                TEXTURE_SIZE,
            ),
            tick_count: 0,
            tiles: tiles.to_vec(),
            scroll_wheel_zoom: 3,
            quit: false,
        }
    }

    fn tick(&mut self, _assets: &mut impl Assets) {
        self.tick_count += 1;
    }

    fn render(&mut self, gfx: &mut impl Gfx) {
        const SCALE: u16 = 2;

        gfx.set_viewport(ViewportStrategy::MatchPhysicalSize);

        let converted_zoom = Self::converted_zoom(self.scroll_wheel_zoom);

        //info!("zoom: {converted_zoom} {}", gfx.physical_aspect_ratio());

        gfx.set_scale(VirtualScale::FloatScale(converted_zoom));

        let tile: u16 = if (self.tick_count / 40).is_multiple_of(2) {
            CHEST
        } else {
            FLOOR_A
        };
        let tile_ref = self.tiles.get_mut((2 * 5) as usize + 3).unwrap();
        *tile_ref = tile;

        let x = fun_value(
            self.tick_count,
            0.07,
            0,
            (VIRTUAL_SCREEN_SIZE.x - 5 * CELL_SIZE * SCALE) as i16,
            0,
        );
        let y = fun_value(
            self.tick_count,
            0.02,
            0,
            (VIRTUAL_SCREEN_SIZE.y - 4 * CELL_SIZE * SCALE) as i16,
            0,
        );
        gfx.tilemap_params(
            (x, y, 1).into(),
            &self.tiles,
            5,
            &self.tile_map_atlas,
            SCALE as u8,
        );

        gfx.tilemap_params((0, 0, -10).into(), &self.tiles, 5, &self.tile_map_atlas, 4);
    }

    fn wants_to_quit(&self) -> bool {
        self.quit
    }

    fn keyboard_input(&mut self, state: ButtonState, key: KeyCode) {
        if state == ButtonState::Pressed && key == KeyCode::Escape {
            self.quit = true;
        }
    }

    fn mouse_wheel(&mut self, delta_y: i16) {
        self.scroll_wheel_zoom += delta_y;
        self.scroll_wheel_zoom = self.scroll_wheel_zoom.clamp(0, SCROLL_WHEEL_MAX as i16);
        info!(
            "total delta: {delta_y} zoom {} factor:{}",
            self.scroll_wheel_zoom,
            Self::converted_zoom(self.scroll_wheel_zoom)
        );
    }
}

fn main() {
    let _ = run::<ViewportPhysicalSizeExample>(
        "Viewport Physical Size Example",
        VIRTUAL_SCREEN_SIZE,
        START_WINDOW_SIZE,
    );
}
