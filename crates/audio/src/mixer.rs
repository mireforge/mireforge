use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use swamp_app::prelude::LocalResource;
#[derive(LocalResource)]
pub struct Mixer {
    pub scene: Arc<Mutex<oddio::SpatialScene>>,
    #[allow(dead_code)]
    scene_control: oddio::SpatialSceneControl,
    #[allow(dead_code)]
    mixer: oddio::Mixer<f32>,
    #[allow(dead_code)]
    mixer_control: oddio::MixerControl<f32>,
}

impl Debug for Mixer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mixer")
    }
}

impl Mixer {
    pub fn new() -> Self {
        let (scene_control, scene) = oddio::SpatialScene::new();
        let (mixer_control, mixer) = oddio::Mixer::<f32>::new();

        Self {
            scene: Arc::new(Mutex::new(scene)),
            scene_control,
            mixer_control,
            mixer,
        }
    }

    #[allow(unused)]
    pub fn play(&mut self, signal: oddio::FramesSignal<f32>) {
        self.mixer_control.play(signal);
    }
}
