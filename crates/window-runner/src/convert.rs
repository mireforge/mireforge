/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use int_math::Vec2;
use swamp_basic_input::prelude::*;
use winit::event::ElementState;
use winit::{event, keyboard};

pub enum EventConversionError {
    ValueNotFound,
}

pub(crate) fn from_element_state(state: ElementState) -> ButtonState {
    match state {
        ElementState::Pressed => ButtonState::Pressed,
        ElementState::Released => ButtonState::Released,
    }
}

pub(crate) fn try_from_mouse_button(
    button: event::MouseButton,
) -> Result<MouseButton, EventConversionError> {
    let converted = match button {
        event::MouseButton::Left => MouseButton::Left,
        event::MouseButton::Right => MouseButton::Right,
        event::MouseButton::Middle => MouseButton::Middle,
        event::MouseButton::Back => MouseButton::Back,
        event::MouseButton::Forward => MouseButton::Forward,
        _ => Err(EventConversionError::ValueNotFound)?,
    };

    Ok(converted)
}

pub(crate) fn from_mouse_scroll_delta(delta: event::MouseScrollDelta) -> MouseScrollDelta {
    match delta {
        event::MouseScrollDelta::LineDelta(x, y) => {
            const WHEEL_DELTA: f32 = 120.0;

            let scale_up = Vec2::new((x * WHEEL_DELTA) as i16, (-y * WHEEL_DELTA) as i16);
            MouseScrollDelta::LineDelta(scale_up)
        }
        event::MouseScrollDelta::PixelDelta(delta) => {
            MouseScrollDelta::PixelDelta(Vec2::new(delta.x as i16, delta.y as i16))
        }
    }
}

