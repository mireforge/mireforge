/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::{Material, Render};
use limnus_app::prelude::{App, Plugin};
use limnus_assets::prelude::Assets as LimnusAssets;
use limnus_clock::Clock;
use limnus_default_stages::{RenderFirst, RenderPostUpdate};
use limnus_screen::{Window, WindowMessage};
use limnus_system_params::{LoRe, Msg, Re, ReM};
use limnus_wgpu_window::WgpuWindow;
use monotonic_time_rs::Millis;
use std::sync::Arc;
use swamp_font::Font;
use tracing::debug;

fn tick(mut wgpu_render: ReM<Render>, window_messages: Msg<WindowMessage>) {
    for msg in window_messages.iter_previous() {
        if let WindowMessage::Resized(size) = msg {
            debug!("wgpu_render detected resized to {:?}", size);
            wgpu_render.resize(*size);
        }
    }
}

/// # Panics
///
pub fn flush_render_tick(
    script: LoRe<Clock>,
    wgpu_window: LoRe<WgpuWindow>,
    mut wgpu_render: ReM<Render>,
    materials: Re<LimnusAssets<Material>>,
    fonts: Re<LimnusAssets<Font>>,
) {
    let now = script.clock.now();

    wgpu_window
        .render(wgpu_render.clear_color(), |render_pass| {
            wgpu_render.render(render_pass, &materials, &fonts, now);
        })
        .unwrap();
}
pub struct RenderWgpuPlugin;

impl Plugin for RenderWgpuPlugin {
    fn post_initialization(&self, app: &mut App) {
        let window = app.local_resources().fetch::<WgpuWindow>();
        let window_settings = app.resource::<Window>();
        let wgpu_render = Render::new(
            Arc::clone(window.device()),
            Arc::clone(window.queue()),
            window.texture_format(),
            window_settings.requested_surface_size,
            window_settings.minimal_surface_size,
            Millis::new(0),
        );
        app.insert_resource(wgpu_render);

        app.add_system(RenderFirst, tick);
        app.add_system(RenderPostUpdate, flush_render_tick);
    }
}
