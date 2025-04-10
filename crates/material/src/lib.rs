/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/mireforge/mireforge
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use limnus_app::prelude::{App, Plugin};
use limnus_asset_id::{AssetName, RawWeakId};
use limnus_asset_registry::AssetRegistry;
use limnus_assets::Assets;
use limnus_assets_loader::{AssetLoader, ConversionError, WrappedAssetLoaderRegistry};
use limnus_local_resource::LocalResourceStorage;
use limnus_resource::ResourceStorage;
use limnus_wgpu_window::BasicDeviceInfo;
use mireforge_render_wgpu::{Render, Texture};
use tracing::debug;

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        {
            let registry = app.resource_mut::<WrappedAssetLoaderRegistry>();
            let loader = MaterialWgpuProcessor::new();

            registry.value.lock().unwrap().register_loader(loader);
        }

        app.insert_resource(Assets::<Texture>::default());
    }
}

#[derive(Default)]
pub struct MaterialWgpuProcessor;

impl MaterialWgpuProcessor {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl AssetLoader for MaterialWgpuProcessor {
    type AssetType = Texture;

    fn convert_and_insert(
        &self,
        id: RawWeakId,
        octets: &[u8],
        resources: &mut ResourceStorage,
        local_resources: &mut LocalResourceStorage,
    ) -> Result<(), ConversionError> {
        let device_info = local_resources.fetch::<BasicDeviceInfo>();

        let name: AssetName;
        {
            let asset_container = resources.fetch::<AssetRegistry>();
            name = asset_container
                .name_raw(id)
                .expect("should know about this Id");
        }

        debug!(?name, "convert from png");
        let dynamic_image = image::load_from_memory_with_format(octets, image::ImageFormat::Png)
            .expect("Failed to load image");

        debug!(?name, "creating texture");
        let wgpu_texture = mireforge_wgpu_sprites::load_texture_from_memory(
            &device_info.device,
            &device_info.queue,
            dynamic_image,
            name.value(),
        );

        {
            let mireforge_render_wgpu = resources.fetch_mut::<Render>();
            let wgpu_material =
                mireforge_render_wgpu.texture_resource_from_texture(&wgpu_texture, name.value());

            let image_assets = resources.fetch_mut::<Assets<Texture>>();
            image_assets.set_raw(id, wgpu_material);
            debug!(?id, ?name, "texture inserted");
        }

        Ok(())
    }
}
