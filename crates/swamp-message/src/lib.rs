/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    slice::Iter,
};

use swamp_macros::Resource;
use swamp_resource::Resource;
use tracing::trace;

/// Trait representing a message in the system.
///
/// Messages must be `'static` to allow for type-erased storage and must implement `Copy`,
/// `Debug`, `Send`, and `Sync` to ensure they can be safely shared across threads and easily debugged.
pub trait Message: 'static + Debug + Send + Sync {}

/// Unique identifier for a specific message type.
///
/// This is primarily used for debugging purposes.
#[derive(Debug)]
pub struct MessageId<M: Message> {
    /// Internal value representing the message ID.
    ///
    value: u16,

    /// Phantom data to associate the ID with its message type.
    _phantom: PhantomData<M>,
}

impl<M: Message> Copy for MessageId<M> {}

impl<M: Message> Clone for MessageId<M> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M: Message> MessageId<M> {
    /// Creates a new `MessageId` with the given value.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    /// Retrieves the underlying value of the `MessageId`.
    #[must_use]
    pub const fn value(&self) -> u16 {
        self.value
    }
}

/// Stores information about a message, including its ID and the message itself.
///
/// This struct is used internally to manage messages across different frames.
#[derive(Debug)]
struct MessageInfo<M: Message> {
    /// Unique identifier for the message.
    #[allow(unused)]
    message_id: MessageId<M>,

    /// The actual message data.
    message: M,
}

/// Container for managing messages of a specific type.
///
/// This struct maintains separate lists for messages from the current and previous frames.
#[derive(Default, Resource, Debug)]
pub struct Messages<M: Message> {
    /// Messages sent in the previous frame.
    previous_frame_messages: Vec<MessageInfo<M>>,

    /// Messages sent in the current frame.
    current_messages: Vec<MessageInfo<M>>,
}

impl<M: Message> Messages<M> {
    /// Creates a new `Messages` container.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            previous_frame_messages: Vec::new(),
            current_messages: Vec::new(),
        }
    }

    /// Sends a new message, assigning it a unique `MessageId`.
    ///
    /// The message is added to the `current_messages` list.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be sent.
    ///
    /// # Returns
    ///
    /// A `MessageId` uniquely identifying the sent message.
    pub fn send(&mut self, message: M) -> MessageId<M> {
        let message_id = MessageId::new(self.current_messages.len() as u16);

        trace!("Sending message: {:?}", message);

        let message_info = MessageInfo {
            message_id,
            message,
        };

        self.current_messages.push(message_info);

        message_id
    }

    /// Swaps the current and previous frame message lists.
    ///
    /// This should be called at the start of each new frame (update) to transition messages appropriately.
    pub fn swap(&mut self) {
        self.previous_frame_messages.clear();

        std::mem::swap(
            &mut self.previous_frame_messages,
            &mut self.current_messages,
        );

        self.current_messages.clear();
    }

    /// Returns an iterator over the current frame's messages.
    #[must_use]
    pub fn iter_current(&self) -> MessagesIterator<M> {
        MessagesIterator {
            iter: self.current_messages.iter(),
        }
    }

    /// Returns an iterator over the previous frame's messages.
    #[must_use]
    pub fn iter_previous(&self) -> MessagesIterator<M> {
        MessagesIterator {
            iter: self.previous_frame_messages.iter(),
        }
    }

    /// Returns the number of messages in the current frame.
    #[must_use]
    pub fn len_current(&self) -> usize {
        self.current_messages.len()
    }

    /// Returns the number of messages in the previous frame.
    #[must_use]
    pub fn len_previous(&self) -> usize {
        self.previous_frame_messages.len()
    }

    /// Checks if there are no messages in the current frame.
    #[must_use]
    pub fn is_empty_current(&self) -> bool {
        self.current_messages.is_empty()
    }

    /// Checks if there are no messages in the previous frame.
    #[must_use]
    pub fn is_empty_previous(&self) -> bool {
        self.previous_frame_messages.is_empty()
    }
}

/// Iterator over messages of a specific type.
///
/// This iterator yields references to messages, allowing for non-consuming traversal.
pub struct MessagesIterator<'a, M: Message> {
    iter: Iter<'a, MessageInfo<M>>,
}

impl<'a, M: Message> Iterator for MessagesIterator<'a, M> {
    type Item = &'a M;

    #[must_use]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|message_info| &message_info.message)
    }
}

impl<'a, M: Message> DoubleEndedIterator for MessagesIterator<'a, M> {
    #[must_use]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|message_info| &message_info.message)
    }
}

/// Trait for type-erased message containers.
///
/// This allows for storing heterogeneous `Messages<M>` containers within a single collection.
pub trait MessageContainer: Any + Send + Sync {
    /// Swaps the current and previous frame message lists.
    fn swap(&mut self);

    /// Provides a reference to the container as `Any` for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Provides a mutable reference to the container as `Any` for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<M: Message> MessageContainer for Messages<M> {
    fn swap(&mut self) {
        self.swap();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Storage for all message types.
///
/// This struct maintains a registry mapping each message type to its corresponding `Messages<M>` container.
#[derive(Default)]
pub struct MessageStorage {
    registry: HashMap<TypeId, Box<dyn MessageContainer>>,
}

impl Debug for MessageStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MessageStorage")
    }
}

