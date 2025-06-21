use alsa::seq::Seq;

/// MIDI Transmitter - Originates messages in MIDI format
pub struct MidiTx {
    seq: Seq,
}

impl MidiTx {
    // Generic MIDI message
    // fn send_midi_msg(
    //     &self,
    //     msg: MessageStatus,
    //     channel: Option<u8>,
    //     data_0: Option<u8>,
    //     data_1: Option<u8>
    //     )
    // {
    //     let data_0 = match data_0 {
    //         Some(data) => Some(data & 0x7F),
    //         None => None,
    //     };

    //     let data_1 = match data_1 {
    //         Some(data) => Some(data & 0x7F),
    //         None => None,
    //     };

    //     // let msg = MidiMessage::new(msg, channel, [data_0, data_1]);
    //     // let _ = self.tx.send(msg);
    // }

    // Channel Voic Messages:
    // fn send_note_off(&self, channel: u8, pitch: u8, velocity: u8) {
    //     let msg = MidiMessage::new(
    //         MessageStatus::NoteOff,
    //         Some(channel),
    //         [Some(pitch), Some(velocity)]
    //         );
    //     self.tx.send(msg);
    // }

    // fn send_note_on(&self, channel: u8, pitch: u8, velocity: u8) {
    //     let msg = MidiMessage::new(
    //         MessageStatus::NoteOn,
    //         Some(channel),
    //         [Some(pitch), Some(velocity)]
    //         );
    //     let _ = self.tx.send(msg);
    // }

    // fn send_key_pressure(&self, channel: u8, key: u8, pressure: u8) {
    //     let msg = MidiMessage::new(
    //         MessageStatus::KeyPressure,
    //         Some(channel),
    //         [Some(key), Some(pressure)]
    //         );
    //     let _ = self.tx.send(msg);
    // }

    // fn send_controller_change(&self, channel: u8, controller: u8, value: u8) {
    //     let msg = MidiMessage::new(
    //         MessageStatus::ControllerChange,
    //         Some(channel),
    //         [Some(controller), Some(value)]
    //         );
    //     let _ = self.tx.send(msg);
    // }

    // fn send_program_change(&self, channel: u8, preset: u8) {
    //     let msg = MidiMessage::new(
    //         MessageStatus::ProgramChange,
    //         Some(channel),
    //         [Some(preset), None]
    //         );
    //     let _ = self.tx.send(msg);
    // }

    // fn send_channel_pressure(&self, channel: u8, pressure: u8) {
    //     let msg = MidiMessage::new(
    //         MessageStatus::ChannelPressure,
    //         Some(channel),
    //         [Some(pressure),
    //         None]
    //         );
    //     let _ = self.tx.send(msg);
    // }

    // fn send_pitch_bend(&self, channel: u8, bend_lsb: u8, bend_msb: u8) {
    //     let msg = MidiMessage::new(
    //         MessageStatus::ControllerChange,
    //         Some(channel),
    //         [Some(bend_lsb), Some(bend_msb)]
    //         );
    //     let _ = self.tx.send(msg);
    // }
    // ------------------------


    // System Exclusive Messages:

    // ------------------------


    // System Common Messages:

    // ------------------------


    // System Real Time Messages:

    // ------------------------
}


