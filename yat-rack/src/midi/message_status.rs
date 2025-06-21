pub enum MessageStatus {
    // Channel Voice Messages
    NoteOff = 0x80,
    NoteOn = 0x90,
    KeyPressure = 0xA0,
    ControllerChange = 0xB0, // data_0 values from 120-127
                             // correspond to channel mode messages
    ProgramChange = 0xC0,
    ChannelPressure = 0xD0,
    PitchBend = 0xE0,

    // System Exclusive Messages
    SystemExclusive = 0xF0, // SOX: Start of Sysstem Exclusive
                            // data_0 corresponds to vendor ID
                            // any amount of data bytes can be sent
                            // following data_0, for any purpose,
                            // as long as they have 0 in MSB
    EndOfSystemExclusive = 0xF7,

    // System Common Messages
    MidiTimeCodeQuarterFrame = 0xF1,
    SongPositionNumber = 0xF2,
    SongSelect = 0xF3,
    Undefined0xF4 = 0xF4,
    Undefined0xF5 = 0xF5,
    TuneRequest = 0xF6,

    // System Real Time Messages
    TimingClock = 0xF8,
    Undefined0xF9 = 0xF9,
    Start = 0xFA,
    Continue = 0xFB,
    Stop = 0xFC,
    Undefined0xFD = 0xFD,
    ActiveSensing = 0xFE,
    SystemReset = 0xFF,
}

impl TryFrom<u8> for MessageStatus {
    type Error = ();

    fn try_from(val: u8) -> Result<MessageStatus, ()> {
        match val {
            // channel voice messages
            0x80 => Ok(MessageStatus::NoteOff),
            0x90 => Ok(MessageStatus::NoteOn),
            0xA0 => Ok(MessageStatus::KeyPressure),
            0xB0 => Ok(MessageStatus::ControllerChange),
            0xC0 => Ok(MessageStatus::ProgramChange),
            0xD0 => Ok(MessageStatus::ChannelPressure),
            0xE0 => Ok(MessageStatus::PitchBend),
            //system exclusive messages
            0xF0 => Ok(MessageStatus::SystemExclusive),
            0xF7 => Ok(MessageStatus::EndOfSystemExclusive),
            // system control messages
            0xF1 => Ok(MessageStatus::EndOfSystemExclusive),
            0xF2 => Ok(MessageStatus::SongPositionNumber),
            0xF3 => Ok(MessageStatus::SongSelect),
            0xF4 => Ok(MessageStatus::Undefined0xF4),
            0xF5 => Ok(MessageStatus::Undefined0xF5),
            0xF6 => Ok(MessageStatus::TuneRequest),
            // system real time messages
            0xF8 => Ok(MessageStatus::TimingClock),
            0xF9 => Ok(MessageStatus::Undefined0xF9),
            0xFA => Ok(MessageStatus::Start),
            0xFB => Ok(MessageStatus::Continue),
            0xFC => Ok(MessageStatus::Stop),
            0xFD => Ok(MessageStatus::Undefined0xFD),
            0xFE => Ok(MessageStatus::ActiveSensing),
            0xFF => Ok(MessageStatus::SystemReset),
            _ => Err(()),
        }
    }
}
