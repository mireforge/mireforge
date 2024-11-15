/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use bytemuck::{Pod, Zeroable};
use std::ops::{Add, Index, Mul};

// ------------ Ortho -------------
pub struct OrthoInfo {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl From<OrthoInfo> for Matrix4 {
    fn from(ortho: OrthoInfo) -> Self {
        let c0r0 = 2.0 / (ortho.right - ortho.left);
        let c1r1 = 2.0 / (ortho.top - ortho.bottom);

        let c2r2 = -2.0 / (ortho.far - ortho.near);

        let c3r0 = -(ortho.right + ortho.left) / (ortho.right - ortho.left);
        let c3r1 = -(ortho.top + ortho.bottom) / (ortho.top - ortho.bottom);
        let c3r2 = -(ortho.far + ortho.near) / (ortho.far - ortho.near);

        //    #[rustfmt::skip]
        Self([
            [c0r0, 0.0, 0.0, 0.0].into(),
            [0.0, c1r1, 0.0, 0.0].into(),
            [0.0, 0.0, c2r2, 0.0].into(),
            [c3r0, c3r1, c3r2, 1.0].into(),
        ])
    }
}

// ----------------- FMatrix4 ----------------
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Matrix4([Vec4; 4]);

unsafe impl Pod for Matrix4 {}
unsafe impl Zeroable for Matrix4 {}

impl Matrix4 {
    #[inline]
    pub fn from_scale(x: f32, y: f32, z: f32) -> Self {
        Self::from([
            [x, 0.0, 0.0, 0.0],
            [0.0, y, 0.0, 0.0],
            [0.0, 0.0, z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    #[inline]
    pub fn from_translation(x: f32, y: f32, z: f32) -> Self {
        Self::from([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x, y, z, 1.0],
        ])
    }

    #[inline]
    pub fn identity() -> Self {
        Self::from_scale(1.0, 1.0, 1.0)
    }
}

impl From<[[f32; 4]; 4]> for Matrix4 {
    fn from(v: [[f32; 4]; 4]) -> Self {
        Self([v[0].into(), v[1].into(), v[2].into(), v[3].into()])
    }
}

impl Index<usize> for Matrix4 {
    type Output = Vec4;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Mul<Self> for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let a = self[0];
        let b = self[1];
        let c = self[2];
        let d = self[3];

        Self([
            a * rhs[0][0] + b * rhs[0][1] + c * rhs[0][2] + d * rhs[0][3],
            a * rhs[1][0] + b * rhs[1][1] + c * rhs[1][2] + d * rhs[1][3],
            a * rhs[2][0] + b * rhs[2][1] + c * rhs[2][2] + d * rhs[2][3],
            a * rhs[3][0] + b * rhs[3][1] + c * rhs[3][2] + d * rhs[3][3],
        ])
    }
}

// ------------- FVec4

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Vec4(pub [f32; 4]);

impl From<[f32; 4]> for Vec4 {
    fn from(v: [f32; 4]) -> Self {
        Self(v)
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self([self[0] * rhs, self[1] * rhs, self[2] * rhs, self[3] * rhs])
    }
}

impl Add<Self> for Vec4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
        ])
    }
}

impl Index<usize> for Vec4 {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
