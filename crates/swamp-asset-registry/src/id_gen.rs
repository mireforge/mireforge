/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::idx_gen::IndexAllocator;
use crate::TypeIdMap;
use message_channel::Sender;
use std::any::TypeId;
use std::collections::HashMap;
use swamp_assets::prelude::{Asset, AssetName, DropMessage, Id, RawAssetId, RawWeakId};

fn get_mut_or_create<K, V, F>(map: &mut HashMap<K, V>, key: K, create: F) -> &mut V
where
    K: std::hash::Hash + Eq,
    F: FnOnce() -> V,
{
    map.entry(key).or_insert_with(create)
}

#[derive(Debug)]
pub struct IdAssigner {
    allocators: TypeIdMap<IndexAllocator>,
    sender: Sender<DropMessage>,
}

impl IdAssigner {
    pub fn new(sender: Sender<DropMessage>) -> Self {
        Self {
            allocators: TypeIdMap::default(),
            sender,
        }
    }

    pub fn allocate<T: Asset>(&mut self, asset_name: AssetName) -> Id<T> {
        let allocator = get_mut_or_create(&mut self.allocators, TypeId::of::<T>(), || {
            IndexAllocator::new()
        });

        let (index, generation) = allocator.create();

        let raw_id = RawAssetId {
            generation,
            index: index as u16,
        };

        Id::<T>::new(raw_id, self.sender.clone(), asset_name)
    }

    pub fn remove<T: Asset>(&mut self, id: Id<T>) {
        let allocator = self
            .allocators
            .get_mut(&TypeId::of::<T>())
            .expect("missing asset allocator");
        let raw_id_with_type_id: RawWeakId = (&id).into();
        let raw_id: RawAssetId = raw_id_with_type_id.into();
        allocator.remove((raw_id.index as usize, raw_id.generation));
    }
}
