use alsa::{Direction, Result};
use alsa::seq::{Addr, PortCap, PortType, Seq, PortSubscribe};
use std::ffi::CString;

pub mod message_status;
pub mod midi_config;
// pub mod midi_tx;
pub mod midi_rx;

/// Create a new client/handler for seq
fn open_client() -> Result<Seq> {
    // Open seq
    let seq = Seq::open(None, Some(Direction::input()), true);

    seq
}

fn set_client_name(seq: &Seq, name: &str) -> Result<()> {
    let client_name = CString::new(name).expect("CString::new failed");

    seq.set_client_name(&client_name)
}

/// Create a port for communication with other clients
fn create_port(seq: &Seq, name: &str) -> Result<i32> {
    let port_name = CString::new(name).expect("CString::new failed");
    let res = seq.create_simple_port(
        &port_name,
        PortCap::WRITE | PortCap::SUBS_WRITE,
        PortType::MIDI_GENERIC
        );

    res
}

fn connect_ports(seq: &Seq, seq_port: i32, client: i32, port: i32) -> Result<()> {
    // Define port addresses
    // Address of virtual midi keyboard
    let snd_addr = Addr {
        client,
        port,
    };
    // Address of subscriber
    let seq_client_id = seq.client_id().expect("Failed to get client ID");
    let dst_addr = Addr {
        client: seq_client_id,
        port: seq_port,
    };

    // Create a subscriber from keyboar to application (yat)
    // Port subscribe
    let subscriber = PortSubscribe::empty().expect("Failed to create PortSubscribe");
    subscriber.set_sender(snd_addr);
    subscriber.set_dest(dst_addr);
    subscriber.set_queue(1);
    subscriber.set_time_update(true);
    subscriber.set_time_real(true);

    // Have yat subscribe to keyboard port
    seq.subscribe_port(&subscriber)
}

pub fn init_midi_rx() -> Result<Seq> {
    let seq: Seq = match open_client() {
        Ok(seq) => seq,
        Err(err) => panic!("Failed to create a new MIDI client for seq: {err}"),
    };

    let name = "yat";
    set_client_name(&seq, name).expect("failed to set client name");

    let seq_port = create_port(&seq, name).expect("Failed to create a port");

    match connect_ports(&seq, seq_port, 128, 0) {
        Ok(_) => Ok(seq),
        Err(err) => panic!("Failed to instantiate MIDI receive device: {err}"),
    }
}
