use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig};
use oddio::SpatialScene;
use std::fmt::Debug;
use std::io;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use swamp_app::prelude::LocalResource;
use tracing::{debug, error, info, trace};

const MIN_SAMPLE_RATE: u32 = 44100;
const MAX_SAMPLE_RATE: u32 = 48000;

#[derive(LocalResource)]
pub struct Audio {
    #[allow(dead_code)]
    device: Device,
    #[allow(dead_code)]
    stream: Stream,
    //#[allow(dead_code)]
    //scene: Arc<oddio::SpatialScene>,
}

impl Debug for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Audio")
    }
}

#[allow(unused)]
fn debug_output(host: Host) {
    for device in host.devices().expect("should have a device") {
        info!(
            "Found device: {:?}",
            device.name().unwrap_or("unknown".to_string())
        );

        let configs = device.supported_output_configs();
        if configs.is_err() {
            continue;
        }

        for config in configs.unwrap() {
            info!(
                "  Channels: {}, Sample Rate: {} - {} Hz, Sample Format: {:?}",
                config.channels(),
                config.min_sample_rate().0,
                config.max_sample_rate().0,
                config.sample_format()
            );
        }
    }
}

impl Audio {
    pub fn new(
        spatial_scene: Arc<Mutex<SpatialScene>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();

        let default_device = host.default_output_device();
        if default_device.is_none() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no ",
            )));
        }
        let device = default_device.unwrap();
        let device_name = device.name().unwrap_or("unknown".parse().unwrap());
        debug!(device = device_name, "default output device");

        let all_supported_configs = device.supported_output_configs()?.collect::<Vec<_>>();

        for config in all_supported_configs {
            debug!("Supported config: {:?}", config);
        }

        let maybe_supported_config = device
            .supported_output_configs()?
            .filter(|config| {
                config.min_sample_rate().0 <= MAX_SAMPLE_RATE
                    && config.max_sample_rate().0 >= MIN_SAMPLE_RATE
            })
            .next();

        if maybe_supported_config.is_none() {
            error!("No supported output configurations with with an accepted output_config.");
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                "no supported output configurations found",
            )));
        }

        let supported_config = maybe_supported_config
            .unwrap()
            .with_sample_rate(cpal::SampleRate(MIN_SAMPLE_RATE));

        trace!(config=?supported_config, "Selected output config");

        let config: StreamConfig = supported_config.into();

        let sample_rate = config.sample_rate.0 as f32;
        info!(device=device_name, sample_rate, config=?&config, "selected device and configuration");

        //let scene = Arc::new(oddio::SpatialScene);

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let out_frames = oddio::frame_stereo(data);
                oddio::run(
                    spatial_scene.lock().unwrap().deref_mut(),
                    sample_rate as u32,
                    out_frames,
                );
            },
            move |err| {
                error!("Stream error: {}", err);
            },
            None,
        )?;

        stream.play().expect("Failed to start stream");

        Ok(Self { device, stream })
    }
}
