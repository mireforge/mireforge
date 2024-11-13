/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use std::fmt::Debug;
use swamp_macros::Message;
use swamp_message::{Message, MessageStorage};

/// Example message type for testing.
#[derive(Copy, Clone, Message, Debug)]
pub struct MyMessage {
    pub secret: u8,
}

/// Another message type for testing multiple message types.
#[derive(Copy, Clone, Message, Debug, PartialEq, Eq)]
pub struct AnotherMessage {
    pub data: u32,
}

/// Generic message type for testing generics support.
#[derive(Copy, Clone, Message, Debug, Eq, PartialEq)]
pub struct GenericMessage<T: 'static + Debug + Send + Sync + Copy + Clone> {
    pub value: T,
}

#[test_log::test]
fn swapping() {
    let mut storage = MessageStorage::new();

    storage.register_message_type::<MyMessage>();

    {
        let msg_queue = storage
            .get_mut::<MyMessage>()
            .expect("we just registered MyMessage");

        msg_queue.send(MyMessage { secret: 42 });
    }

    {
        let msg_queue_to_read = storage
            .get::<MyMessage>()
            .expect("we just registered MyMessage");
        assert_eq!(msg_queue_to_read.len_current(), 1);

        let mut count = 0;
        for my_message in msg_queue_to_read.iter_current() {
            assert_eq!(my_message.secret, 42);
            count += 1;
        }

        assert_eq!(count, 1);
    }

    storage.swap_all();

    {
        let queue_left = storage
            .get::<MyMessage>()
            .expect("we just registered MyMessage");

        assert_eq!(queue_left.len_previous(), 1);
        assert_eq!(queue_left.len_current(), 0);
    }

    storage.swap_all();

    {
        let empty_queue = storage
            .get::<MyMessage>()
            .expect("we just registered MyMessage");

        assert_eq!(empty_queue.len_previous(), 0);
        assert_eq!(empty_queue.len_current(), 0);
    }
}

#[test_log::test]
fn send_message_with_generic_type() {
    let mut storage = MessageStorage::new();

    // Use send_message to send a generic message without prior registration
    let _ = storage.send(GenericMessage { value: 42.42 });

    // Verify that GenericMessage<f64> was automatically registered and message was sent
    {
        let msg_queue = storage
            .get::<GenericMessage<f64>>()
            .expect("GenericMessage<f64> should be automatically registered");

        assert_eq!(msg_queue.len_current(), 1);

        let msg = msg_queue.iter_current().next().unwrap();
        assert_eq!(msg.value, 42.42);
    }

    // Swap messages
    storage.swap_all();

    // Verify previous messages
    {
        let msg_queue = storage
            .get::<GenericMessage<f64>>()
            .expect("GenericMessage<f64> should be automatically registered");

        assert_eq!(msg_queue.len_previous(), 1);
        assert_eq!(msg_queue.len_current(), 0);

        let msg = msg_queue.iter_previous().next().unwrap();
        assert_eq!(msg.value, 42.42);
    }
}

#[test_log::test]
fn multiple_message_types() {
    let mut storage = MessageStorage::new();

    storage.register_message_type::<MyMessage>();
    storage.register_message_type::<AnotherMessage>();

    // Send messages of different types
    {
        let my_msg_queue = storage
            .get_mut::<MyMessage>()
            .expect("MyMessage should be registered");
        my_msg_queue.send(MyMessage { secret: 100 });

        let another_msg_queue = storage
            .get_mut::<AnotherMessage>()
            .expect("AnotherMessage should be registered");
        another_msg_queue.send(AnotherMessage { data: 999 });
    }

    // Verify current messages for both types
    {
        let my_msg_queue = storage
            .get::<MyMessage>()
            .expect("MyMessage should be registered");
        assert_eq!(my_msg_queue.len_current(), 1);
        let my_msg = my_msg_queue.iter_current().next().unwrap();
        assert_eq!(my_msg.secret, 100);

        let another_msg_queue = storage
            .get::<AnotherMessage>()
            .expect("AnotherMessage should be registered");
        assert_eq!(another_msg_queue.len_current(), 1);
        let another_msg = another_msg_queue.iter_current().next().unwrap();
        assert_eq!(another_msg.data, 999);
    }

    // Swap messages
    storage.swap_all();

    // Verify previous messages for both types
    {
        let my_msg_queue = storage
            .get::<MyMessage>()
            .expect("MyMessage should be registered");
        assert_eq!(my_msg_queue.len_previous(), 1);
        let my_msg = my_msg_queue.iter_previous().next().unwrap();
        assert_eq!(my_msg.secret, 100);

        let another_msg_queue = storage
            .get::<AnotherMessage>()
            .expect("AnotherMessage should be registered");
        assert_eq!(another_msg_queue.len_previous(), 1);
        let another_msg = another_msg_queue.iter_previous().next().unwrap();
        assert_eq!(another_msg.data, 999);
    }

    // Swap again to clear messages
    storage.swap_all();

    // Verify all messages are cleared
    {
        let my_msg_queue = storage
            .get::<MyMessage>()
            .expect("MyMessage should be registered");
        assert_eq!(my_msg_queue.len_previous(), 0);
        assert_eq!(my_msg_queue.len_current(), 0);

        let another_msg_queue = storage
            .get::<AnotherMessage>()
            .expect("AnotherMessage should be registered");
        assert_eq!(another_msg_queue.len_previous(), 0);
        assert_eq!(another_msg_queue.len_current(), 0);
    }
}
