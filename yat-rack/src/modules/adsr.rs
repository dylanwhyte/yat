use std::sync::{Arc, RwLock};

use crate::clock::Clock;
use crate::modules::io_module::IoModule;
use crate::types::{IoPort, PortNotFoundError, PortResult, SampleType};

#[derive(PartialEq, Eq)]
enum AdsrState {
    Inactive,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// An ADSR (Attack Decay Sustain Release) envelope generator
pub struct Adsr {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    input_ports: Vec<String>,

    output_ports: Vec<String>,

    /// The input audio signal
    in_audio: IoPort,

    /// The signal which informs the envelope generator that a note is
    /// active
    in_trigger: IoPort,

    /// The time (seconds) until the signal's full amplitude is reached
    in_attack: IoPort,

    /// The time (seconds) until the signal decays to the sustain amplitude
    in_decay: IoPort,

    /// The percentage amplitude to remain at, while a not is held
    in_sustain: IoPort,

    /// The time (seconds) for the amplitude to fully decay after a
    /// note is released
    in_release: IoPort,

    /// The modulated output signal of the envelope generator
    out_audio_out: IoPort,

    /// The current time for which a note has been active
    active_time: SampleType,

    /// The time at which a note was triggered
    trigger_time: SampleType,

    /// Which phase of processing the ADSR is in
    adsr_state: AdsrState,

    /// Time of the rack's clock
    clock: Arc<RwLock<Clock>>,
}

impl Adsr {
    /// Create a new, unordered IoModule
    pub fn new(id: String, time: Arc<RwLock<Clock>>) -> Self {
        let order = None;
        let input_ports = vec!["amp".to_string(), "freq".to_string()];
        let output_ports = vec!["audio_out".to_string()];

        let in_audio = Arc::new(RwLock::new(None));
        let in_trigger = Arc::new(RwLock::new(None));
        let in_attack = Arc::new(RwLock::new(None));
        let in_decay = Arc::new(RwLock::new(None));
        let in_sustain = Arc::new(RwLock::new(None));
        let in_release = Arc::new(RwLock::new(None));
        let out_audio_out = Arc::new(RwLock::new(None));

        let active_time = 0f64;
        let trigger_time = 0f64;
        let adsr_state = AdsrState::Inactive;

        Self {
            id,
            order,
            input_ports,
            output_ports,
            in_audio,
            in_trigger,
            in_attack,
            in_decay,
            in_sustain,
            in_release,
            out_audio_out,
            active_time,
            trigger_time,
            adsr_state,
            clock: time,
        }
    }
}

impl PartialEq for Adsr {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl IoModule for Adsr {
    /// Read inputs and populate outputs
    fn process_inputs(&mut self) {
        let trigger_active = self
            .in_trigger
            .read()
            .expect("RwLock is poisoned")
            .unwrap_or(0f64)
            != 0f64;

        // no key is active
        if (self.adsr_state == AdsrState::Inactive) && (!trigger_active) {
            let mut value = self.out_audio_out.write().expect("RwLock is poisoned");
            *value = Some(0f64);
        } else {
            // FIXME: Add time to module
            let clock = self.clock.read().expect("RwLock is poisoned");

            let audio_in = self
                .in_audio
                .read()
                .expect("RwLock is poisoned")
                .unwrap_or(0f64);
            let sustain_amp = self
                .in_sustain
                .read()
                .expect("RwLock is poisoned")
                .unwrap_or(0.5f64);

            // This makes sense as a default value, in case attack and decay are zero
            let mut audio_out = audio_in * sustain_amp;

            // TODO: this may be entirely wrong
            match self.adsr_state {
                AdsrState::Inactive => {
                    if trigger_active {
                        self.trigger_time = clock.get_current_time().unwrap();
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Attack;
                    }
                }
                AdsrState::Attack => {
                    // Transition to max amplitude, and change state to decay after time
                    // If released, go straight to that
                    // Effectively set to zero, but avoiding potential zero division
                    let attack = self
                        .in_attack
                        .read()
                        .expect("RwLock is poisoned")
                        .unwrap_or(clock.time_delta);
                    if !trigger_active {
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Release;
                    } else if self.active_time >= attack {
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Decay;
                    } else {
                        // Gradually increase amplitude to max
                        audio_out = audio_in * (self.active_time / attack);
                    }
                }
                AdsrState::Decay => {
                    // Transition to sustain amplitude
                    // Effectively set to zero, but avoiding potential zero division
                    let decay = self
                        .in_decay
                        .read()
                        .expect("RwLock is poisoned")
                        .unwrap_or(clock.time_delta);
                    if !trigger_active {
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Release;
                    } else if self.active_time >= decay {
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Sustain;
                    } else {
                        let sustain_amp = self
                            .in_sustain
                            .read()
                            .expect("RwLock is poisoned")
                            .unwrap_or(0.5f64);

                        // Decay to sustain amplitude
                        audio_out =
                            audio_in * (1f64 - ((self.active_time * (1f64 - sustain_amp)) / decay));
                    }
                }
                AdsrState::Sustain => {
                    // Output at sustain level while trigger is active
                    if !trigger_active {
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Release;
                    } else {
                        audio_out = audio_in * sustain_amp;
                    }
                }
                AdsrState::Release => {
                    // Effectively set to zero, but avoiding potential zero division
                    let release = self
                        .in_release
                        .read()
                        .expect("RwLock is poisoned")
                        .unwrap_or(clock.time_delta);
                    // Decay to zero
                    if self.active_time >= release {
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Inactive;
                    } else {
                        audio_out =
                            audio_in * ((1f64 - (self.active_time / release)) * sustain_amp);
                    }
                }
            }

            // Note: while it's technically incorrect to increment here, as it occurs between state transitions,
            // it prevents a bunch of handling of zero division and only increase the active time by an
            // insignificant value
            self.active_time += clock.time_delta;

            let mut value = self.out_audio_out.write().expect("RwLock is poisoned");
            *value = Some(audio_out);
        }
    }

    /// Return a module's ID
    fn get_id(&self) -> &String {
        &self.id
    }

    fn get_in_ports(&self) -> &Vec<String> {
        &self.input_ports
    }

    fn get_out_ports(&self) -> &Vec<String> {
        &self.output_ports
    }

    /// Return a reference to one of the module's input ports
    fn get_in_port_ref(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            "audio_in" => Some(self.in_audio.clone()),
            "trigger" => Some(self.in_trigger.clone()),
            "attack" => Some(self.in_attack.clone()),
            "decay" => Some(self.in_decay.clone()),
            "sustain" => Some(self.in_sustain.clone()),
            "release" => Some(self.in_release.clone()),
            _ => None,
        }
    }

    fn get_out_port_ref(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            "audio_out" => Some(self.out_audio_out.clone()),
            _ => None,
        }
    }

    /// Set the value of a module's input port
    fn set_in_port(&mut self, port_id: &str, out_port: IoPort) -> PortResult<String> {
        match port_id {
            "audio_in" => self.in_audio = out_port,
            "trigger" => self.in_trigger = out_port,
            "attack" => self.in_attack = out_port,
            "decay" => self.in_decay = out_port,
            "sustain" => self.in_sustain = out_port,
            "release" => self.in_release = out_port,
            _ => return Err(PortNotFoundError),
        }

        Ok(format!("{}: Set port {}\n", self.get_id(), port_id))
    }

    fn get_module_order(&self) -> Option<u64> {
        self.order
    }

    fn set_module_order(&mut self, new_order: Option<u64>) {
        self.order = new_order;
    }
}
