/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use crate::logic::GameLogic;
use crate::{ApplicationAudio, ApplicationLogic};
use limnus_app::prelude::{App, Plugin};
use limnus_audio_mixer::{AudioMixer, StereoSample};
use limnus_default_stages::FixedUpdate;
use limnus_local_resource::prelude::LocalResource;
use limnus_resource::ResourceStorage;
use limnus_system_params::{LoRe, LoReM, Re};
use monotonic_time_rs::{InstantMonotonicClock, MonotonicClock};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use swamp_game_assets::GameAssets;
use swamp_game_audio::GameAudio;
use tracing::trace;

impl<A: ApplicationAudio<L>, L: ApplicationLogic> Debug for GameAudioRender<A, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameAudioRender")
    }
}

pub fn advanced_game_audio_tick<R: ApplicationAudio<L>, L: ApplicationLogic>(
    mut audio_render: LoReM<GameAudioRender<R, L>>,
    logic: LoRe<GameLogic<L>>,
    mut audio_mixer: LoReM<AudioMixer>,
    stereo_samples: Re<limnus_assets::Assets<StereoSample>>,
) {
    let mut game_audio = GameAudio::new(&mut audio_mixer, &stereo_samples);
    audio_render.audio.audio(&mut game_audio, &logic.logic);
}

#[derive(LocalResource)]
pub struct GameAudioRender<A: ApplicationAudio<L>, L: ApplicationLogic> {
    audio: A,
    _phantom: PhantomData<L>,
}

impl<A: ApplicationAudio<L>, L: ApplicationLogic> GameAudioRender<A, L> {
    pub fn new(all_resources: &mut ResourceStorage) -> Self {
        let clock = InstantMonotonicClock::new();
        let mut assets = GameAssets::new(all_resources, clock.now());
        Self {
            audio: A::new(&mut assets),
            _phantom: PhantomData,
        }
    }
}

#[derive(Default)]
pub struct GameAudioRenderPlugin<A: ApplicationAudio<L>, L: ApplicationLogic> {
    _phantom: PhantomData<(A, L)>,
}

impl<A: ApplicationAudio<L>, L: ApplicationLogic> GameAudioRenderPlugin<A, L> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<A: ApplicationAudio<L>, L: ApplicationLogic> Plugin for GameAudioRenderPlugin<A, L> {
    fn post_initialization(&self, app: &mut App) {
        trace!("GameAudioRenderPlugin startup");
        let all_resources = app.resources_mut();
        let internal_audio = GameAudioRender::<A, L>::new(all_resources);
        app.insert_local_resource(internal_audio);

        app.add_system(FixedUpdate, advanced_game_audio_tick::<A, L>);
    }
}
