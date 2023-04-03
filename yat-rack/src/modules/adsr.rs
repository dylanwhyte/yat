use std::sync::{Arc, RwLock, Weak};

use crate::clock::Clock;
use crate::modules::io_module::IoModule;
use crate::types::{PortNotFoundError, PortResult, SampleType};
use crate::in_port::InPort;
use crate::out_port::OutPort;

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

    /// The signal which informs the envelope generator that a note is
    /// active
    in_gate: InPort,

    /// The time (seconds) until the signal's full amplitude is reached
    in_attack: InPort,

    /// The time (seconds) until the signal decays to the sustain amplitude
    in_decay: InPort,

    /// The percentage amplitude to remain at, while a not is held
    in_sustain: InPort,

    /// The time (seconds) for the amplitude to fully decay after a
    /// note is released
    in_release: InPort,

    /// The modulated output signal of the envelope generator
    out_signal_out: OutPort,

    /// The current time for which a note has been active
    active_time: SampleType,

    /// The time at which a note was triggered
    gate_trigger_time: SampleType,

    /// Which phase of processing the ADSR is in
    adsr_state: AdsrState,

    /// Time of the rack's clock
    clock: Arc<RwLock<Clock>>,

    /// Signal level at the time the gate is released. This is used to
    /// smoothly transition from any state in the ADSR to zero.
    pre_release_sig: SampleType,
}

impl Adsr {
    /// Create a new, unordered IoModule
    pub fn new(id: String, clock: Arc<RwLock<Clock>>) -> Self {
        let order = None;
        let input_ports = vec!["gate".into(), "attack".into(),
                                "decay".into(), "sustain".into(),
                                "release".into()];
        let output_ports = vec!["signal_out".into()];

        let time_delta = clock.read().expect("RwLock is poisoned").time_delta;

        let in_gate = InPort::new("gate".into(), 0.0, 1.0, 0.0);
        let in_attack = InPort::new("attack".into(), 0.0, 1.0, time_delta);
        let in_decay = InPort::new("decay".into(), 0.0, 1.0, time_delta);
        let in_sustain = InPort::new("sustain".into(), 0.0, 1.0, 1.0);
        let in_release = InPort::new("release".into(), 0.0, 1.0, time_delta);
        let out_signal_out = OutPort::new("signal_out".into());

        let active_time = 0f64;
        let gate_trigger_time = 0f64;
        let adsr_state = AdsrState::Inactive;

        let pre_release_sig = 0f64;

        Self {
            id,
            order,
            input_ports,
            output_ports,
            in_gate,
            in_attack,
            in_decay,
            in_sustain,
            in_release,
            out_signal_out,
            active_time,
            gate_trigger_time,
            adsr_state,
            clock,
            pre_release_sig,
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
        let gate_active = self.in_gate.get_value() != 0.0;

        // no key is active
        if (self.adsr_state == AdsrState::Inactive) && (!gate_active) {
            self.out_signal_out.set_value(0.0);
        } else {
            let clock = self.clock.read().expect("RwLock is poisoned");

            let sustain_amp = self.in_sustain.get_value();

            // This makes sense as a default value, in case attack and decay are zero
            let mut signal_out = sustain_amp;

            match self.adsr_state {
                AdsrState::Inactive => {
                    if gate_active {
                        self.gate_trigger_time = clock.get_current_time().unwrap();
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Attack;
                    }
                }
                AdsrState::Attack => {
                    // Transition to max amplitude, and change state to decay after time
                    // If released, go straight to that
                    // Effectively set to zero, but avoiding potential zero division
                    let attack = self.in_attack.get_value();

                    if !gate_active {
                        self.pre_release_sig = self.active_time / attack;
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Release;
                    } else if self.active_time >= attack {
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Decay;
                    } else {
                        // Gradually increase amplitude to max
                        signal_out = self.active_time / attack;
                    }
                }
                AdsrState::Decay => {
                    // Transition to sustain amplitude
                    // Effectively set to zero, but avoiding potential zero division
                    let decay = self.in_decay.get_value();

                    if !gate_active {
                        self.pre_release_sig =
                            1f64 - ((self.active_time * (1f64 - sustain_amp)) / decay);
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Release;
                    } else if self.active_time >= decay {
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Sustain;
                    } else {
                        let sustain_amp = self.in_sustain.get_value();

                        // Decay to sustain amplitude
                        signal_out = 1f64 - ((self.active_time * (1f64 - sustain_amp)) / decay);
                    }
                }
                AdsrState::Sustain => {
                    // Output at sustain level while gate is active
                    if !gate_active {
                        self.pre_release_sig = sustain_amp;
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Release;
                    } else {
                        signal_out = sustain_amp;
                    }
                }
                AdsrState::Release => {
                    if gate_active {
                        self.active_time = 0f64;
                        self.adsr_state = AdsrState::Attack;
                    } else {
                        // Effectively set to zero, but avoiding potential zero division
                        let release = self.in_release.get_value();

                        // Decay to zero
                        if self.active_time >= release {
                            self.active_time = 0f64;
                            self.adsr_state = AdsrState::Inactive;
                        } else {
                            signal_out = (1f64 - (self.active_time / release)) * self.pre_release_sig;
                        }
                    }
                }
            }

            // Note: while it's technically incorrect to increment here,
            // as it occurs between state transitions,
            // it prevents a bunch of handling of zero division and
            // only increase the active time by an insignificant value
            self.active_time += clock.time_delta;

            self.out_signal_out.set_value(signal_out);
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
    fn has_port_with_id(&self, port_id: &str) -> bool {
        match port_id {
            "gate" | "attack" | "decay"
                | "sustain" | "release" => true,
            _ => false,
        }
    }

    fn get_out_port_ref(&self, port_id: &str) -> Option<&OutPort> {
        match port_id {
            "signal_out" => Some(&self.out_signal_out),
            _ => None,
        }
    }

    /// Set the value of a module's input port
    fn set_in_port(&mut self, port_id: &str, out_port_ref: Weak<RwLock<Option<SampleType>>>) -> PortResult<String> {
        match port_id {
            "gate" => self.in_gate.set_value(out_port_ref),
            "attack" => self.in_attack.set_value(out_port_ref),
            "decay" => self.in_decay.set_value(out_port_ref),
            "sustain" => self.in_sustain.set_value(out_port_ref),
            "release" => self.in_release.set_value(out_port_ref),
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
