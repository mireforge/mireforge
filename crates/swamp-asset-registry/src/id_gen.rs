/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::idx_gen::IndexAllocator;
use crate::TypeIdMap;
use std::any::TypeId;
use std::collections::HashMap;
use swamp_assets::{Asset, Id, RawAssetId, RawAssetIdWithTypeId};

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
}

impl IdAssigner {
    pub fn new() -> Self {
        Self {
            allocators: TypeIdMap::default(),
        }
    }

    pub fn allocate<T: Asset>(&mut self) -> Id<T> {
        let allocator = get_mut_or_create(&mut self.allocators, TypeId::of::<T>(), || {
            IndexAllocator::new()
        });

        let (index, generation) = allocator.create();

        let raw_id = RawAssetId {
            generation,
            index: index as u16,
        };

        RawAssetIdWithTypeId::with_asset_type::<T>(raw_id).into()
    }

    pub fn remove<T: Asset>(&mut self, id: Id<T>) {
        let allocator = self
            .allocators
            .get_mut(&TypeId::of::<T>())
            .expect("missing asset allocator");
        let raw_id_with_type_id: RawAssetIdWithTypeId = id.into();
        let raw_id: RawAssetId = raw_id_with_type_id.into();
        allocator.remove((raw_id.index as usize, raw_id.generation));
    }
}
