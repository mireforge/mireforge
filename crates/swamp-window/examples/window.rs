/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use std::sync::Arc;
use swamp_log::prelude::{info, trace};
use swamp_window::AppHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    ElementState, InnerSizeWriter, MouseButton, MouseScrollDelta, Touch, TouchPhase,
};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

pub struct Handler {
    should_quit: bool,
    cursor_visible: bool,
}

impl AppHandler for Handler {
    // Query
    fn min_size(&self) -> (u16, u16) {
        (640, 480)
    }

    fn start_size(&self) -> (u16, u16) {
        (640 * 2, 480 * 2)
    }

    fn cursor_should_be_visible(&self) -> bool {
        self.cursor_visible
    }

    // Window
    fn redraw(&mut self) -> bool {
        trace!("redraw");
        !self.should_quit
    }

    fn got_focus(&mut self) {
        info!("got focus");
    }

    fn lost_focus(&mut self) {
        info!("lost focus");
    }

    fn window_created(&mut self, window: Arc<Window>) {
        info!("window was created {window:?}");
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        info!("resized {size:?}");
    }

    // Keyboard
    fn keyboard_input(&mut self, element_state: ElementState, physical_key: PhysicalKey) {
        info!("keyboard_input {element_state:?} {physical_key:?}");
        self.should_quit = physical_key == PhysicalKey::Code(KeyCode::KeyQ);
        if element_state == ElementState::Pressed
            && physical_key == PhysicalKey::Code(KeyCode::KeyC)
        {
            self.cursor_visible = !self.cursor_visible;
            info!("toggle cursor to: {}", self.cursor_visible);
        }
    }

    // Cursor

    fn cursor_entered(&mut self) {
        info!("cursor entered");
    }

    fn cursor_left(&mut self) {
        info!("cursor left");
    }

    fn cursor_moved(&mut self, physical_position: PhysicalPosition<u32>) {
        info!("cursor moved {physical_position:?}");
    }

    // Mouse
    fn mouse_input(&mut self, element_state: ElementState, button: MouseButton) {
        info!("mouse_input {element_state:?} {button:?}");
    }

    fn mouse_wheel(&mut self, delta: MouseScrollDelta, touch_phase: TouchPhase) {
        info!("mouse_wheel {delta:?} {touch_phase:?}");
    }

    fn pinch_gesture(&mut self, delta: f64, touch_phase: TouchPhase) {
        info!("pinch_gesture {delta:?} {touch_phase:?}");
    }

    fn mouse_motion(&mut self, motion: (f64, f64)) {
        info!("mouse motion {motion:?}");
    }

    // Touch
    fn touch(&mut self, touch: Touch) {
        info!("touch {touch:?}");
    }

    // Environment
    fn scale_factor_changed(&mut self, scale_factor: f64, mut inner_size_writer: InnerSizeWriter) {
        info!("scale factor changed {scale_factor:?}");
        inner_size_writer
            .request_inner_size(PhysicalSize::new(800, 500))
            .unwrap()
    }
}

fn main() {
    env_logger::init();

    let mut handler = Handler {
        should_quit: false,
        cursor_visible: true,
    };

    swamp_window::WindowRunner::run_app(&mut handler, "Handler Callback Example").unwrap()
}
