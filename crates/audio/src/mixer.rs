use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use swamp_app::prelude::LocalResource;
#[derive(LocalResource)]
pub struct AudioMixer {
    //pub scene: Arc<Mutex<oddio::SpatialScene>>,
    //#[allow(dead_code)]
    //scene_control: oddio::SpatialSceneControl,
    #[allow(dead_code)]
    pub mixer: Arc<Mutex<oddio::Mixer<[f32; 2]>>>,
    #[allow(dead_code)]
    mixer_control: oddio::MixerControl<[f32; 2]>,
}

impl Debug for AudioMixer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mixer")
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioMixer {
    pub fn new() -> Self {
        //let (scene_control, scene) = oddio::SpatialScene::new();
        let (mixer_control, mixer) = oddio::Mixer::<[f32; 2]>::new();

        Self {
            // scene: Arc::new(Mutex::new(scene)),
            // scene_control,
            mixer_control,
            mixer: Arc::new(Mutex::new(mixer)),
        }
    }

    #[allow(unused)]
    pub fn play(&mut self, signal: oddio::FramesSignal<[f32; 2]>) {
        self.mixer_control.play(signal);
    }
}
