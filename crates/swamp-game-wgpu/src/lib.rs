/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use crate::prelude::Glyph;
use int_math::{URect, UVec2};
use monotonic_time_rs::{InstantMonotonicClock, Millis, MonotonicClock};
use std::cmp::{max, min};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use swamp_app::prelude::{
    App, AppReturnValue, ApplicationExit, Msg, Plugin, Re, ReM, ResourceStorage, UpdatePhase,
};
use swamp_app::prelude::{MessagesIterator, Resource};
use swamp_app::system_types::ReAll;
use swamp_asset_registry::AssetRegistry;
use swamp_assets::{AssetName, Id};
pub use swamp_basic_input::prelude::*;
use swamp_game::{Application, Assets};
use swamp_render_wgpu::prelude::Font;
use swamp_render_wgpu::{FixedAtlas, FontAndMaterial, MaterialRef};
use swamp_render_wgpu::{Material, Render};
use swamp_screen::WindowMessage;
use swamp_wgpu_window::WgpuWindow;
use tracing::info;

#[derive(Debug, Resource)]
pub struct GameWgpuSettings {
    pub virtual_size: UVec2,
}

pub struct WgpuAssets<'a> {
    //asset_loader: &'a mut AssetRegistry,
    resource_storage: &'a mut ResourceStorage,
    clock: InstantMonotonicClock,
}

impl<'a> WgpuAssets<'a> {
    pub fn new(resource_storage: &'a mut ResourceStorage) -> Self {
        Self {
            //            asset_loader,
            resource_storage,
            clock: InstantMonotonicClock::new(),
        }
    }
}

impl<'a> Assets for WgpuAssets<'a> {
    fn material_png(&mut self, name: impl Into<AssetName>) -> MaterialRef {
        let asset_loader = self
            .resource_storage
            .get_mut::<AssetRegistry>()
            .expect("should exist registry");
        asset_loader.load::<Material>(name.into().with_extension("png"))
    }

    fn frame_fixed_grid_material_png(
        &mut self,
        name: impl Into<AssetName>,
        grid_size: UVec2,
        texture_size: UVec2,
    ) -> FixedAtlas {
        let material_ref = self.material_png(name);

        FixedAtlas::new(grid_size, texture_size, material_ref)
    }

    fn bm_font(&mut self, name: impl Into<AssetName>) -> FontAndMaterial {
        let asset_name = name.into();
        let asset_loader = self
            .resource_storage
            .get_mut::<AssetRegistry>()
            .expect("should exist registry");
        let font_ref = asset_loader.load::<Font>(asset_name.clone().with_extension("fnt"));
        let material_ref = asset_loader.load::<Material>(asset_name.clone().with_extension("png"));

        FontAndMaterial {
            font_ref,
            material_ref,
        }
    }

    fn font(&self, font_ref: Id<Font>) -> Option<&Font> {
        let font_assets = self
            .resource_storage
            .get::<swamp_assets::Assets<Font>>()
            .expect("font assets should be a thing");

        font_assets.get(&font_ref)
    }

    fn text_glyphs(&self, text: &str, font_and_mat: &FontAndMaterial) -> Option<Vec<Glyph>> {
        if let Some(font) = self.font(font_and_mat.font_ref) {
            let glyphs = font.draw(text);
            Some(glyphs)
        } else {
            None
        }
    }

    fn now(&self) -> Millis {
        self.clock.now()
    }
}

#[derive(Resource)]
pub struct WgpuGame<G: Application> {
    game: G,
}

impl<G: Application> Debug for WgpuGame<G> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "WgpuGame")
    }
}

impl<G: Application> WgpuGame<G> {
    #[must_use]
    pub fn new(assets: &mut impl Assets) -> Self {
        Self {
            game: G::new(assets),
        }
    }

    pub fn inputs(&mut self, iter: MessagesIterator<InputMessage>) {
        for message in iter {
            match message {
                InputMessage::KeyboardInput(button_state, key_code) => {
                    info!("{:?}", key_code);
                    self.game.keyboard_input(*button_state, *key_code)
                }
                InputMessage::MouseInput(button_state, button) => {
                    self.game.mouse_input(*button_state, *button);
                }
                InputMessage::MouseWheel(scroll_delta, _touch_phase) => {
                    if let MouseScrollDelta::LineDelta(delta) = scroll_delta {
                        let game_scroll_y = (-delta.y as f32 * 120.0) as i16;
                        self.game.mouse_wheel(game_scroll_y);
                    }
                }
            }
        }
    }

