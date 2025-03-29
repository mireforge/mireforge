/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::logic::GameLogic;
use crate::{ApplicationLogic, ApplicationRender};
use limnus_app::prelude::{App, Plugin};
use limnus_default_stages::RenderUpdate;
use limnus_local_resource::prelude::LocalResource;
use limnus_resource::ResourceStorage;
use limnus_system_params::{LoRe, LoReM, ReM};
use mireforge_game_assets::GameAssets;
use mireforge_render_wgpu::Render;
use monotonic_time_rs::{InstantMonotonicClock, Millis, MonotonicClock};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use tracing::trace;

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

    pub fn render(&mut self, logic: &L, wgpu_render: &mut Render, now: Millis) {
        wgpu_render.set_now(now);
        self.renderer.render(wgpu_render, logic);
    }
}

pub fn advanced_game_render_tick<R: ApplicationRender<L>, L: ApplicationLogic>(
    mut game_render: LoReM<GameRenderer<R, L>>,
    logic: LoRe<GameLogic<L>>,
    mut wgpu_render: ReM<Render>,
) {
    let now = game_render.clock.now();
    game_render.render(&logic.logic, &mut wgpu_render, now);
}

#[derive(Default)]
pub struct GameRendererPlugin<R: ApplicationRender<L>, L: ApplicationLogic> {
    _phantom: PhantomData<(R, L)>,
}

impl<A: ApplicationRender<L>, L: ApplicationLogic> GameRendererPlugin<A, L> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<R: ApplicationRender<L>, L: ApplicationLogic> Plugin for GameRendererPlugin<R, L> {
    fn post_initialization(&self, app: &mut App) {
        trace!("GameRendererPlugin startup");
        let all_resources = app.resources_mut();

        let game_renderer = GameRenderer::<R, L>::new(all_resources);
        app.insert_local_resource(game_renderer);

        app.add_system(RenderUpdate, advanced_game_render_tick::<R, L>);
    }
}
