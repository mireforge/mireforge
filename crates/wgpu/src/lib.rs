/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use wgpu::{
    AddressMode, Device, FilterMode, Sampler, SamplerDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderSource,
};

#[must_use]
pub fn create_shader_module(device: &Device, name: &str, shader_source: &str) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
        label: Some(name),
        source: ShaderSource::Wgsl(shader_source.into()),
    })
}

#[must_use]
pub fn create_nearest_sampler(device: &Device, label: &str) -> Sampler {
    device.create_sampler(&SamplerDescriptor {
        label: Some(label),
        address_mode_u: AddressMode::Repeat, // AddressMode::ClampToEdge,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        compare: None,
        anisotropy_clamp: 1,
        lod_min_clamp: 0.0,
        lod_max_clamp: 32.0,
        border_color: None,
    })
}