    /*
    fn cursor_moved(&mut self, physical_position: UVec2) {
        let viewport = info.main_render.viewport();

        let relative_x = max(
            0,
            min(
                physical_position.x as i64 - viewport.position.x as i64,
                (viewport.size.x - 1) as i64,
            ),
        );

        let relative_y = max(
            0,
            min(
                physical_position.y as i64 - viewport.position.y as i64,
                (viewport.size.y - 1) as i64,
            ),
        );

        let clamped_to_viewport: UVec2 =
            UVec2::new(relative_x as u16, (viewport.size.y - 1) - relative_y as u16);
        let virtual_position_x = (clamped_to_viewport.x as u64
            * info.main_render.virtual_surface_size().x as u64)
            / viewport.size.x as u64;
        let virtual_position_y = (clamped_to_viewport.y as u64
            * info.main_render.virtual_surface_size().y as u64)
            / viewport.size.y as u64;

        let virtual_position = UVec2::new(virtual_position_x as u16, virtual_position_y as u16);
        self.cursor_moved_delayed = Some(virtual_position);
    }

     */

    pub fn cursor_moved(
        &mut self,
        physical_position: UVec2,
        viewport: URect,
        virtual_surface_size: UVec2,
    ) {
        let relative_x = max(
            0,
            min(
                physical_position.x as i64 - viewport.position.x as i64,
                (viewport.size.x - 1) as i64,
            ),
        );

        let relative_y = max(
            0,
            min(
                physical_position.y as i64 - viewport.position.y as i64,
                (viewport.size.y - 1) as i64,
            ),
        );

        let clamped_to_viewport: UVec2 = UVec2::new(relative_x as u16, relative_y as u16);

        let virtual_position_x =
            (clamped_to_viewport.x as u64 * virtual_surface_size.x as u64) / viewport.size.x as u64;

        let virtual_position_y =
            (clamped_to_viewport.y as u64 * virtual_surface_size.y as u64) / viewport.size.y as u64;

        let virtual_position = UVec2::new(virtual_position_x as u16, virtual_position_y as u16);
        self.game.cursor_moved(virtual_position)
    }

    pub fn mouse_move(&mut self, iter: MessagesIterator<WindowMessage>, wgpu_render: &Render) {
        for message in iter {
            match message {
                WindowMessage::CursorMoved(position) => self.cursor_moved(
                    *position,
                    wgpu_render.viewport(),
                    wgpu_render.virtual_surface_size(),
                ),
                WindowMessage::WindowCreated() => {}
                WindowMessage::Resized(_) => {}
            }
        }
    }

    pub fn tick(&mut self, assets: &mut impl Assets) {
        self.game.tick(assets);
    }

    pub fn render(
        &mut self,
        wgpu: &WgpuWindow,
        wgpu_render: &mut Render,
        materials: &swamp_assets::Assets<Material>,
        fonts: &swamp_assets::Assets<Font>,
    ) {
        self.game.render(wgpu_render);

        wgpu.render(wgpu_render.clear_color(), |render_pass| {
            wgpu_render.render(render_pass, materials, fonts)
        })
        .unwrap();
    }
}

