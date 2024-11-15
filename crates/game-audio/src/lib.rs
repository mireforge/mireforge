use swamp_assets::Assets;
use swamp_audio::mixer;
use swamp_audio_sample::{StereoSample, StereoSampleRef};
use tracing::debug;

pub type SoundHandle = u16;

pub trait Audio {
    fn play(&mut self, audio: &StereoSampleRef) -> SoundHandle;
}

// We will only borrow these resources for a single function call
pub struct GameAudio<'a> {
    pub mixer: &'a mut mixer::AudioMixer,
    pub stereo_samples: &'a Assets<StereoSample>,
}

impl<'a> GameAudio<'a> {
    pub fn new(mixer: &'a mut mixer::AudioMixer, stereo_samples: &'a Assets<StereoSample>) -> Self {
        Self {
            mixer,
            stereo_samples,
        }
    }
}

impl Audio for GameAudio<'_> {
    fn play(&mut self, sample_id: &StereoSampleRef) -> SoundHandle {
        debug!(sample_id=%sample_id, "playing sample");
        let stereo_sample = self.stereo_samples.fetch(sample_id);
        self.mixer
            .play(oddio::FramesSignal::from(stereo_sample.frames().clone()));

        // TODO: fix this

        53
    }
}
