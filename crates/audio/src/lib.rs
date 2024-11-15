use crate::low_level::Audio;
use crate::mixer::AudioMixer;
use std::sync::Arc;
use swamp_app::prelude::{App, Plugin};
use tracing::error;

mod low_level;
pub mod mixer;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        let mixer = AudioMixer::new();
        let scene = Arc::clone(&mixer.mixer);
        app.insert_local_resource(mixer);

        let result = Audio::new(scene);
        if let Ok(audio) = result {
            app.insert_local_resource(audio);
        } else {
            error!(
                err = result.unwrap_err(),
                "could not initialize audio thread "
            );
        }
    }
}