impl MessageStorage {
    /// Creates a new, empty `MessageStorage`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Registers a new message type within the storage.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The message type to register.
    ///
    /// # Example
    ///
    /// ```rust
    /// use swamp_message::prelude::*;
    ///
    /// #[derive(Message, Debug, Copy, Clone)]
    /// struct MyMessage;
    ///
    /// let mut storage = MessageStorage::new();
    ///
    /// storage.register_message_type::<MyMessage>();
    /// ```
    pub fn register_message_type<M: Message>(&mut self) {
        let type_id = TypeId::of::<M>();
        self.registry.insert(
            type_id,
            Box::new(Messages::<M>::new()) as Box<dyn MessageContainer>,
        );
    }

    /// Retrieves a mutable reference to the `Messages<M>` container for the specified message type.
    ///
    /// Returns `None` if the message type has not been registered.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The message type to retrieve.
    ///
    /// # Example
    ///
    /// ```rust
    /// use swamp_message::prelude::*;
    ///
    /// #[derive(Message, Debug, Copy, Clone)]
    /// struct MyMessage;
    ///
    /// let mut storage = MessageStorage::new();
    ///
    /// if let Some(messages) = storage.get_mut::<MyMessage>() {
    ///     // Use `messages` here
    /// }
    /// ```
    #[must_use]
    pub fn get_mut<M: Message>(&mut self) -> Option<&mut Messages<M>> {
        self.registry
            .get_mut(&TypeId::of::<M>())
            .and_then(|boxed| boxed.as_any_mut().downcast_mut::<Messages<M>>())
    }

    /// Retrieves an immutable reference to the `Messages<M>` container for the specified message type.
    ///
    /// Returns `None` if the message type has not been registered.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The message type to retrieve.
    ///
    /// # Example
    ///
    /// ```rust
    /// use swamp_message::prelude::*;
    ///
    /// #[derive(Message, Debug, Copy, Clone)]
    /// struct MyMessage;
    ///
    /// let mut storage = MessageStorage::new();
    ///
    /// if let Some(messages) = storage.get::<MyMessage>() {
    ///     // Use `messages` here
    /// }
    /// ```
    #[must_use]
    pub fn get<M: Message>(&self) -> Option<&Messages<M>> {
        self.registry
            .get(&TypeId::of::<M>())
            .and_then(|boxed| boxed.as_any().downcast_ref::<Messages<M>>())
    }

    /// Swaps the current and previous frame message lists for all registered message types.
    ///
    /// This should be called at the start of each new frame to transition messages appropriately.
    ///
    /// **Note:** The order in which message queues are swapped is not deterministic due to the nature of `HashMap`.
    /// This is generally acceptable but should be considered if order matters.
    pub fn swap_all(&mut self) {
        for messages_any in &mut self.registry.values_mut() {
            messages_any.swap();
        }
    }

    /// Sends a message of a specific type.
    ///
    /// This method abstracts over the message type, automatically handling registration if necessary.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The message type to send.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be sent.
    ///
    /// # Returns
    ///
    /// A `MessageId` uniquely identifying the sent message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use swamp_message::prelude::*;
    ///
    /// #[derive(Message, Debug, Copy, Clone)]
    /// struct MyMessage;
    ///
    /// let mut storage = MessageStorage::new();
    ///
    /// let msg_id = storage.send(MyMessage { /* fields */ });
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn send<M: Message>(&mut self, message: M) -> MessageId<M> {
        // Ensure the message type is registered.
        if !self.registry.contains_key(&TypeId::of::<M>()) {
            self.register_message_type::<M>();
        }

        // It's safe to unwrap here because we just registered the type if it wasn't present.
        self.get_mut::<M>()
            .expect("Message type should be registered")
            .send(message)
    }

    /// Iterates over all messages of a specific type in the current frame.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The message type to iterate over.
    ///
    /// # Returns
    ///
    /// An iterator over references to messages of type `M` in the current frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use swamp_message::prelude::*;
    ///
    /// #[derive(Message, Debug, Copy, Clone)]
    /// struct MyMessage;
    ///
    /// let mut storage = MessageStorage::new();
    ///
    /// for message in storage.iter_current::<MyMessage>() {
    ///     // Process `message`
    /// }
    /// ```
    #[must_use]
    pub fn iter_current<M: Message>(&self) -> Option<MessagesIterator<M>> {
        self.get::<M>().map(|messages| messages.iter_current())
    }

    /// Iterates over all messages of a specific type in the previous frame.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The message type to iterate over.
    ///
    /// # Returns
    ///
    /// An iterator over references to messages of type `M` in the previous frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use swamp_message::prelude::*;
    ///
    /// #[derive(Message, Debug, Copy, Clone)]
    /// struct MyMessage;
    ///
    /// let mut storage = MessageStorage::new();
    ///
    /// for message in storage.iter_previous::<MyMessage>() {
    ///     // Process `message`
    /// }
    /// ```
    #[must_use]
    pub fn iter_previous<M: Message>(&self) -> Option<MessagesIterator<M>> {
        self.get::<M>().map(|messages| messages.iter_previous())
    }
}
