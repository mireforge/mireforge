use crate::logic::GameLogic;
use crate::{ApplicationLogic, ApplicationRender};
use limnus_app::prelude::{App, Plugin};
use limnus_local_resource::prelude::LocalResource;
use limnus_resource::ResourceStorage;
use limnus_system_params::{LoRe, LoReM, Re, ReM};
use limnus_system_runner::UpdatePhase;
use limnus_wgpu_window::WgpuWindow;
use monotonic_time_rs::{InstantMonotonicClock, Millis, MonotonicClock};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use swamp_font::Font;
use swamp_game_assets::GameAssets;
use swamp_render_wgpu::{Material, Render};
use tracing::debug;

#[derive(LocalResource)]
pub struct GameRenderer<R: ApplicationRender<L>, L: ApplicationLogic> {
    renderer: R,
    clock: InstantMonotonicClock,
    _phantom: PhantomData<L>,
}

impl<R: ApplicationRender<L>, L: ApplicationLogic> Debug for GameRenderer<R, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameAudioRender")
    }
}

impl<R: ApplicationRender<L>, L: ApplicationLogic> GameRenderer<R, L> {
    #[must_use]
    pub fn new(all_resources: &mut ResourceStorage) -> Self {
        let clock = InstantMonotonicClock::new();
        let mut assets = GameAssets::new(all_resources, clock.now());
        let renderer = R::new(&mut assets);

        Self {
            renderer,
            clock,
            _phantom: PhantomData,
        }
    }

    pub fn render(
        &mut self,
        logic: &L,
        wgpu: &WgpuWindow,
        wgpu_render: &mut Render,
        materials: &limnus_assets::Assets<Material>,
        fonts: &limnus_assets::Assets<Font>,
        now: Millis,
    ) {
        wgpu_render.set_now(now);
        self.renderer.render(wgpu_render, logic);

        wgpu.render(wgpu_render.clear_color(), |render_pass| {
            wgpu_render.render(render_pass, materials, fonts, now)
        })
        .unwrap();
    }
}

pub fn advanced_game_render_tick<R: ApplicationRender<L>, L: ApplicationLogic>(
    mut game_render: LoReM<GameRenderer<R, L>>,
    logic: LoRe<GameLogic<L>>,
    materials: Re<limnus_assets::Assets<Material>>,
    fonts: Re<limnus_assets::Assets<Font>>,
    window: Re<WgpuWindow>,
    mut wgpu_render: ReM<Render>,
) {
    let now = game_render.clock.now();
    game_render.render(
        &logic.logic,
        &window,
        &mut wgpu_render,
        &materials,
        &fonts,
        now,
    );
}

/*

let window = app.resource::<WgpuWindow>();
let window_settings = app.resource::<Window>();
let wgpu_render = Render::new(
Arc::clone(window.device()),
Arc::clone(window.queue()),
window.texture_format(),
window_settings.requested_surface_size,
window_settings.minimal_surface_size,
Millis::new(0),
);
*/

#[derive(Default)]
pub struct GameRendererPlugin<R: ApplicationRender<L>, L: ApplicationLogic> {
    _phantom: PhantomData<(R, L)>,
}

impl<A: ApplicationRender<L>, L: ApplicationLogic> GameRendererPlugin<A, L> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<R: ApplicationRender<L>, L: ApplicationLogic> Plugin for GameRendererPlugin<R, L> {
    fn post_initialization(&self, app: &mut App) {
        debug!("calling WgpuGame::new()");
        let all_resources = app.resources_mut();

        let game_renderer = GameRenderer::<R, L>::new(all_resources);
        app.insert_local_resource(game_renderer);

        app.add_system(UpdatePhase::Update, advanced_game_render_tick::<R, L>);
    }
}
