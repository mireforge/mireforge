/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::owner::{AssetOwner, DropMessage};
use colored::Colorize;
use fixstr::FixStr;
use message_channel::Sender;
use std::any::{type_name, TypeId};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;
pub mod owner;

const FIXED_CAPACITY_SIZE: usize = 32;

pub trait Asset: 'static + Debug + Send + Sync {}
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct RawAssetId {
    pub generation: u16,
    pub index: u16,
}

impl RawAssetId {
    pub fn new(generation: u16, index: u16) -> Self {
        Self { generation, index }
    }
}

impl From<RawWeakId> for RawAssetId {
    fn from(value: RawWeakId) -> Self {
        value.raw_id
    }
}

impl Display for RawAssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}",
            self.index.to_string().bright_green(),
            self.generation.to_string().green()
        )
    }
}

fn short_type_name<T>() -> &'static str {
    type_name::<T>().split("::").last().unwrap()
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct RawWeakId {
    raw_id: RawAssetId,
    type_id: TypeId,
    debug_asset_name: AssetName,
    debug_type_id: FixStr<32>,
}

impl<A: Asset> From<&Id<A>> for RawWeakId {
    fn from(id: &Id<A>) -> Self {
        Self {
            raw_id: id.owner.raw_id().raw_id,
            type_id: TypeId::of::<A>(),
            debug_type_id: FixStr::new_unchecked(short_type_name::<A>()),
            debug_asset_name: id.owner.asset_name().unwrap(),
        }
    }
}

impl RawWeakId {
    #[must_use]
    pub fn with_asset_type<A: Asset>(id: RawAssetId, asset_name: AssetName) -> Self {
        Self {
            raw_id: id,
            type_id: TypeId::of::<A>(),
            debug_asset_name: asset_name,
            debug_type_id: FixStr::new_unchecked(short_type_name::<A>()),
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

impl Display for RawWeakId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.raw_id,
            self.debug_asset_name,
            self.debug_type_id.as_str().yellow()
        )
    }
}

/// You are free to copy and clone it, it has no ownership (no reference counting or similar)
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct WeakId<A: Asset> {
    raw_id: RawWeakId,
    phantom_data: PhantomData<A>,
}

// Manual Copy implementation
impl<A: Asset> Copy for WeakId<A> {} // No bounds needed since PhantomData<T> is always Copy

// Manual Clone implementation to satisfy Copy requirement
impl<A: Asset> Clone for WeakId<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Asset> WeakId<A> {
    #[must_use]
    pub fn new(raw_id: RawWeakId) -> Self {
        Self {
            raw_id,
            phantom_data: PhantomData,
        }
    }

    pub fn raw_id(&self) -> RawWeakId {
        self.raw_id
    }
}

impl<A: Asset> Display for WeakId<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "weak {}", self.raw_id)
    }
}

impl<A: Asset> From<WeakId<A>> for RawWeakId {
    fn from(id: WeakId<A>) -> Self {
        id.raw_id
    }
}

// Note: Do not implement a generic copy or clone for Id<A>
pub struct Id<A: Asset> {
    owner: Arc<AssetOwner>,
    _phantom_data: PhantomData<A>,
}

impl<A: Asset> Debug for Id<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.owner)
    }
}

impl<A: Asset> Display for Id<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.owner)
    }
}

impl<A: Asset> PartialEq<Self> for Id<A> {
    fn eq(&self, other: &Self) -> bool {
        self.owner.eq(&other.owner)
    }
}

impl<A: Asset> Eq for Id<A> {}

impl<A: Asset> PartialOrd<Self> for Id<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: Asset> Ord for Id<A> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.owner.cmp(&other.owner)
    }
}

impl<A: Asset> Clone for Id<A> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            _phantom_data: PhantomData,
        }
    }
}

impl<A: Asset> From<&Id<A>> for WeakId<A> {
    fn from(id: &Id<A>) -> WeakId<A> {
        WeakId::<A>::new(id.owner.raw_id())
    }
}

impl<A: Asset> Id<A> {
    pub fn new(raw_id: RawAssetId, sender: Sender<DropMessage>, asset_name: AssetName) -> Self {
        let raw_id_type = RawWeakId::with_asset_type::<A>(raw_id, asset_name);
        Self {
            owner: Arc::new(AssetOwner::new(raw_id_type, Some(asset_name), sender)),
            _phantom_data: PhantomData,
        }
    }

    pub fn asset_name(&self) -> Option<AssetName> {
        self.owner.asset_name()
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
/// use swamp_asset_id::is_valid_asset_name;
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
    if s.len() > FIXED_CAPACITY_SIZE {
        return false;
    }
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

#[derive(Debug, Copy, Clone, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct AssetName {
    value: FixStr<FIXED_CAPACITY_SIZE>,
}

impl AssetName {
    #[must_use]
    pub fn with_extension(&self, extension: &str) -> impl Into<AssetName> {
        let added = format!("{}.{}", self.value.as_str(), extension);
        Self {
            value: FixStr::new_unchecked(added.as_str()),
        }
    }
}

// Example usage:
impl AssetName {
    pub fn new(value: &str) -> Self {
        assert!(is_valid_asset_name(value), "invalid asset name: {}", value);
        Self {
            value: FixStr::new_unchecked(value),
        }
    }

    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl Display for AssetName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v = format!("{}", self.value).cyan();
        write!(f, "{}", v)
    }
}

impl From<&str> for AssetName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<AssetName> for PathBuf {
    fn from(value: AssetName) -> Self {
        value.value().into()
    }
}

impl<A: Asset> From<&Id<A>> for RawAssetId {
    fn from(value: &Id<A>) -> Self {
        value.owner.raw_id().raw_id
    }
}
