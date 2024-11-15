/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;

/// A trait representing a local resource (single threaded). It extends `Any` to allow for downcasting.
pub trait LocalResource: Any + Debug + 'static {}

/// Storage for various resources identified by their `TypeId`.
#[derive(Debug)]
pub struct LocalResourceStorage {
    resources: HashMap<TypeId, Box<dyn Any + 'static>>,
}

impl Default for LocalResourceStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalResourceStorage {
    #[must_use]
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Inserts a new resource into the storage.
    ///
    /// If a resource of the same type already exists, it will be replaced.
    pub fn insert<R: LocalResource>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
    }

    /// Retrieves a reference to a resource of type `R`.
    ///
    /// # Panics
    ///
    /// Panics if the resource does not exist.
    #[must_use]
    pub fn fetch<R: LocalResource>(&self) -> &R {
        self.resources
            .get(&TypeId::of::<R>())
            .unwrap_or_else(|| panic!("LocalResource of type '{}' not found.", type_name::<R>()))
            .downcast_ref::<R>()
            .expect("Failed to downcast resource to the expected type.")
    }

    /// Retrieves a mutable reference to a resource of type `R`.
    ///
    /// # Panics
    ///
    /// Panics if the resource does not exist.
    #[must_use]
    pub fn fetch_mut<R: LocalResource>(&mut self) -> &mut R {
        self.resources
            .get_mut(&TypeId::of::<R>())
            .unwrap_or_else(|| panic!("LocalResource of type '{}' not found.", type_name::<R>()))
            .downcast_mut::<R>()
            .expect("Failed to downcast resource to the expected type.")
    }

    /// Retrieves an immutable reference to a resource of type `R`.
    ///
    /// Returns `Some(&R)` if the resource exists, otherwise returns `None`.
    #[must_use]
    pub fn get<R: LocalResource + 'static>(&self) -> Option<&R> {
        self.resources
            .get(&TypeId::of::<R>())
            .and_then(|boxed_any| boxed_any.downcast_ref::<R>())
    }

    /// Retrieves a mutable reference to a resource of type `R`.
    ///
    /// Returns `Some(&mut R)` if the resource exists, otherwise returns `None`.
    #[must_use]
    pub fn get_mut<R: LocalResource + 'static>(&mut self) -> Option<&mut R> {
        self.resources
            .get_mut(&TypeId::of::<R>())
            .and_then(|boxed_any| boxed_any.downcast_mut::<R>())
    }

    /// Removes a resource of type `R` from the storage.
    ///
    /// Returns `Some(R)` if the resource was present, otherwise `None`.
    ///
    /// # Panics
    ///
    /// Panics if the resource stored is not of the expected type `R`. Should be very unlikely.
    pub fn remove<R: LocalResource>(&mut self) -> Option<R> {
        self.resources.remove(&TypeId::of::<R>()).map(|boxed_any| {
            *boxed_any
                .downcast::<R>()
                .expect("Failed to downcast resource to the expected type.")
        })
    }

    /// Checks if a resource of type `R` exists in the storage.
    #[must_use]
    pub fn contains<R: LocalResource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
    }
}
