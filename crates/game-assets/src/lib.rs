/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use int_math::UVec2;

use limnus_asset_id::{AssetName, Id};
use limnus_asset_registry::AssetRegistry;
use limnus_audio_mixer::{StereoSample, StereoSampleRef};
use limnus_resource::ResourceStorage;
use mireforge_font::{Font, GlyphDraw};
use mireforge_render_wgpu::{
    FixedAtlas, FontAndMaterial, Material, MaterialBase, MaterialKind, MaterialRef,
    NineSliceAndMaterial, Slices, Texture, TextureRef,
};
use monotonic_time_rs::Millis;
use std::fmt::Debug;
use std::sync::Arc;

pub trait Assets {
    #[must_use]
    fn now(&self) -> Millis;

    #[must_use]
    fn texture_png(&mut self, name: impl Into<AssetName>) -> TextureRef;

    #[must_use]
    fn material_png(&mut self, name: impl Into<AssetName>) -> MaterialRef;

    #[must_use]
    fn material_alpha_mask(
        &mut self,
        name: impl Into<AssetName>,
        mask: impl Into<AssetName>,
    ) -> MaterialRef;

    #[must_use]
    fn light_material_png(&mut self, name: impl Into<AssetName>) -> MaterialRef;

    #[must_use]
    fn frame_fixed_grid_material_png(
        &mut self,
        name: impl Into<AssetName>,
        grid_size: UVec2,
        texture_size: UVec2,
    ) -> FixedAtlas;

    #[must_use]
    fn nine_slice_material_png(
        &mut self,
        name: impl Into<AssetName>,
        slices: Slices,
    ) -> NineSliceAndMaterial;

    #[must_use]
    fn bm_font(&mut self, name: impl Into<AssetName>) -> FontAndMaterial;

    #[must_use]
    fn bm_font_txt(&mut self, name: impl Into<AssetName>) -> FontAndMaterial;

    #[must_use]
    fn text_glyphs(&self, text: &str, font_and_mat: &FontAndMaterial) -> Option<GlyphDraw>;

    #[must_use]
    fn font(&self, font_ref: &Id<Font>) -> Option<&Font>;
    #[must_use]
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
    pub const fn new(resource_storage: &'a mut ResourceStorage, now: Millis) -> Self {
        Self {
            now,
            resource_storage,
        }
    }
}

impl Assets for GameAssets<'_> {
    fn now(&self) -> Millis {
        self.now
    }

    fn texture_png(&mut self, name: impl Into<AssetName>) -> TextureRef {
        let asset_loader = self
            .resource_storage
            .get_mut::<AssetRegistry>()
            .expect("should exist registry");

        let texture_id = asset_loader.load::<Texture>(name.into().with_extension("png"));

        TextureRef::from(texture_id)
    }

    fn material_png(&mut self, name: impl Into<AssetName>) -> MaterialRef {
        let asset_loader = self
            .resource_storage
            .get_mut::<AssetRegistry>()
            .expect("should exist registry");

        let texture_ref = asset_loader.load::<Texture>(name.into().with_extension("png"));

        let material = Material {
            base: MaterialBase {
                //pipeline: self.renderer().normal_sprite_pipeline.clone(),
            },
            kind: MaterialKind::NormalSprite {
                primary_texture: texture_ref,
            },
        };

        Arc::new(material)
    }

    fn material_alpha_mask(
        &mut self,
        name: impl Into<AssetName>,
        mask: impl Into<AssetName>,
    ) -> MaterialRef {
        let asset_loader = self
            .resource_storage
            .get_mut::<AssetRegistry>()
            .expect("should exist registry");
        let diffuse_texture_id = asset_loader.load::<Texture>(name.into().with_extension("png"));
        let alpha_mask_texture_id = asset_loader.load::<Texture>(mask.into().with_extension("png"));
        let material = Material {
            base: MaterialBase {},
            kind: MaterialKind::AlphaMasker {
                primary_texture: diffuse_texture_id,
                alpha_texture: alpha_mask_texture_id,
            },
        };

        Arc::new(material)
    }

    fn light_material_png(&mut self, name: impl Into<AssetName>) -> MaterialRef {
        let asset_loader = self
            .resource_storage
            .get_mut::<AssetRegistry>()
            .expect("should exist registry");

        let texture_ref = asset_loader.load::<Texture>(name.into().with_extension("png"));

        let material = Material {
            base: MaterialBase {
                //pipeline: self.renderer().normal_sprite_pipeline.clone(),
            },
            kind: MaterialKind::LightAdd {
                primary_texture: texture_ref,
            },
        };

        Arc::new(material)
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

    fn nine_slice_material_png(
        &mut self,
        name: impl Into<AssetName>,
        slices: Slices,
    ) -> NineSliceAndMaterial {
        let material_ref = self.material_png(name);

        NineSliceAndMaterial {
            slices,
            material_ref,
        }
    }

    fn bm_font(&mut self, name: impl Into<AssetName>) -> FontAndMaterial {
        let asset_name = name.into();
        let asset_loader = self
            .resource_storage
            .get_mut::<AssetRegistry>()
            .expect("should exist registry");
        let font_ref = asset_loader.load::<Font>(asset_name.clone().with_extension("fnt"));
        let texture_id = asset_loader.load::<Texture>(asset_name.clone().with_extension("png"));

        let material = Material {
            base: MaterialBase {
                //pipeline: self.renderer().normal_sprite_pipeline.clone(),
            },
            kind: MaterialKind::NormalSprite {
                primary_texture: texture_id,
            },
        };

        FontAndMaterial {
            font_ref,
            material_ref: Arc::new(material),
        }
    }

    fn bm_font_txt(&mut self, name: impl Into<AssetName>) -> FontAndMaterial {
        let asset_name = name.into();
        let asset_loader = self
            .resource_storage
            .get_mut::<AssetRegistry>()
            .expect("should exist registry");
        let font_ref = asset_loader.load::<Font>(asset_name.clone().with_extension("txt.fnt"));
        let texture_id = asset_loader.load::<Texture>(asset_name.clone().with_extension("png"));

        let material = Material {
            base: MaterialBase {
                //pipeline: self.renderer().normal_sprite_pipeline.clone(),
            },
            kind: MaterialKind::NormalSprite {
                primary_texture: texture_id,
            },
        };

        FontAndMaterial {
            font_ref,
            material_ref: Arc::new(material),
        }
    }

    fn text_glyphs(&self, text: &str, font_and_mat: &FontAndMaterial) -> Option<GlyphDraw> {
        match self.font(&font_and_mat.font_ref) {
            Some(font) => {
                let glyphs = font.draw(text);
                Some(glyphs)
            }
            _ => None,
        }
    }

    fn font(&self, font_ref: &Id<Font>) -> Option<&Font> {
        let font_assets = self
            .resource_storage
            .get::<limnus_assets::Assets<Font>>()
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
