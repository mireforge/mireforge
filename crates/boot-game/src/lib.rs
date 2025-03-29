/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use int_math::UVec2;
use limnus::DefaultPlugins;
use limnus::prelude::{App, AppReturnValue, ScreenMode};
use limnus::prelude::{Plugin, Window};
use mireforge_font::FontPlugin;
use mireforge_game::{Application, GamePlugin, GameSettings};
use mireforge_material::MaterialPlugin;
use mireforge_render_wgpu::plugin::RenderWgpuPlugin;

// #[must_use] // TODO: should be able to convert AppReturnValue in the future
pub fn run<T: Application>(
    title: &str,
    virtual_size: UVec2,
    requested_surface_size: UVec2,
) -> AppReturnValue {
    App::new()
        .insert_resource(Window {
            title: title.to_string(),
            requested_surface_size,
            minimal_surface_size: virtual_size,
            mode: ScreenMode::Windowed,
        })
        .insert_resource(GameSettings { virtual_size })
        .add_plugins((DefaultPlugins, SwampDefaultPlugins))
        .add_plugins(GamePlugin::<T>::new())
        .run()
}

pub struct SwampDefaultPlugins;

impl Plugin for SwampDefaultPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((RenderWgpuPlugin, MaterialPlugin, FontPlugin));
    }
}
