/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use swamp_assets::prelude::*;
use swamp_assets_loader::{AssetLoader, ConversionError};
use swamp_resource::ResourceStorage;

#[derive(Asset, Debug)]
pub struct TestImage {
    pub width: u32,
}

pub struct ImageSettings {
    pub use_alpha: bool,
}

pub struct TestImageLoader {
    pub settings: ImageSettings,
}

impl TestImageLoader {
    pub fn new(settings: ImageSettings) -> Self {
        Self { settings }
    }
}

impl AssetLoader for TestImageLoader {
    type AssetType = TestImage;

    fn convert_and_insert(
        &self,
        id: RawWeakId,
        _octets: &[u8],
        resources: &mut ResourceStorage,
    ) -> Result<(), ConversionError> {
        // TODO: do png conversion
        let image_assets = resources.fetch_mut::<Assets<TestImage>>();

        image_assets.set_raw(id, TestImage { width: 320 });

        Ok(())
    }
}

#[test_log::test]
fn register() {

    /*
    let mut registry = AssetLoaderRegistry::new();

    let image_loader = TestImageLoader::new(ImageSettings { use_alpha: true });
    registry.register_loader(image_loader);
    let mut image_assets = Assets::<TestImage>::new(1024);
    let raw_id =  RawAssetId {
        generation: 0,
        index: 0,
    };


    let mut world = ResourceStorage::new();
    world.insert(image_assets);

    let raw_id_with_type = RawAssetIdWithTypeId::from(typed_id.clone());

    {
        let immutable_assets = world.get::<Assets<TestImage>>().unwrap();
        assert!(!immutable_assets.contains(&typed_id));
    }

    registry
        .convert_and_insert(raw_id_with_type, &[1, 2, 3], &mut world)
        .expect("Failed to convert asset");

    let immutable_assets = world.fetch::<Assets<TestImage>>();
    let image_again = immutable_assets.fetch(&typed_id);

    assert_eq!(image_again.width, 320);
     */
}
