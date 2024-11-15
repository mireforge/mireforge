/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
/*

use std::marker::PhantomData;
use swamp_app::prelude::*;
use tracing::info;

pub struct SomePlugin;

impl Plugin for SomePlugin {
    fn build(&self, app: &mut App) {
        let mut ticking = Ticking { tick_count: 0 };
        let mut ticking2 = Ticking { tick_count: 0 };
        // Add the functions as a tuple of closures
        app.add_functions(
            UpdatePhase::First,
            (
                move || {
                    ticking.tick();
                },
                move || {
                    ticking2.another_tick();
                },
            ),
        );
    }
}

pub struct Ticking {
    tick_count: u32,
}

impl Ticking {
    fn tick(&mut self) {
        info!("Ticking::tick have been ticked!");
        self.tick_count += 1;
    }

    fn another_tick(&mut self) {
        info!("Ticking::another_tick have been ticked!");
        self.tick_count += 1;
    }
}

pub struct AnotherPlugin;

impl Plugin for AnotherPlugin {
    fn build(&self, app: &mut App) {
        app.add_functions(UpdatePhase::Update, (fn1, fn2));
    }
}

fn fn1() {
    info!("fn1 has been called!")
}

fn fn2() {
    info!("fn2 has been called!")
}

#[test]
fn app_that_returns() {
    let value = App::new()
        .set_runner(|_app| AppReturnValue::Value(1))
        .add_plugins(SomePlugin)
        .run();

    assert_eq!(value, AppReturnValue::Value(1));
}

#[test]
fn add_multiple_plugins() {
    let value = App::new()
        .set_runner(|_app| AppReturnValue::Value(1))
        .add_plugins((SomePlugin, AnotherPlugin))
        .run();

    assert_eq!(value, AppReturnValue::Value(1));
}

#[test_log::test]
fn run_tickers() {
    let value = App::new()
        .set_runner(|mut app| {
            app.update();
            AppReturnValue::Value(2)
        })
        .add_plugins((SomePlugin, AnotherPlugin))
        .run();

    assert_eq!(value, AppReturnValue::Value(2));
}

fn start_loop(mut app: App) -> AppReturnValue {
    for _ in 0..3 {
        app.update();
    }

    AppReturnValue::Value(99)
}

struct LoopRunnerPlugin;

impl Plugin for LoopRunnerPlugin {
    fn build(&self, app: &mut App) {
        app.set_runner(start_loop);
    }
}

#[test_log::test]
fn loop_runner_tickers() {
    let value = App::new()
        .add_plugins((LoopRunnerPlugin, SomePlugin, AnotherPlugin))
        .run();

    assert_eq!(value, AppReturnValue::Value(99));
}

#[derive(Debug, Resource)]
pub struct Secret {
    pub value: u8,
}

pub struct ResourceAwarePlugin;

impl Plugin for ResourceAwarePlugin {
    fn build(&self, app: &mut App) {
        {
            let found_secret = app.resource::<Secret>();

            assert_eq!(found_secret.value, 99);
        }

        app.insert_resource(Secret { value: 100 });
    }
}

#[test_log::test]
fn with_resources() {
    let value = App::new()
        .insert_resource(Secret { value: 99 })
        .add_plugins((LoopRunnerPlugin, ResourceAwarePlugin))
        .run();

    assert_eq!(value, AppReturnValue::Value(99));
}

pub trait Gfx {
    fn draw_sprite(x: i32, y: i32);
}

pub trait GameApplication: Send + Sync + 'static {
    fn new() -> Self
    where
        Self: Sized;
    fn tick(&mut self);

    fn render(&mut self, gfx: impl Gfx);
}

pub struct GameApplicationResource {
    pub game: Box<dyn GameApplication>,
}

#[derive(Debug, Default)]
pub struct GamePlugin<T> {
    phantom_data: PhantomData<T>,
}

pub fn tick_game(&mut self) {
    if let Some(instance) = self.instance.as_mut() {
        instance.tick();
    } else {
        info!("creating game!");
        self.instance = Some(T::new());
    }
}

impl<T: GameApplication> Plugin for GamePlugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(T::new());
        app.add_system(UpdatePhase::Update, || tick_game.update());
    }
}

#[derive(Default)]
pub struct MyGame {}
impl GameApplication for MyGame {
    fn new() -> Self {
        Self {}
    }
    fn tick(&mut self) {
        info!("my game is ticked!")
    }

    fn render(&mut self, _gfx: impl Gfx) {
        info!("my game is rendering!")
    }
}

#[test_log::test]
fn small_game() {
    let value = App::new()
        .insert_resource(Secret { value: 99 })
        .add_plugins((LoopRunnerPlugin, GamePlugin::<MyGame>::default()))
        .run();

    assert_eq!(value, AppReturnValue::Value(99));
}
*/
