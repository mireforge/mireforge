/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

pub mod prelude;

use crate::prelude::trace;
use swamp_app::prelude::{App, Plugin};

//use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
//use tracing_subscriber::util::SubscriberInitExt;
pub struct LogPlugin;

impl Plugin for LogPlugin {
    fn build(&self, _app: &mut App) {
        init_logger();
        trace!("log plugin started")
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
}

#[cfg(target_arch = "wasm32")]
pub fn init_logger() {
    tracing_subscriber::registry()
        .with(tracing_wasm::WASMLayer::new(
            tracing_wasm::WASMLayerConfig::default(),
        ))
        .init();
}
