/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use crate::prelude::*;
use sparse_slot::SparseSlot;
use std::fmt::{Debug, Formatter};
use swamp_resource::prelude::*;
use tracing::debug;

#[derive(Resource)]
pub struct Assets<A: Asset> {
    storage: SparseSlot<A>,
}

impl<A: Asset> Debug for Assets<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Assets capacity: {}", self.storage.len())
    }
}

impl<A: Asset> Default for Assets<A> {
    fn default() -> Self {
        Self {
            storage: SparseSlot::<A>::new(1024),
        }
    }
}

impl<A: Asset> Assets<A> {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            storage: SparseSlot::new(capacity),
        }
    }

    /// # Panics
    pub fn set(&mut self, id: &Id<A>, asset: A) {
        debug!("setting resource {id} of asset: {asset:?}");
        self.storage
            .try_set(to_slot_map_id(id), asset)
            .expect("internal error");
    }

    pub fn set_raw(&mut self, id: RawWeakId, asset: A) {
        debug!("setting resource {id} of asset: {asset:?}");
        self.storage
            .try_set(to_slot_map_id_from_raw(id), asset)
            .expect("internal error");
    }

    pub fn remove(&mut self, id: &Id<A>) {
        self.storage.remove(to_slot_map_id(id));
    }

    #[must_use]
    pub fn get(&self, id: &Id<A>) -> Option<&A> {
        self.storage.get(to_slot_map_id(id))
    }

    pub fn get_weak(&self, weak_id: WeakId<A>) -> Option<&A> {
        self.storage.get(to_slot_map_id_from_weak(weak_id))
    }

    /// # Panics
    /// if id is missing
    #[must_use]
    pub fn fetch(&self, id: &Id<A>) -> &A {
        self.storage.get(to_slot_map_id(id)).unwrap()
    }

    #[must_use]
    pub fn get_mut(&mut self, id: &Id<A>) -> Option<&mut A> {
        self.storage.get_mut(to_slot_map_id(id))
    }

    #[must_use]
    pub fn contains(&self, id: &Id<A>) -> bool {
        self.get(id).is_some()
    }

    // TODO:
    /*
        pub fn iter(&self) -> impl Iterator<Item=(Id<A>, &A)> {
            self.storage.iter().map(|(id, asset)| {
                (
                    Id {
                        raw_id: RawAssetId {
                            generation: id.generation,
                            index: id.index as u16,
                        },

                        _phantom_data: PhantomData,
                    },
                    asset,
                )
            })
        }
    */
    #[must_use]
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
}

fn to_slot_map_id<A: Asset>(typed_id: &Id<A>) -> sparse_slot::Id {
    let raw_id_type: RawWeakId = typed_id.into();

    to_slot_map_id_from_raw(raw_id_type)
}

fn to_slot_map_id_from_weak<A: Asset>(raw_id_type: WeakId<A>) -> sparse_slot::Id {
    let raw_id: RawWeakId = raw_id_type.into();

    to_slot_map_id_from_raw(raw_id)
}

fn to_slot_map_id_from_raw(raw_id_type: RawWeakId) -> sparse_slot::Id {
    let raw_id: RawAssetId = raw_id_type.into();

    sparse_slot::Id::new(raw_id.index as usize, raw_id.generation)
}
