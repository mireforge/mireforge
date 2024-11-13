/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use swamp_boot_game::prelude::*;

pub struct ExampleGame {}

impl Application for ExampleGame {
    fn new(_assets: &mut impl Assets) -> Self {
        Self {}
    }
    fn tick(&mut self, _assets: &mut impl Assets) {
        info!("ticking!");
    }

    fn render(&mut self, _gfx: &mut impl Gfx) {
        info!("rendering!");
    }

    fn mouse_input(&mut self, state: ButtonState, button: MouseButton) {
        info!("mouse_input: {state:?} {button:?}");
    }
}

fn main() {
    run::<ExampleGame>("example game", UVec2::new(640, 480), UVec2::new(320, 240));
}
