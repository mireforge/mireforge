/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use sparse_slot::SparseSlot;
use std::any::TypeId;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::path::PathBuf;
use swamp_resource::prelude::*;
use tracing::debug;

pub trait Asset: 'static + Debug + Send + Sync {}

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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct RawAssetId {
    pub generation: u16,
    pub index: u16,
}

impl Into<RawAssetId> for RawAssetIdWithTypeId {
    fn into(self) -> RawAssetId {
        self.raw_id
    }
}

/*
impl Into (usize, usize) for RawAssetId {
    fn into(self) -> (usize, usize) {
        (self.index as usize, self.generation as usize)
    }
}
*/

impl Display for RawAssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.index, self.generation)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RawAssetIdWithTypeId {
    raw_id: RawAssetId,
    pub type_id: TypeId,
}

impl<A: Asset> From<Id<A>> for RawAssetIdWithTypeId {
    fn from(id: Id<A>) -> Self {
        Self {
            raw_id: id.raw_id,
            type_id: TypeId::of::<A>(),
        }
    }
}

impl RawAssetIdWithTypeId {
    #[must_use]
    pub fn with_asset_type<A: Asset>(id: RawAssetId) -> Self {
        Self {
            raw_id: id,
            type_id: TypeId::of::<A>(),
        }
    }
}

impl Display for RawAssetIdWithTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{:?}", self.raw_id, self.type_id)
    }
}

#[derive(Debug)]
pub struct Id<A: Asset> {
    raw_id: RawAssetId,
    _phantom_data: PhantomData<A>,
}

impl<A: Asset> Copy for Id<A> {}

impl<A: Asset> Clone for Id<A> {
    fn clone(&self) -> Self {
        Self {
            raw_id: self.raw_id,
            _phantom_data: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.raw_id = source.raw_id;
    }
}

impl<A: Asset> Eq for Id<A> {}

impl<A: Asset> PartialEq<Self> for Id<A> {
    fn eq(&self, other: &Self) -> bool {
        self.raw_id == other.raw_id
    }
}

impl<A: Asset> PartialOrd<Self> for Id<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: Asset> Ord for Id<A> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw_id.cmp(&other.raw_id)
    }
}

impl<A: Asset> Id<A> {
    #[must_use]
    pub const fn from_raw(raw_id: RawAssetIdWithTypeId) -> Self {
        Self {
            raw_id: raw_id.raw_id,
            _phantom_data: PhantomData,
        }
    }
}

impl<A: Asset> Display for Id<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_id)
    }
}

impl From<sparse_slot::Id> for RawAssetId {
    fn from(id: sparse_slot::Id) -> Self {
        Self {
            index: id.index as u16,
            generation: id.generation,
        }
    }
}

impl Into<sparse_slot::Id> for RawAssetId {
    fn into(self) -> sparse_slot::Id {
        sparse_slot::Id::new(self.index as usize, self.generation)
    }
}

impl<A: Asset> From<RawAssetIdWithTypeId> for Id<A> {
    fn from(id: RawAssetIdWithTypeId) -> Self {
        Self {
            raw_id: id.raw_id,
            _phantom_data: PhantomData,
        }
    }
}

#[must_use]
pub fn to_slot_map_id<A: Asset>(id: &Id<A>) -> sparse_slot::Id {
    sparse_slot::Id {
        index: id.raw_id.index as usize,
        generation: id.raw_id.generation,
    }
}

/// Validates an asset name according to strict (opinionated) naming conventions:
///
/// # Rules
/// - Must start with a lowercase letter (a-z)
/// - Can contain lowercase letters, numbers, underscores, hyphens and forward slashes
/// - Cannot end with special characters: slash (/), underscore (_), dot (.) or hyphen (-)
/// - Cannot contain consecutive special characters: slashes (//), underscores (__), dots (..) or hyphens (--)
/// - Forward slashes (/) can be used as path separators
///
/// # Examples
/// ```
/// use swamp_assets::is_valid_asset_name;
///
/// assert!(is_valid_asset_name("assets/textures/wood"));
/// assert!(is_valid_asset_name("player-model"));
/// assert!(is_valid_asset_name("player2-model"));
/// assert!(is_valid_asset_name("should.work.png"));
/// assert!(!is_valid_asset_name("_invalid"));
/// assert!(!is_valid_asset_name("also__invalid"));
/// assert!(!is_valid_asset_name("assets//textures"));
/// ```
#[must_use]
pub fn is_valid_asset_name(s: &str) -> bool {
    let mut chars = s.chars();

    matches!(chars.next(), Some(_c @ 'a'..='z'))
        && !s.ends_with(['/', '-', '_', '.'])
        && !s.contains("//")
        && !s.contains("__")
        && !s.contains("--")
        && !s.contains("..")
        && chars.all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '/' | '.')
        })
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AssetName {
    value: String,
}

impl AssetName {
    #[must_use]
    pub fn with_extension(&self, extension: &str) -> impl Into<AssetName> + Sized {
        Self {
            value: format!("{}.{}", self.value, extension),
        }
    }
}

// Example usage:
impl AssetName {
    pub fn new(value: &str) -> Self {
        assert!(is_valid_asset_name(value), "invalid asset name: {}", value);
        Self {
            value: value.into(),
        }
    }

    #[must_use]
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

impl Display for AssetName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssetName {{ '{}' }}", self.value)
    }
}

impl From<&str> for AssetName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl Into<PathBuf> for AssetName {
    fn into(self) -> PathBuf {
        self.value.into()
    }
}

impl<A: Asset> From<RawAssetId> for Id<A> {
    fn from(value: RawAssetId) -> Self {
        Self {
            raw_id: value,
            _phantom_data: PhantomData,
        }
    }
}

impl<A: Asset> From<&Id<A>> for RawAssetId {
    fn from(value: &Id<A>) -> Self {
        value.raw_id
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

    pub fn remove(&mut self, id: &Id<A>) {
        self.storage.remove(to_slot_map_id(id));
    }

    #[must_use]
    pub fn get(&self, id: &Id<A>) -> Option<&A> {
        self.storage.get(to_slot_map_id(id))
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

    pub fn iter(&self) -> impl Iterator<Item = (Id<A>, &A)> {
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

    #[must_use]
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
}
