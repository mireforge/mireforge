/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
mod id_gen;
mod idx_gen;

use crate::id_gen::IdAssigner;
use chunk_reader::get_platform_reader;
use message_channel::{Channel, Receiver, Sender};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use swamp_app::prelude::*;
use swamp_app::system_types::ReAll;
use swamp_assets::prelude::*;
use swamp_assets_loader::{AssetLoaderRegistry, LoadError, WrappedAssetLoaderRegistry};
use swamp_loader::{load, Blob};
use swamp_loader_plugin::{LoaderReceiver, LoaderSender};
use tracing::debug;

#[derive(Debug)]
pub enum Phase {
    Loading,
    Error,
    Defined,
}

#[derive(Debug)]
pub struct AssetInfo {
    pub name: AssetName,
    pub phase: Phase,
}

type TypeIdMap<T> = HashMap<TypeId, T>;

#[derive(Debug, Resource)]
pub struct AssetRegistry {
    infos: HashMap<RawWeakId, AssetInfo>,
    sender: Sender<Blob>,
    id_assigner: IdAssigner,
    converters: Arc<Mutex<AssetLoaderRegistry>>,
    #[allow(unused)]
    drop_channel_receiver: Receiver<DropMessage>,
}

impl AssetRegistry {
    #[must_use]
    pub fn new(
        sender: Sender<Blob>,
        asset_loader_registry: Arc<Mutex<AssetLoaderRegistry>>,
    ) -> Self {
        let (drop_channel_sender, drop_channel_receiver) = Channel::create();
        Self {
            infos: HashMap::new(),
            sender,
            id_assigner: IdAssigner::new(drop_channel_sender),
            converters: asset_loader_registry,
            drop_channel_receiver,
        }
    }

    pub fn load<T: Asset>(&mut self, name: impl Into<AssetName>) -> Id<T> {
        let asset_name = name.into();
        debug!(asset_name=%asset_name, "Loading");
        let reader = get_platform_reader("assets/");
        let typed_id = self.id_assigner.allocate::<T>(asset_name);
        let raw_type_id: RawWeakId = (&typed_id).into();
        self.infos.insert(
            raw_type_id,
            AssetInfo {
                name: asset_name,
                phase: Phase::Loading,
            },
        );
        let sender = self.sender.clone();
        {
            future_runner::run_future(async move {
                load(reader, &sender, asset_name, raw_type_id).await;
            });
        }
        typed_id
    }

    pub fn name<A: Asset>(&self, id: Id<A>) -> Option<AssetName> {
        let raw_id = (&id).into();
        self.name_raw(raw_id)
    }

    pub fn name_raw(&self, raw_id: RawWeakId) -> Option<AssetName> {
        self.infos.get(&raw_id).map(|info| info.name)
    }

    pub fn blob_loaded(
        &mut self,
        id: RawWeakId,
        octets: &[u8],
        resources: &mut ResourceStorage,
    ) -> Result<(), LoadError> {
        self.infos.get_mut(&id).unwrap().phase = Phase::Defined;
        self.converters
            .lock()
            .unwrap()
            .convert_and_insert(id, octets, resources)
    }

    pub fn asset_id_dropped<A: Asset>(&mut self, id: Id<A>) {
        self.infos.remove(&(&id).into());
        self.id_assigner.remove(id);
    }
}

pub struct AssetRegistryPlugin;

impl Plugin for AssetRegistryPlugin {
    fn build(&self, app: &mut App) {
        let sender = app.resource_take::<LoaderSender>();
        {
            let asset_loader_registry = app.resource::<WrappedAssetLoaderRegistry>();
            app.insert_resource(AssetRegistry::new(
                sender.sender,
                Arc::clone(&asset_loader_registry.value),
            ));
        }
        app.add_system(UpdatePhase::First, tick);
    }
}

fn tick(
    loader_receiver: Re<LoaderReceiver>,
    mut asset_container: ReM<AssetRegistry>,
    mut mut_access_to_resources: ReAll,
) {
    if let Some(blob) = loader_receiver.receiver.try_recv() {
        debug!("loaded {:?}, starting conversion", blob);
        asset_container
            .blob_loaded(blob.id, &blob.content, &mut mut_access_to_resources)
            .expect("couldn't convert")
    }
}
