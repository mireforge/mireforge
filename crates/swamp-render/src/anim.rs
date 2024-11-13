/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use monotonic_time_rs::{Millis, MillisDuration};
use std::ops::{Div, Sub};

pub trait AnimationLookup {
    fn frame(&self) -> u16;
}

#[derive(Debug)]
pub struct FrameAnimation {
    start_frame: u16,
    count: u8,
    frame: u16,
    started_at_time: Millis,
    frame_duration: MillisDuration,
}

#[derive(Debug, Copy, Clone)]
pub struct Tick(u64);

impl Tick {
    #[inline]
    pub const fn inner(&self) -> u64 {
        self.0
    }
}

impl Sub for Tick {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

pub type NumberOfTicks = u64;

impl Div<NumberOfTicks> for Tick {
    type Output = u64;

    fn div(self, rhs: NumberOfTicks) -> Self::Output {
        self.0 / rhs
    }
}

pub type Fps = u16;

impl FrameAnimation {
    pub fn new(start_frame: u16, count: u8, fps: Fps, now: Millis) -> Self {
        Self {
            start_frame,
            count,
            started_at_time: now,
            frame: start_frame,
            frame_duration: MillisDuration::from_millis(1000) / (fps as u32),
        }
    }
    pub fn update(&mut self, now: Millis) {
        let elapsed_ticks = now - self.started_at_time;
        let frames_since_start = elapsed_ticks.as_millis() / self.frame_duration.as_millis();
        let frames_index = (frames_since_start % self.count as u64) as u16;

        self.frame = self.start_frame + frames_index;
    }
}

impl AnimationLookup for FrameAnimation {
    fn frame(&self) -> u16 {
        self.frame
    }
}
