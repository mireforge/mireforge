use crate::Render;
use std::sync::Arc;
use swamp_app::prelude::{App, Msg, Plugin, ReM, UpdatePhase};
use swamp_screen::{Window, WindowMessage};
use swamp_wgpu_window::WgpuWindow;
use tracing::debug;

fn tick(mut wgpu_render: ReM<Render>, window_messages: Msg<WindowMessage>) {
    for msg in window_messages.iter_previous() {
        if let WindowMessage::Resized(size) = msg {
            debug!("wgpu_render detected resized to {:?}", size);
            wgpu_render.resize(*size)
        }
    }
}

pub struct RenderWgpuPlugin;

impl Plugin for RenderWgpuPlugin {
    fn post_initialization(&self, app: &mut App) {
        let window = app.resource::<WgpuWindow>();
        let window_settings = app.resource::<Window>();
        let wgpu_render = Render::new(
            Arc::clone(window.device()),
            Arc::clone(window.queue()),
            window.texture_format(),
            window_settings.requested_surface_size,
            window_settings.minimal_surface_size,
        );
        app.insert_resource(wgpu_render);
        app.add_system(UpdatePhase::First, tick);
    }
}
