/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod anim;
pub mod prelude;

use int_math::UVec2;
use std::fmt;

pub enum VirtualScale {
    IntScale(u16),
    FloatScale(f32),
}

fn gcd(a: u16, b: u16) -> u16 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn aspect_ratio(size: UVec2) -> (u16, u16) {
    let divisor = gcd(size.x, size.y);
    (size.x / divisor, size.y / divisor)
}

#[derive(Debug)]
pub enum AspectRatio {
    Ratio16By9,
    Ratio21By9,
    Ratio16By10,
    Ratio4By3,
    Other(f32),
}

impl AspectRatio {
    fn convert(value: UVec2) -> Self {
        let aspect = aspect_ratio(value);
        match aspect {
            (16, 9) => Self::Ratio16By9,
            (21, 9) => Self::Ratio21By9,
            (16, 10) => Self::Ratio16By10,
            (4, 3) => Self::Ratio4By3,
            _ => Self::Other(f32::from(value.x) / f32::from(value.y)),
        }
    }
}

impl From<(u16, u16)> for AspectRatio {
    fn from(value: (u16, u16)) -> Self {
        Self::convert(value.into())
    }
}

impl From<UVec2> for AspectRatio {
    fn from(value: UVec2) -> Self {
        Self::convert(value)
    }
}

impl fmt::Display for AspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ratio16By9 => write!(f, "16:9"),
            Self::Ratio16By10 => write!(f, "16:10"),
            Self::Ratio21By9 => write!(f, "21:9"),
            Self::Ratio4By3 => write!(f, "4:3"),
            Self::Other(vec) => write!(f, "aspect ratio: {vec:?}"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }
}

impl Color {
    #[must_use]
    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::from_octet(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        )
    }

    #[must_use]
    pub const fn from_hex(value: u32) -> Self {
        let r = ((value >> 24) & 0xFF) as u8;
        let g = ((value >> 16) & 0xFF) as u8;
        let b = ((value >> 8) & 0xFF) as u8;
        let a = ((value) & 0xFF) as u8;
        Self::from_octet(r, g, b, a)
    }

    #[must_use]
    pub fn to_f32_slice(&self) -> [f32; 4] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
            f32::from(self.a) / 255.0,
        ]
    }

    #[must_use]
    pub const fn from_octet(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[must_use]
    pub fn to_f64(&self) -> (f64, f64, f64, f64) {
        (
            f64::from(self.r) / 255.0,
            f64::from(self.g) / 255.0,
            f64::from(self.b) / 255.0,
            f64::from(self.a) / 255.0,
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ViewportStrategy {
    /// Tries to set the viewport to fit the virtual surface size within the physical surface size.
    /// Depending on resolution, it can cause black borders, both vertically and horizontally.
    /// Always keeps the aspect ratio, and is "pixel perfect"
    FitIntegerScaling,

    /// Tries to set the viewport to fit the virtual surface size within the physical surface size.
    /// Depending on resolution, it can cause "black borders", *either* vertically and horizontally.
    /// Always keeps the aspect ratio, but might not be pixel perfect
    FitFloatScaling,

    /// The viewport will be the same as the physical size.
    MatchPhysicalSize,
}