pub struct GameWgpuPlugin<G: Application> {
    pub phantom_data: PhantomData<G>,
}
impl<G: Application> Default for GameWgpuPlugin<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Application> GameWgpuPlugin<G> {
    pub const fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

// TODO: add support for having tuple arguments to have maximum seven parameters
#[allow(clippy::too_many_arguments)]
pub fn tick<G: Application>(
    window: Re<WgpuWindow>,
    mut wgpu_render: ReM<Render>,
    materials: Re<swamp_assets::Assets<Material>>,
    fonts: Re<swamp_assets::Assets<Font>>,
    input_messages: Msg<InputMessage>,
    window_messages: Msg<WindowMessage>,
    mut all_resources: ReAll,
    mut internal_game: ReM<WgpuGame<G>>,
) {
    internal_game.inputs(input_messages.iter_previous());
    internal_game.mouse_move(window_messages.iter_previous(), &wgpu_render);

    let mut asset = WgpuAssets::new(&mut all_resources);
    internal_game.tick(&mut asset);
    if internal_game.game.wants_to_quit() {
        all_resources.insert(ApplicationExit {
            value: AppReturnValue::Value(0),
        });
    }
    internal_game.render(&window, &mut wgpu_render, &materials, &fonts);
}

impl<G: Application> Plugin for GameWgpuPlugin<G> {
    fn post_initialization(&self, app: &mut App) {
        let storage = app.resources_mut();
        //let asset_container = storage.get_mut::<AssetRegistry>();

        let mut asset = WgpuAssets::new(storage);
        let internal_game = WgpuGame::<G>::new(&mut asset);
        app.insert_resource(internal_game);

        app.add_system(UpdatePhase::Update, tick::<G>);
    }
}

/*
impl<T: Application> ApplicationWrap<T> {
    pub fn run(title: &str, virtual_surface_size: UVec2, suggested_physical_surface_size: UVec2) {
        let mut app = Self {
            app: None,
            virtual_surface_size,
            suggested_physical_surface_size,
            cursor_moved_delayed: None,
            wgpu_window: None,
        };

        let _ = swamp_window::WindowRunner::run_app(&mut app, title);
    }

    fn after(&mut self, wgpu_window: WgpuWindow) {
        let asset_reader = swamp_assets::get_platform_reader("assets/".to_string());
        // let physical_size = window.inner_size();
        let physical_size = PhysicalSize::new(10, 10);
        let mut render = Render::new(
            Arc::clone(wgpu_window.device()),
            Arc::clone(wgpu_window.queue()),
            wgpu_window.surface_config().format,
            (physical_size.width as u16, physical_size.height as u16).into(),
            self.virtual_surface_size,
            asset_reader,
        );

        let custom_app = T::new(&mut render);

        self.app = Some(AppInfo {
            window: wgpu_window,
            main_render: render,
            app: custom_app,
        });
    }
}

pub struct AppInfo<T: Application> {
    window: WgpuWindow,
    main_render: Render,
    app: T,
}

pub struct ApplicationWrap<T: Application> {
    app: Option<AppInfo<T>>,
    virtual_surface_size: UVec2,
    suggested_physical_surface_size: UVec2,
    cursor_moved_delayed: Option<UVec2>,
    wgpu_window: Option<WgpuWindow>,
}

impl<T: Application> AppHandler for ApplicationWrap<T> {
    fn min_size(&self) -> (u16, u16) {
        (self.virtual_surface_size.x, self.virtual_surface_size.y)
    }

    fn start_size(&self) -> (u16, u16) {
        (
            self.suggested_physical_surface_size.x,
            self.suggested_physical_surface_size.y,
        )
    }

    fn cursor_should_be_visible(&self) -> bool {
        self.app
            .as_ref()
            .map_or(true, |info| info.app.wants_cursor_visible())
    }

    fn redraw(&mut self) -> bool {
        if let Some(ref mut info) = self.app {
            if let Some(cursor_move_delayed) = self.cursor_moved_delayed {
                info.app.cursor_moved(cursor_move_delayed);
                self.cursor_moved_delayed = None;
            }
            info.app.tick(); // TODO: Fix a better tick rate
            if info.app.wants_to_quit() {
                return false;
            }

            info.app.render(&mut info.main_render);
            info.window
                .render(info.main_render.clear_color(), |render_pass| {
                    info.main_render.render(render_pass)
                })
                .expect("TODO: panic message");
        }

        true
    }

    fn got_focus(&mut self) {}

    fn lost_focus(&mut self) {}

    fn window_created(&mut self, window: Arc<Window>) {
        trace!("create window, boot up");
    }

    fn resized(&mut self, physical_size: dpi::PhysicalSize<u32>) {
        trace!("window resized (physical_size: {:?})", physical_size);
        if let Some(ref mut info) = self.app {
            info.window.resize(physical_size);
            info.main_render
                .resize((physical_size.width as u16, physical_size.height as u16).into());
        }
    }

    fn keyboard_input(&mut self, element_state: ElementState, physical_key: PhysicalKey) {
        if let Some(ref mut info) = self.app {
            if let PhysicalKey::Code(key_code) = physical_key {
                if let Ok(keycode) = key_code.try_into() {
                    info.app.keyboard_input(element_state.into(), keycode);
                }
            }
        }
    }

    fn cursor_entered(&mut self) {
        if let Some(ref mut info) = self.app {
            info.app.cursor_entered();
        }
    }

    fn cursor_left(&mut self) {
        if let Some(ref mut info) = self.app {
            info.app.cursor_left();
        }
    }

    fn cursor_moved(&mut self, physical_position: PhysicalPosition<f64>) {
        if let Some(ref mut info) = self.app {
            self.cursor_moved_delayed = Some(virtual_position);
        }
    }

    fn mouse_input(&mut self, element_state: ElementState, button: MouseButton) {
        if let Some(ref mut info) = self.app {
            if let Ok(converted_button) = button.try_into() {
                info.app.mouse_input(element_state.into(), converted_button);
            }
        }
    }

    fn mouse_wheel(&mut self, delta: MouseScrollDelta, _touch_phase: TouchPhase) {
        if let Some(ref mut info) = self.app {
            if let MouseScrollDelta::LineDelta(.., y) = delta {
                info.app.mouse_wheel((-y * 120.0) as i16);
            }
        }
    }

    fn mouse_motion(&mut self, delta: (f64, f64)) {
        if let Some(ref mut info) = self.app {
            let factor = 65.0;
            let converted = Vec2::new((delta.0 * factor) as i16, (-delta.1 * factor) as i16);
            info.app.mouse_motion(converted);
        }
    }

    fn touch(&mut self, _touch: Touch) {
        // TODO:
    }

    fn scale_factor_changed(&mut self, scale_factor: f64, mut inner_size_writer: InnerSizeWriter) {
        if let Some(ref mut info) = self.app {
            if let Some(new_inner) = info.app.scale_factor_changed(scale_factor) {
                let physical_size = PhysicalSize::new(new_inner.x as u32, new_inner.y as u32);
                inner_size_writer.request_inner_size(physical_size).unwrap();
            }
        }
    }
}
*/
