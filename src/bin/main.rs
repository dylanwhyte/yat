#![allow(dead_code)]
use std::sync::{Arc, RwLock, Mutex};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::io::{self, BufRead};

use cpal::{Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use yat::io_module_trait::IoModuleTrait;
use yat::types::{ModuleResult, SampleType};
use yat::rack::Rack;
use yat::oscillator::Oscillator;
use yat::audo_out::AudioOut;


fn main() -> ModuleResult<()> {
    //let audio_out_port = setup_audio_thread();
    //let audio_out_port = Arc::new(RwLock::new(None));

    let (audio_out, audio_rx) = AudioOut::new(String::from("audio_out"));
    let rack = Arc::new(Mutex::new(Rack::new()));

    setup_audio_thread(audio_rx);

    let c_rack_ref = Arc::clone(&rack);
    let s_rack_ref = Arc::clone(&rack);


    let osc = Oscillator::new("osc".to_string(),
    rack.lock().unwrap().clock.time.clone());

    let mut lfo1 = Oscillator::new("lfo1".to_string(),
    rack.lock().unwrap().clock.time.clone());

    let mut lfo2 = Oscillator::new("lfo2".to_string(),
    rack.lock().unwrap().clock.time.clone());

    let ctrl_a = Arc::new(RwLock::new(Some(5.0)));
    lfo1.set_in_port("freq", ctrl_a);

    let ctrl_b = Arc::new(RwLock::new(Some(500.0)));
    lfo2.set_in_port("freq", ctrl_b);

    { rack.lock().unwrap().add_module(Arc::new(Mutex::new(osc))); }
    { rack.lock().unwrap().add_module(Arc::new(Mutex::new(lfo1))); }
    { rack.lock().unwrap().add_module(Arc::new(Mutex::new(lfo2))); }
    { rack.lock().unwrap().add_module(Arc::new(Mutex::new(audio_out))); }

    { rack.lock().unwrap().connect_modules("lfo1", "audio_out", "osc", "amp")?; }

    { rack.lock().unwrap().connect_modules("lfo2", "audio_out", "osc", "freq")?; }

    { rack.lock().unwrap().connect_modules("osc", "audio_out", "audio_out", "audio_in")?; }


    { rack.lock().unwrap().print_connection("lfo1", "osc"); }

    { rack.lock().unwrap().print_connection("osc", "audio_out"); }

    { rack.lock().unwrap().print_module_order(); }


    let stdin = io::stdin();
        //let controller_thread_scope =
    thread::scope(|c_scope| {
            let (quit_tx, quit_rx) = mpsc::sync_channel(1);
            c_scope.spawn(move || {
                loop {
                    while *s_rack_ref.lock().unwrap().running.get_mut() {
                        { s_rack_ref.lock().unwrap().process_module_chain(); }
                    }
                    match quit_rx.try_recv() {
                        Ok(_) => break,
                        Err(_) => continue,
                    }
                }
            });

            for line in stdin.lock().lines() {
                let command = line.unwrap();
                if command == "quit" {
                    println!("Quiting...");
                    c_scope.spawn(|| { c_rack_ref.lock().unwrap().stop() });
                    quit_tx.send(true).unwrap();
                    break;
                } else if command == "stop" {
                    println!("stopping...");
                    c_scope.spawn(|| { c_rack_ref.lock().unwrap().stop() });
                } else if command == "run" {
                    println!("running...");
                    c_scope.spawn(|| { c_rack_ref.lock().unwrap().run() });
                } else {
                    println!("Unknown command: {}", command);
                }
            }

        });


    Ok(())
}

fn start_controller() {
    let stdin = io::stdin();
    thread::spawn(move || {
        for line in stdin.lock().lines() {
            let command = line.unwrap();
            println!("Command: {}", command);
            if command == "quit" {
                println!("Quiting...");
                break;
            } else {
                println!("Unknown command: {}", command);
            }
        }
    });

}


fn setup_audio_thread(
    audio_rx: Receiver<SampleType>)
{ //-> IoPort {

    let _ = thread::spawn(move || {

        let host = cpal::default_host();
        let device = host.default_output_device().expect("no device available");
        let config = device.default_output_config().unwrap();

        let _ = match config.sample_format() {
            SampleFormat::F32 => run::<f32>(
                &device,
                &config.into(),
                audio_rx).unwrap(),
            SampleFormat::I16 => run::<i16>(
                &device,
                &config.into(),
                audio_rx).unwrap(),
            SampleFormat::U16 => run::<u16>(
                &device,
                &config.into(),
                audio_rx).unwrap(),
        };
    });

   //audio_thread.join();

    //ret_audio_out
}

// Build output stream and play audio
fn run<T: Sample>(device: &cpal::Device, config: &cpal::StreamConfig, audio_rx: Receiver<SampleType>)
-> Result<(), anyhow::Error> {


    // Get sample rate and channel number from the config
    let sample_rate = config.sample_rate.0 as f32;
    println!("sample rate: {}", sample_rate);
    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on the stream: {}", err);

    // Define some variables we need for a simple oscillator
	// Build an output stream
	let stream = device.build_output_stream(
		config,
		move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
			for frame in data.chunks_mut(channels) {
				// Convert the make_noise output into a sample
                let next_sample = audio_rx.recv().unwrap();
                let value: T = cpal::Sample::from::<f32>(&next_sample);

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
    std::thread::park();


    Ok(())
}
