use std::error::Error;
use std::sync::mpsc::Receiver;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};

use yat_rack::types::{SampleType, SAMPLE_RATE};

use std::thread;

pub fn setup_audio_thread(audio_rx: Receiver<SampleType>) {
    let _ = thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no device available");

        let channels = 2u16;
        let sample_rate = cpal::SampleRate(SAMPLE_RATE as u32);
        let suported_buffer_size = cpal::SupportedBufferSize::Unknown;
        let sample_format = cpal::SampleFormat::F32;

        let config = cpal::SupportedStreamConfig::new(
            channels,
            sample_rate,
            suported_buffer_size,
            sample_format
        );

        match config.sample_format() {
            SampleFormat::F32 => run::<f32>(&device, &config.into(), audio_rx).unwrap(),
            SampleFormat::I16 => run::<i16>(&device, &config.into(), audio_rx).unwrap(),
            SampleFormat::U16 => run::<u16>(&device, &config.into(), audio_rx).unwrap(),
        }
    });
}

// Build output stream and play audio
fn run<T: Sample>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    audio_rx: Receiver<SampleType>,
) -> Result<(), Box<dyn Error>> {
    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on the stream: {}", err);

    // Build an output stream
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                // NOTE: Converting from the rack's 64-bit floats to 32-bit for samples
                let next_sample: f32 = match audio_rx.recv() {
                    Ok(sample) => sample as f32,
                    Err(_) => break,
                };
                let value: T = cpal::Sample::from::<f32>(&next_sample);

                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        },
        err_fn,
    )?;

    stream.play()?;

    std::thread::park();

    Ok(())
}

