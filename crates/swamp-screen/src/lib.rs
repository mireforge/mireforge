/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use int_math::UVec2;
use swamp_message::prelude::Message;
use swamp_resource::prelude::*;

#[derive(Debug, Resource, Clone)]
pub struct Window {
    pub fullscreen: bool,
    pub title: String,
    pub requested_surface_size: UVec2,
    pub minimal_surface_size: UVec2,
}

#[derive(Message, Debug)]
pub enum WindowMessage {
    CursorMoved(UVec2),
    WindowCreated(),
    Resized(UVec2),
}
