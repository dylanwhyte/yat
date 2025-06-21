use alsa::Rawmidi;

/// Holds midi configuration information for the project
struct MidiConfig {
    /// A raw midi handle
    device1: Option<Rawmidi>,
}
