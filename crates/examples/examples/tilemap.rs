/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use swamp::prelude::*;

const VIRTUAL_SCREEN_SIZE: UVec2 = UVec2::new(320 * 2, 240 * 2);
const START_WINDOW_SIZE: UVec2 = UVec2::new(VIRTUAL_SCREEN_SIZE.x * 2, VIRTUAL_SCREEN_SIZE.y * 2);

const TEXTURE_SIZE: UVec2 = UVec2::new(330, 460);

#[derive(Debug)]
pub struct TileMapExample {
    tile_map_atlas: FixedAtlas,
    tick_count: u32,
    tiles: Vec<u16>,
    cursor_position: UVec2,
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

const SCROLL_WHEEL_MAX: u16 = 2_000; // can differ a lot between platforms

fn fun_value(tick_count: u32, speed: f32, min: i16, max: i16, offset: u32) -> i16 {
    let angle = (tick_count + offset) as f32 * 0.1 * speed;
    let sin_value = angle.sin();

    let pos_sin = (sin_value + 1.0) / 2.0;
    let pos_sin_int = (pos_sin * (max - min) as f32) as u16;

    pos_sin_int as i16 + min
}

impl Application for TileMapExample {
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
            cursor_position: UVec2::new(0, 0),
            scroll_wheel_zoom: 3,
            quit: false,
        }
    }

    fn tick(&mut self, _assets: &mut impl Assets) {
        self.tick_count += 1;
    }

    fn render(&mut self, gfx: &mut impl Gfx) {
        const SCALE: u16 = 2;

        let tile: u16 = if ((self.tick_count / 40) % 2) == 0 {
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

        let mut converted_zoom =
            (self.scroll_wheel_zoom as u64 * 5 / SCROLL_WHEEL_MAX as u64) as u8;
        converted_zoom = converted_zoom.clamp(0, 5) + 1;

        gfx.tilemap_params(
            (
                self.cursor_position.x as i16,
                self.cursor_position.y as i16,
                -10,
            )
                .into(),
            &self.tiles,
            5,
            &self.tile_map_atlas,
            converted_zoom,
        );
    }

    fn wants_to_quit(&self) -> bool {
        self.quit
    }

    fn keyboard_input(&mut self, state: ButtonState, key: KeyCode) {
        if state == ButtonState::Pressed && key == KeyCode::Escape {
            self.quit = true;
        }
    }

    fn cursor_moved(&mut self, position: UVec2) {
        self.cursor_position = position;
    }

    fn mouse_wheel(&mut self, delta_y: i16) {
        self.scroll_wheel_zoom += delta_y;
        self.scroll_wheel_zoom = self.scroll_wheel_zoom.clamp(0, SCROLL_WHEEL_MAX as i16);
    }
}

fn main() {
    run::<TileMapExample>("TileMap Example", VIRTUAL_SCREEN_SIZE, START_WINDOW_SIZE);
}
