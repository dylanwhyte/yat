#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use yat::types::Result;
use yat::rack::Rack;
use yat::oscillator::Oscillator;
use yat::io_module::IoModule;

use yat::types::{IoPort,SampleType};
use yat::io_module_trait::IoModuleTrait;

use yat::gen_io_module_type;

fn main() -> Result<()> {
    let mut rack = Rack::new();

    gen_io_module_type!(Filter);

    let flt_lp: Filter = Filter::new(
        "flt_lp".to_string(),
        HashMap::from([
            ("audio_in".to_string(), Arc::new(RwLock::new(None))),
            ("cut_off".to_string(), Arc::new(RwLock::new(None))),
        ]),
        HashMap::from([
            ("audio_out".to_string(), Arc::new(RwLock::new(None))),
        ]),
        |flt: &Filter, _timee: f64| {
            let _cut_off = flt.read_in_port_value("cut_off").unwrap();
            let audio_in = flt.read_in_port_value("audio_in").unwrap();

            flt.write_out_port_value("audio_out", Some(audio_in));
        },
    );

    fn process_inputs(osc: &Oscillator, time: f64) {
        let pi: SampleType = 3.14159265359;

        let amp = osc.read_in_port_value("amp").unwrap();
        let freq = osc.read_in_port_value("freq").unwrap();
        let audio_out = amp * (2.0 * pi * freq * time).sin();

        osc.write_out_port_value("audio_out", Some(audio_out));
    }


    let osc_process_fn: fn(&Oscillator, f64) = process_inputs;
    let osc = Oscillator::new("osc".to_string(), osc_process_fn);
    let filter = IoModule::new_blank("flt".to_string());
    let adsr = IoModule::new_blank("adsr".to_string());
    let output = IoModule::new_blank("output".to_string());

    let ctrl_a = IoModule::new_blank("ctrl_a".to_string());
    let ctrl_b = IoModule::new_blank("ctrl_b".to_string());
    let ctrl_c = IoModule::new_blank("ctrl_c".to_string());
    let ctrl_d = IoModule::new_blank("ctrl_d".to_string());
    let ctrl_e = IoModule::new_blank("ctrl_e".to_string());
    let ctrl_f = IoModule::new_blank("ctrl_f".to_string());

    rack.add_module(Box::new(osc));
    rack.add_module(Box::new(filter));
    rack.add_module(Box::new(adsr));
    rack.add_module(Box::new(output));
    rack.add_module(Box::new(ctrl_a));
    rack.add_module(Box::new(ctrl_b));
    rack.add_module(Box::new(ctrl_c));
    rack.add_module(Box::new(ctrl_d));
    rack.add_module(Box::new(ctrl_e));
    rack.add_module(Box::new(ctrl_f));

    // Add module created via macro
    rack.add_module(Box::new(flt_lp));

    rack.modules.get_mut("flt").unwrap().create_port("in", "audio_in");
    rack.modules.get_mut("flt").unwrap().create_port("in", "freq");
    rack.modules.get_mut("flt").unwrap().create_port("out", "audio_out");

    rack.modules.get_mut("adsr").unwrap().create_port("in", "attack");
    rack.modules.get_mut("adsr").unwrap().create_port("in", "decay");
    rack.modules.get_mut("adsr").unwrap().create_port("in", "sustain");
    rack.modules.get_mut("adsr").unwrap().create_port("in", "release");
    rack.modules.get_mut("adsr").unwrap().create_port("in", "audio_in");
    rack.modules.get_mut("adsr").unwrap().create_port("out", "audio_out");

    rack.modules.get_mut("ctrl_a").unwrap().create_port("out", "ctrl_out");
    rack.modules.get_mut("ctrl_b").unwrap().create_port("out", "ctrl_out");
    rack.modules.get_mut("ctrl_c").unwrap().create_port("out", "ctrl_out");
    rack.modules.get_mut("ctrl_d").unwrap().create_port("out", "ctrl_out");
    rack.modules.get_mut("ctrl_e").unwrap().create_port("out", "ctrl_out");
    rack.modules.get_mut("ctrl_f").unwrap().create_port("out", "ctrl_out");


    rack.modules.get_mut("output").unwrap().create_port("in", "audio_in");

    println!("Connecting ctrl knobs");
    rack.connect_modules("ctrl_e", "ctrl_out", "osc", "amp")?;
    rack.connect_modules("ctrl_a", "ctrl_out", "osc", "freq")?;

    println!("Connecting flt");
    rack.connect_modules("flt", "audio_out", "adsr", "audio_in")?;
    rack.connect_modules("ctrl_a", "ctrl_out", "adsr", "attack")?;
    rack.connect_modules("ctrl_b", "ctrl_out", "adsr", "decay")?;
    rack.connect_modules("ctrl_c", "ctrl_out", "adsr", "sustain")?;
    rack.connect_modules("ctrl_d", "ctrl_out", "adsr", "release")?;

    println!("Connecting osc");
    rack.connect_modules("osc", "audio_out", "flt", "audio_in")?;

    rack.connect_modules("adsr", "audio_out", "output", "audio_in")?;

    println!("Connecting flt_lp");
    rack.connect_modules("ctrl_f", "ctrl_out", "flt_lp", "cut_off")?;
    rack.connect_modules("osc", "audio_out", "flt_lp", "audio_in")?;
    rack.connect_modules("flt_lp", "audio_out", "output", "audio_in")?;

    //rack.list_ports();

    rack.print_connection("osc", "flt");
    rack.print_connection("ctrl_a", "osc");
    rack.print_connection("ctrl_a", "adsr");
    rack.print_connection("ctrl_b", "adsr");
    rack.print_connection("flt", "adsr");

    rack.print_connection("flt_lp", "osc");
    rack.print_connection("osc", "flt_lp");
    rack.print_connection("ctrl_f", "flt_lp");
    rack.print_connection("flt_lp", "output");

    rack.print_connection("adsr", "output");


    rack.print_module_order();

    Ok(())
}


