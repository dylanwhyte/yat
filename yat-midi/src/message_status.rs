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

