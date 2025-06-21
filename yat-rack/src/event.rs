use hashbrown::HashMap;

/// - Control
///     - MIDI
/// - Add module ("add")
///     add <module_type> <module_id>
/// - Connect modules ("connect")
///     connect <out_module_id> <out_port_id> <in_module> <in_module_id>
/// - Disconnect modules
/// - Connect control to module ("connect")
/// - Set focus control ("focus")
///     set <ctrl_id> <port_id> <value>
/// - Set control value
///     set <ctrl_id> <port_id> <value>
/// - Print ("print/info")
///     set <ctrl_id> <port_id> <value>
///     - ports
///     - modules
///     - connections
///     - module order
/// - Process chain
/// - Start ("run")
/// - Stop ("stop")
/// - Unknown command
pub enum Event {
    Midi(u8, u8, u8),
    Command(String, HashMap<String, String>),
}

