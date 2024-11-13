/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use int_math::Vec2;

#[derive(Debug, Hash, Clone, Copy, Eq, PartialEq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Hash, Clone, Copy, Eq, PartialEq)]
pub enum MouseScrollDelta {
    LineDelta(Vec2),
    PixelDelta(Vec2),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Hash, Clone, Copy, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}
