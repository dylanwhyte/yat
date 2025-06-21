use alsa::seq::{EventType, EvNote, EvCtrl, EventData, Seq, Input};

/// MIDI Reciever - Accepts messages in MIDI format
/// and executes MIDI Commands
pub struct MidiRx {
	seq: Seq,
    status: Option<EventType>,
    channel: u8,
}

impl MidiRx {
    pub fn new(seq: Seq) -> Self {
        let status = None;
        let channel = 0;

        Self {
            seq,
            status,
            channel,
        }
    }

    pub fn get_input(&self) -> Input {
        self.seq.input()
    }

    pub fn receive_message<E: EventData>(&mut self) -> Option<MidiMessage<E>> {
        let mut input = self.seq.input();

        let event = match input.event_input() {
            Ok(ev) => ev,
            Err(err) => {
                // FIXME: Handle correctly
                panic!("Failed to read event..");
            }
        };

        let event_type = event.get_type();


        // TODO: Difference between EventData implementers
        // TODO: Are keypress and/or chanpress EvNote or EvCtrl?
        match event_type {
            EventType::Noteoff
                | EventType::Noteon
                | EventType::Keypress => {
                    println!("Status: {:?}", event_type);
                    // TODO: Add error handling
                    // TODO: Verify that these are all note events
                    let data: EvNote  = event.get_data().unwrap();

                    // self.channel = data.channel;
                    // self.note = data.note;

                    println!("EvNote: Received {:?} - channel: {}, note: {}, vel: {}, duration: {}",
                             event_type,
                             data.channel,
                             data.note,
                             data.velocity,
                             data.duration);

                    Some(MidiMessage::new(event_type, data))
                }
            EventType::Pgmchange
                | EventType::Controller
                | EventType::Pitchbend
                | EventType::Chanpress => {
                    // TODO: Add error handling
                    let data: EvCtrl = event.get_data().unwrap();

                    println!("EvCtrl: Received {:?} - channel: {}, param: {}, val: {} ",
                             event_type,
                             data.channel,
                             data.param,
                             data.value,
                             );
                    // Some(MidiMessage::new(event_type, data))
                    None
                }
            _ => None,
        }
    }
}