pub(crate) fn from_touch_phase(button: event::TouchPhase) -> TouchPhase {
    match button {
        event::TouchPhase::Started => TouchPhase::Started,
        event::TouchPhase::Moved => TouchPhase::Moved,
        event::TouchPhase::Ended => TouchPhase::Ended,
        event::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

#[derive(Debug)]
pub enum KeyCodeError {
    KeyCodeNotFound,
}

pub(crate) fn try_from_key_code(keycode: keyboard::KeyCode) -> Result<KeyCode, KeyCodeError> {
    let swamp_key_code = match keycode {
        keyboard::KeyCode::Backquote => KeyCode::Backquote,
        keyboard::KeyCode::Backslash => KeyCode::Backslash,
        keyboard::KeyCode::BracketLeft => KeyCode::BracketLeft,
        keyboard::KeyCode::BracketRight => KeyCode::BracketRight,
        keyboard::KeyCode::Comma => KeyCode::Comma,

        keyboard::KeyCode::Digit0 => KeyCode::Digit0,
        keyboard::KeyCode::Digit1 => KeyCode::Digit1,
        keyboard::KeyCode::Digit2 => KeyCode::Digit2,
        keyboard::KeyCode::Digit3 => KeyCode::Digit3,
        keyboard::KeyCode::Digit4 => KeyCode::Digit4,
        keyboard::KeyCode::Digit5 => KeyCode::Digit5,
        keyboard::KeyCode::Digit6 => KeyCode::Digit6,
        keyboard::KeyCode::Digit7 => KeyCode::Digit7,
        keyboard::KeyCode::Digit8 => KeyCode::Digit8,
        keyboard::KeyCode::Digit9 => KeyCode::Digit9,
        keyboard::KeyCode::Equal => KeyCode::Equal,

        keyboard::KeyCode::KeyA => KeyCode::KeyA,
        keyboard::KeyCode::KeyB => KeyCode::KeyB,
        keyboard::KeyCode::KeyC => KeyCode::KeyC,
        keyboard::KeyCode::KeyD => KeyCode::KeyD,
        keyboard::KeyCode::KeyE => KeyCode::KeyE,
        keyboard::KeyCode::KeyF => KeyCode::KeyF,
        keyboard::KeyCode::KeyG => KeyCode::KeyG,
        keyboard::KeyCode::KeyH => KeyCode::KeyH,
        keyboard::KeyCode::KeyI => KeyCode::KeyI,
        keyboard::KeyCode::KeyJ => KeyCode::KeyJ,
        keyboard::KeyCode::KeyK => KeyCode::KeyK,
        keyboard::KeyCode::KeyL => KeyCode::KeyL,
        keyboard::KeyCode::KeyM => KeyCode::KeyM,
        keyboard::KeyCode::KeyN => KeyCode::KeyN,
        keyboard::KeyCode::KeyO => KeyCode::KeyO,
        keyboard::KeyCode::KeyP => KeyCode::KeyP,
        keyboard::KeyCode::KeyQ => KeyCode::KeyQ,
        keyboard::KeyCode::KeyR => KeyCode::KeyR,
        keyboard::KeyCode::KeyS => KeyCode::KeyS,
        keyboard::KeyCode::KeyT => KeyCode::KeyT,
        keyboard::KeyCode::KeyU => KeyCode::KeyU,
        keyboard::KeyCode::KeyV => KeyCode::KeyV,
        keyboard::KeyCode::KeyW => KeyCode::KeyW,
        keyboard::KeyCode::KeyX => KeyCode::KeyX,
        keyboard::KeyCode::KeyY => KeyCode::KeyY,
        keyboard::KeyCode::KeyZ => KeyCode::KeyZ,

        keyboard::KeyCode::Minus => KeyCode::Minus,
        keyboard::KeyCode::Period => KeyCode::Period,
        keyboard::KeyCode::Quote => KeyCode::Quote,
        keyboard::KeyCode::Semicolon => KeyCode::Semicolon,
        keyboard::KeyCode::Slash => KeyCode::Slash,

        keyboard::KeyCode::AltLeft => KeyCode::AltLeft,
        keyboard::KeyCode::AltRight => KeyCode::AltRight,
        keyboard::KeyCode::Backspace => KeyCode::Backspace,
        keyboard::KeyCode::CapsLock => KeyCode::CapsLock,

        keyboard::KeyCode::ContextMenu => KeyCode::ContextMenu,
        keyboard::KeyCode::ControlLeft => KeyCode::ControlLeft,
        keyboard::KeyCode::ControlRight => KeyCode::ControlRight,
        keyboard::KeyCode::Enter => KeyCode::Enter,
        keyboard::KeyCode::SuperLeft => KeyCode::SuperLeft,
        keyboard::KeyCode::SuperRight => KeyCode::SuperRight,
        keyboard::KeyCode::ShiftLeft => KeyCode::ShiftLeft,
        keyboard::KeyCode::ShiftRight => KeyCode::ShiftRight,
        keyboard::KeyCode::Space => KeyCode::Space,
        keyboard::KeyCode::Tab => KeyCode::Tab,
        keyboard::KeyCode::Delete => KeyCode::Delete,
        keyboard::KeyCode::End => KeyCode::End,
        keyboard::KeyCode::Help => KeyCode::Help,
        keyboard::KeyCode::Home => KeyCode::Home,
        keyboard::KeyCode::Insert => KeyCode::Insert,
        keyboard::KeyCode::PageDown => KeyCode::PageDown,
        keyboard::KeyCode::PageUp => KeyCode::PageUp,
        keyboard::KeyCode::ArrowDown => KeyCode::ArrowDown,
        keyboard::KeyCode::ArrowLeft => KeyCode::ArrowLeft,
        keyboard::KeyCode::ArrowRight => KeyCode::ArrowRight,
        keyboard::KeyCode::ArrowUp => KeyCode::ArrowUp,
        keyboard::KeyCode::NumLock => KeyCode::NumLock,

        keyboard::KeyCode::Escape => KeyCode::Escape,

        keyboard::KeyCode::PrintScreen => KeyCode::PrintScreen,
        keyboard::KeyCode::ScrollLock => KeyCode::ScrollLock,
        keyboard::KeyCode::Pause => KeyCode::Pause,

        keyboard::KeyCode::F1 => KeyCode::F1,
        keyboard::KeyCode::F2 => KeyCode::F2,
        keyboard::KeyCode::F3 => KeyCode::F3,
        keyboard::KeyCode::F4 => KeyCode::F4,
        keyboard::KeyCode::F5 => KeyCode::F5,
        keyboard::KeyCode::F6 => KeyCode::F6,
        keyboard::KeyCode::F7 => KeyCode::F7,
        keyboard::KeyCode::F8 => KeyCode::F8,
        keyboard::KeyCode::F9 => KeyCode::F9,
        keyboard::KeyCode::F10 => KeyCode::F10,
        keyboard::KeyCode::F11 => KeyCode::F11,
        keyboard::KeyCode::F12 => KeyCode::F12,

        _ => Err(KeyCodeError::KeyCodeNotFound)?,
    };

    Ok(swamp_key_code)
}
