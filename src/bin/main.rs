#![allow(dead_code)]
use std::sync::{Arc, RwLock};
use std::thread;

use cpal::{Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use yat::types::ModuleResult;
use yat::rack::Rack;
use yat::oscillator::Oscillator;
use yat::audo_out::AudioOut;

use yat::types::IoPort;


fn main() -> ModuleResult<()> {
    //let audio_out_port = setup_audio_thread();
    let audio_out_port = Arc::new(RwLock::new(None));
    setup_audio_thread(audio_out_port.clone());

    let audio_out = AudioOut::new(String::from("audio_out"), audio_out_port);

    let mut rack = Rack::new();

    let osc = Oscillator::new("osc".to_string());

    rack.add_module(Box::new(osc));
    rack.add_module(Box::new(audio_out));

    rack.connect_modules("osc", "audio_out", "audio_out", "audio_in")?;

    rack.print_connection("osc", "audio_out");

    rack.print_module_order();

    //while true {
        rack.process_module_chain();
    //}

    Ok(())
}


fn setup_audio_thread(io_port: IoPort) { //-> IoPort {

   //let audio_thread = thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        let config = device.default_output_config().unwrap();

      let ret_audio_out = match config.sample_format() {
            SampleFormat::F32 => run::<f32>(&device, &config.into(), io_port.clone()).unwrap(),
            SampleFormat::I16 => run::<i16>(&device, &config.into(), io_port.clone()).unwrap(),
            SampleFormat::U16 => run::<u16>(&device, &config.into(), io_port.clone()).unwrap(),
        };
    //});

   //audio_thread.join();

    //ret_audio_out
}

// Build output stream and play audio
fn run<T: Sample>(device: &cpal::Device, config: &cpal::StreamConfig, io_port: IoPort)
-> Result<IoPort, anyhow::Error> {


    let ret_audio_out = Arc::new(RwLock::new(None));
    let thread_audio_out = io_port.clone();


    // Get sample rate and channel number from the config
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on the stream: {}", err);

    // Define some variables we need for a simple oscillator
	// Build an output stream
	let stream = device.build_output_stream(
		config,
		move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
			for frame in data.chunks_mut(channels) {
				// Convert the make_noise output into a sample
                let value: T = match *thread_audio_out.to_owned().read().unwrap() {
                    Some(val) => {
                        cpal::Sample::from::<f32>(&val)
                    },
                    None => {
                        cpal::Sample::from::<f32>(&0.0)
                    },
                };

				for sample in frame.iter_mut() {
					*sample = value;
				}
			}
		},
		err_fn,
    )?;

    // Play the stream
    stream.play()?;

    // Park the thread so our noise plays continuously until the app is closed
    //std::thread::park();


    Ok(ret_audio_out)
}
