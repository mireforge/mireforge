/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::button::{ButtonState, MouseButton, MouseScrollDelta, TouchPhase};
use crate::key::KeyCode;
use swamp_message::Message;

mod button;
mod key;
pub mod prelude;

#[derive(Debug, Copy, Clone)]
pub enum InputMessage {
    KeyboardInput(ButtonState, KeyCode),
    MouseInput(ButtonState, MouseButton),
    MouseWheel(MouseScrollDelta, TouchPhase),
}

impl Message for InputMessage {}
