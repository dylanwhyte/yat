# To Do

## Version 2

### Rack

- Rack should have no knowledge of modules -- or better put -- we should just be
  add generic `Impl IoModule`s to Rack.
- Redefine the entry point to the application and interface of the Rack
    - In general, it should have audio inputs and outputs, so that it's main
      priority is simply to calculate the chain.
- It might be better to redesign Rack to run in more of an Event Loop, than
  spawning multiple threads and having its logic handled all over the place.
    - E.g., Read events/commands, read inputs, update controls, fill buffer of
      audio.


### Interface

- The general interface should be flexible enough, so that multiple interface
types can be crafted, e.g., Standalone GUI, plugin formats, terminal UI.


### Extensible Modules

- Modules should be implemented in a separate crate and should be loadable via
  shared libraries. This enables user-extensibility, and dynamically loading new
  content.


### Misc.  Architecture

- Create Event Server, Audio Server, Config, Module loader, etc.
- Interfaces might deal more with the Event Server, which can contact other
  components in turn.
- Configuration files should also be designed for:
    - App settings (audio/midi config, etc.)
    - Project files
    - Rack files (in case support for multiple racks is added)
- Event commands:
    - stop
    - run
    - add <type> <id>
    - connect <out_mod_id> <out_port> <in_mod_id> <in_port>
    - set <ctrl_id> <port_id> <value>
    - focus <ctrl_id>
    - print <modules|module-order|ports [module_id]>


### TUI

Rather than defining each command individually in the TUI app, have a single
function that sends commands to an event server.


## Misc To do
- Set limits on controls and input, e.g. button, adsr values
	- For controls, this can be capped from the resective "Set value"
	  functions. Maybe an ability to alter some limits, e.g. knob, should
	  be considered too.
	- For inputs, maybe this should be managed in the process function, or
	  maybe the user should have control.
- Look for refactor oportunities
- Do more error handling

## App
- Fix key input so that shift key works as intended (caps and special chars)
- Create some kind of midi interface that sits between the keyboard and yat
  controls, so that controls can operate entirely via midi.

## Rack
- Fix module ordering algorithm

## Module
- Change input ports to be weak references. This makes sense, as the
  input-output system is a one-to-many, wherein if the input is removed, the
  outputs should not retain the dead reference.
  - Input ports should be an option value, so that an Arc containing None is
  not required for empty ports.

## MIDI
- Can I remove the alsa dependency from yat-rack and do everything through
  yat-midi?
- Think of how MIDI is handled in project.
	- Using Linux's raw midi interface rather than seq midi interface may
	  allow for a cleaner, separate midi crate, without a dependency on
	  alsa-rs in yat-rack.


## Binary
- Package for debian/ubuntu and Arch
- Set real-time priority
