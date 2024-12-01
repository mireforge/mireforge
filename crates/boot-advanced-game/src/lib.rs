/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use int_math::UVec2;
use limnus::prelude::{App, AppReturnValue, DefaultPlugins, Window};
use swamp_advanced_game::audio::GameAudioRenderPlugin;
use swamp_advanced_game::logic::GameLogicPlugin;
use swamp_advanced_game::render::GameRendererPlugin;
use swamp_advanced_game::{ApplicationAudio, ApplicationLogic, ApplicationRender};
use swamp_font::FontPlugin;
use swamp_material::MaterialPlugin;
use swamp_render_wgpu::plugin::RenderWgpuPlugin;

pub fn run_advanced<L: ApplicationLogic, R: ApplicationRender<L>, A: ApplicationAudio<L>>(
    title: &str,
    virtual_size: UVec2,
    requested_surface_size: UVec2,
) -> AppReturnValue {
    App::new()
        .insert_resource(Window {
            title: title.to_string(),
            requested_surface_size,
            minimal_surface_size: virtual_size,
            fullscreen: false,
        })
        .add_plugins((DefaultPlugins, RenderWgpuPlugin, MaterialPlugin))
        .add_plugins(GameRendererPlugin::<R, L>::new())
        .add_plugins(GameLogicPlugin::<L>::new())
        .add_plugins(GameAudioRenderPlugin::<A, L>::new())
        .add_plugins(FontPlugin)
        .run()
}
