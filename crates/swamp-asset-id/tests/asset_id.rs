/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use message_channel::Channel;
use swamp_asset_id::owner::DropMessage;
use swamp_asset_id::{Asset, AssetName, Id, RawAssetId, RawWeakId};

#[derive(Debug)]
pub struct TestAsset {
    pub value: i32,
}

impl Asset for TestAsset {}

#[test]
fn test() {
    let (sender, receiver) = Channel::<DropMessage>::create();

    let raw_id = RawAssetId::new(0, 0);

    assert!(receiver.try_recv().is_none());
    let typed_id = Id::<TestAsset>::new(raw_id, sender, AssetName::new("test"));
    let raw_type_id: &RawWeakId = &(&typed_id).into();
    assert!(receiver.try_recv().is_none());
    drop(typed_id);
    assert_eq!(receiver.try_recv().unwrap().asset_id, *raw_type_id);
    assert!(receiver.try_recv().is_none());
}
