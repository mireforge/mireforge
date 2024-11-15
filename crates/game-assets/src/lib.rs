use int_math::UVec2;
use monotonic_time_rs::Millis;
use std::fmt::Debug;
use swamp_asset_registry::AssetRegistry;
use swamp_assets::prelude::{AssetName, Id};
use swamp_audio_sample::{StereoSample, StereoSampleRef};
use swamp_render_wgpu::prelude::{Font, Glyph};
use swamp_render_wgpu::{FixedAtlas, FontAndMaterial, Material, MaterialRef};
use swamp_resource::ResourceStorage;

pub trait Assets {
    fn now(&self) -> Millis;

    fn material_png(&mut self, name: impl Into<AssetName>) -> MaterialRef;

    fn frame_fixed_grid_material_png(
        &mut self,
        name: impl Into<AssetName>,
        grid_size: UVec2,
        texture_size: UVec2,
    ) -> FixedAtlas;

    fn bm_font(&mut self, name: impl Into<AssetName>) -> FontAndMaterial;

    fn text_glyphs(&self, text: &str, font_and_mat: &FontAndMaterial) -> Option<Vec<Glyph>>;

    fn font(&self, font_ref: &Id<Font>) -> Option<&Font>;
    fn audio_sample_wav(&mut self, name: impl Into<AssetName>) -> StereoSampleRef;
}

pub struct GameAssets<'a> {
    now: Millis,
    resource_storage: &'a mut ResourceStorage,
}

impl Debug for GameAssets<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "assets")
    }
}

impl<'a> GameAssets<'a> {
    pub fn new(resource_storage: &'a mut ResourceStorage, now: Millis) -> Self {
        Self {
            resource_storage,
            now,
        }
    }
}

impl Assets for GameAssets<'_> {
    fn now(&self) -> Millis {
        self.now
    }

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

    fn text_glyphs(&self, text: &str, font_and_mat: &FontAndMaterial) -> Option<Vec<Glyph>> {
        if let Some(font) = self.font(&font_and_mat.font_ref) {
            let glyphs = font.draw(text);
            Some(glyphs)
        } else {
            None
        }
    }

    fn font(&self, font_ref: &Id<Font>) -> Option<&Font> {
        let font_assets = self
            .resource_storage
            .get::<swamp_assets::Assets<Font>>()
            .expect("font assets should be a thing");

        font_assets.get(font_ref)
    }

    fn audio_sample_wav(&mut self, name: impl Into<AssetName>) -> StereoSampleRef {
        let asset_loader = self
            .resource_storage
            .get_mut::<AssetRegistry>()
            .expect("should exist registry");
        asset_loader.load::<StereoSample>(name.into().with_extension("wav"))
    }
}
