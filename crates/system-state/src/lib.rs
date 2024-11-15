/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use swamp_local_resource::{LocalResource, LocalResourceStorage};
use swamp_message::{Message, MessageStorage, Messages};
use swamp_resource::{Resource, ResourceStorage};

#[derive(Debug, Default)]
pub struct State {
    resources: ResourceStorage,
    local_resources: LocalResourceStorage,
    messages: MessageStorage,
}

impl State {
    #[must_use]
    pub fn new() -> Self {
        Self {
            resources: ResourceStorage::new(),
            messages: MessageStorage::new(),
            local_resources: LocalResourceStorage::new(),
        }
    }

    #[must_use]
    pub const fn messages(&self) -> &MessageStorage {
        &self.messages
    }

    pub fn messages_mut(&mut self) -> &mut MessageStorage {
        &mut self.messages
    }

    #[must_use]
    pub const fn resources(&self) -> &ResourceStorage {
        &self.resources
    }

    #[must_use]
    pub fn resources_mut(&mut self) -> &mut ResourceStorage {
        &mut self.resources
    }

    #[must_use]
    pub fn local_resources_mut(&mut self) -> &mut LocalResourceStorage {
        &mut self.local_resources
    }

    #[inline]
    #[must_use]
    pub fn resource<R: Resource>(&self) -> &R {
        self.resources.fetch::<R>()
    }

    #[inline]
    pub fn resource_mut<R: Resource>(&mut self) -> &mut R {
        self.resources.fetch_mut::<R>()
    }

    #[inline]
    #[must_use]
    pub fn local_resource<R: LocalResource>(&self) -> &R {
        self.local_resources.fetch::<R>()
    }

    #[inline]
    pub fn local_resource_mut<R: LocalResource>(&mut self) -> &mut R {
        self.local_resources.fetch_mut::<R>()
    }

    /// # Panics
    pub fn message_mut<M: Message>(&mut self) -> &mut Messages<M> {
        self.messages.get_mut::<M>().expect("Failed to get message")
    }

    /// # Panics
    pub fn message<M: Message>(&mut self) -> &Messages<M> {
        self.messages.get::<M>().expect("Failed to get message")
    }
}
