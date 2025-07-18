/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use monotonic_time_rs::{Millis, MillisDuration};
use std::ops::{Div, Sub};

pub trait AnimationLookup {
    fn frame(&self) -> u16;
}

#[derive(Debug, Copy, Clone)]
pub struct Tick(u64);

impl Tick {
    #[inline]
    #[must_use]
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

#[derive(Debug, Copy, Clone)]
pub struct FrameAnimationConfig {
    start_frame: u16,
    count: u8,
    frame_duration: MillisDuration,
}

impl FrameAnimationConfig {
    #[must_use]
    pub fn new(start_frame: u16, count: u8, fps: Fps) -> Self {
        Self {
            start_frame,
            count,
            frame_duration: MillisDuration::from_millis(1000) / u32::from(fps),
        }
    }
}

#[derive(Debug)]
pub enum PlayMode {
    Once,
    Repeat,
}

#[derive(Debug)]
pub struct FrameAnimation {
    started_at_time: Millis,
    is_playing: bool,
    config: FrameAnimationConfig,
    relative_frame: u8,
    play_mode: PlayMode,
}

impl FrameAnimation {
    #[must_use]
    pub fn new(config: FrameAnimationConfig) -> Self {
        Self {
            started_at_time: Millis::new(0),
            is_playing: false,
            config,
            relative_frame: 0,
            play_mode: PlayMode::Once,
        }
    }

    /// # Panics
    ///
    pub fn update(&mut self, now: Millis) {
        if !self.is_playing {
            return;
        }

        assert!(now >= self.started_at_time);

        let elapsed_ticks = now - self.started_at_time;
        let frames_since_start = elapsed_ticks.as_millis() / self.config.frame_duration.as_millis();

        match self.play_mode {
            PlayMode::Once => {
                if frames_since_start >= u64::from(self.config.count) {
                    self.is_playing = false;
                    self.relative_frame = (u16::from(self.config.count) - 1) as u8;
                } else {
                    self.relative_frame = frames_since_start as u8;
                }
            }
            PlayMode::Repeat => {
                self.relative_frame = (frames_since_start % u64::from(self.config.count)) as u8;
            }
        }
    }

    #[must_use]
    pub const fn is_done(&self) -> bool {
        !self.is_playing
    }

    #[must_use]
    pub const fn is_playing(&self) -> bool {
        self.is_playing
    }

    #[must_use]
    pub const fn absolute_frame(&self) -> u16 {
        self.relative_frame as u16 + self.config.start_frame
    }

    #[must_use]
    pub const fn relative_frame(&self) -> u16 {
        self.relative_frame as u16
    }

    pub const fn play(&mut self, now: Millis) {
        self.is_playing = true;
        self.play_mode = PlayMode::Once;
        self.started_at_time = now;
    }

    pub const fn play_repeat(&mut self, now: Millis) {
        self.is_playing = true;
        self.play_mode = PlayMode::Repeat;
        self.started_at_time = now;
    }
}

impl AnimationLookup for FrameAnimation {
    fn frame(&self) -> u16 {
        self.absolute_frame()
    }
}
