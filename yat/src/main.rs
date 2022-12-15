use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};

use crossterm::event::KeyEventKind;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use unicode_width::UnicodeWidthStr;

use yat_rack::modules::audio_out::AudioOut;
use yat_rack::rack::Rack;
use yat_rack::types::SampleType;

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Report hold & release events
    execute!(
        stdout,
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
            //KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES,
        )
    )
    .unwrap();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = app.run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;

    execute!(io::stdout(), PopKeyboardEnhancementFlags).unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

enum InputMode {
    Normal,
    Control,
    Editing,
}

/// App holds the state of the application
pub struct App {
    /// The rack which encloses modules
    rack: Arc<Mutex<Rack>>,
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// Command history
    commands: Vec<String>,
    /// History of recorded messages
    messages: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            rack: Arc::new(Mutex::new(Rack::new())),
            input: String::new(),
            input_mode: InputMode::Normal,
            commands: Vec::new(),
            messages: Vec::new(),
        }
    }
}

impl App {
    pub fn run_app<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        // setup audio and interface
        let (audio_out, audio_rx) = AudioOut::new(String::from("audio_out"));
        let rack = self.rack.clone();

        // Add the audio_out module by defualt
        // TODO: Make sure there is only one of these for now
        { rack.lock().unwrap().add_module(Arc::new(Mutex::new(audio_out))); }

        App::setup_audio_thread(audio_rx);

        let c_rack_ref = Arc::clone(&rack);
        let s_rack_ref = Arc::clone(&rack);

        let (quit_tx, quit_rx) = mpsc::sync_channel(1);
        thread::scope(|c_scope| {
            //let running_pair = Arc::clone(&s_rack_ref.lock().unwrap().running);
            c_scope.spawn(move || {
                // TODO: Use std::sync::Convar to actually block CPU
                loop {
                    while *s_rack_ref.lock().unwrap().running.get_mut() {
                        {
                            s_rack_ref.lock().unwrap().process_module_chain();
                        }
                    }
                    match quit_rx.try_recv() {
                        Ok(_) => break,
                        Err(_) => continue,
                    }
                }
            });

            loop {
                terminal.draw(|f| self.ui(f))?;

                if let Event::Key(key) = event::read()? {
                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('c') => {
                                self.input_mode = InputMode::Control;
                            }
                            KeyCode::Char('e') => {
                                self.input_mode = InputMode::Editing;
                            }
                            KeyCode::Char('q') => {
                                self.messages.push("Quiting...\n".into());
                                c_scope.spawn(|| c_rack_ref.lock().unwrap().stop());
                                quit_tx.send(true).unwrap();
                                return Ok::<(), io::Error>(());
                            }
                            _ => {}
                        },
                        InputMode::Control => {
                            match key.kind {
                                KeyEventKind::Press => {
                                    match key.code {
                                        KeyCode::Char(key_code) => {
                                            //{ press_messages.lock().unwrap().push("Press\n".into()); }
                                            // Fills buffer
                                            {
                                                c_rack_ref
                                                    .lock()
                                                    .unwrap()
                                                    .send_control_key(key_code);
                                            }
                                            {
                                                c_rack_ref.lock().unwrap().send_control_key(' ');
                                            }
                                        }
                                        KeyCode::Esc => {
                                            self.input_mode = InputMode::Normal;
                                            //*exit_control_clone.lock().unwrap().get_mut() = true;
                                        }
                                        _ => {}
                                    }
                                }
                                KeyEventKind::Repeat => {}
                                KeyEventKind::Release => {
                                    c_rack_ref.lock().unwrap().send_control_key('*');
                                }
                            }
                        }

                        InputMode::Editing => match key.kind {
                            KeyEventKind::Repeat => {}
                            KeyEventKind::Release => {}
                            KeyEventKind::Press => {
                                match key.code {
                                    KeyCode::Enter => {
                                        self.commands.push(self.input.drain(..).collect());

                                        if self.commands.last().unwrap() == "clear messages" {
                                            self.messages.clear();
                                        } else if self.commands.last().unwrap() == "quit" {
                                            self.messages.push("Quiting...\n".into());
                                            c_scope.spawn(|| c_rack_ref.lock().unwrap().stop());
                                            quit_tx.send(true).unwrap();
                                            return Ok(());
                                        } else if self.commands.last().unwrap() == "stop" {
                                            self.messages.push("stopping...\n".into());
                                            c_scope.spawn(|| c_rack_ref.lock().unwrap().stop());
                                        } else if self.commands.last().unwrap() == "run" {
                                            self.messages.push("running...\n".into());
                                            c_scope.spawn(|| c_rack_ref.lock().unwrap().run());
                                        } else if self.commands.last().unwrap().starts_with("add") {
                                            let mut split_command =
                                                self.commands.last().unwrap().split(" ");
                                            if self.commands.last().unwrap().split(" ").count() != 3
                                            {
                                                self.messages.push(
                                                    "usage: add <module_type> <module_id>\n".into(),
                                                );
                                            } else {
                                                let module_type = split_command.nth(1).unwrap();
                                                let module_id = split_command.nth(0).unwrap();
                                                match c_rack_ref
                                                    .lock()
                                                    .unwrap()
                                                    .add_module_type(module_type, module_id)
                                                {
                                                    Ok(res) => self.messages.push(res),
                                                    Err(e) => self.messages.push(format!(
                                                        "Failed to add {} {}: {}",
                                                        module_type, module_id, e
                                                    )),
                                                }
                                            }
                                        } else if self
                                            .commands
                                            .last()
                                            .unwrap()
                                            .starts_with("connect")
                                        {
                                            let mut split_command =
                                                self.commands.last().unwrap().split(" ");
                                            if self.commands.last().unwrap().split(" ").count() == 5
                                            {
                                                let out_module_id = split_command.nth(1).unwrap();
                                                let out_port_id = split_command.nth(0).unwrap();
                                                let in_module_id = split_command.nth(0).unwrap();
                                                let in_port_id = split_command.nth(0).unwrap();
                                                match c_rack_ref.lock().unwrap().connect_modules(
                                                    out_module_id,
                                                    out_port_id,
                                                    in_module_id,
                                                    in_port_id,
                                                ) {
                                                    Ok(message) => self.messages.push(message),
                                                    Err(e) => self.messages.push(format!(
                                                        "Failed to connect modules: {}",
                                                        e
                                                    )),
                                                }
                                                //} else if self.commands.last().unwrap().split(" ").count() == 4 {
                                                //// TODO: Add proper error handling
                                                //let ctrl_id = split_command.nth(1).unwrap();
                                                //let in_module_id = split_command.nth(0).unwrap();
                                                //let in_port_id = split_command.nth(0).unwrap();
                                                //c_rack_ref.lock().unwrap().connect_ctrl(ctrl_id, in_module_id, in_port_id);
                                            } else {
                                                self.messages.push("usagee: connect <out_module_id> <out_port_id> <in_module> <in_module_id>\n".into());
                                            }
                                        } else if self.commands.last().unwrap().starts_with("set") {
                                            let mut split_command =
                                                self.commands.last().unwrap().split(" ");
                                            if self.commands.last().unwrap().split(" ").count() != 4
                                            {
                                                self.messages.push(
                                                    "usage: set <ctrl_id> <port_id> <value>\n"
                                                        .into(),
                                                );
                                            } else {
                                                let ctrl_id = split_command.nth(1).unwrap();
                                                let port_id = split_command.nth(0).unwrap();

                                                // TODO: Add proper error handling
                                                let value =
                                                    split_command.nth(0).unwrap().parse().ok();

                                                // TODO: Add proper error handling
                                                match c_rack_ref
                                                    .lock()
                                                    .unwrap()
                                                    .set_ctrl_value(ctrl_id, port_id, value)
                                                {
                                                    Ok(res) => self.messages.push(res),
                                                    Err(e) => self.messages.push(format!(
                                                        "Failed to update control: {}",
                                                        e
                                                    )),
                                                }
                                            }
                                        } else if self.commands.last().unwrap().starts_with("focus")
                                        {
                                            let mut split_command =
                                                self.commands.last().unwrap().split(" ");

                                            if self.commands.last().unwrap().split(" ").count() != 2
                                            {
                                                self.messages
                                                    .push("usage: focus <ctrl_id>\n".into());
                                            } else {
                                                let ctrl_id = split_command.nth(1).unwrap();

                                                match c_rack_ref
                                                    .lock()
                                                    .unwrap()
                                                    .set_focus_control(ctrl_id)
                                                {
                                                    Ok(res) => self.messages.push(res),
                                                    Err(e) => self.messages.push(format!(
                                                        "Failed to focus control: {}",
                                                        e
                                                    )),
                                                }
                                            }
                                        } else if self.commands.last().unwrap().starts_with("print")
                                        {
                                            if self.commands.last().unwrap().split(" ").count() == 2
                                            {
                                                match self.commands.last().unwrap().split(" ").nth(1).unwrap() {
                                                    "modules" => self.messages.push(c_rack_ref.lock().unwrap().print_modules()),
                                                    "module-order" => self.messages.push(c_rack_ref.lock().unwrap().print_module_order()),
                                                    "ports" => self.messages.push(c_rack_ref.lock().unwrap().print_ports(None)),
                                                    _ => self.messages.push("usage: print <modules|module-order|ports [module_id]>".into()),
                                                }
                                            } else if self
                                                .commands
                                                .last()
                                                .unwrap()
                                                .split(" ")
                                                .count()
                                                == 3
                                            {
                                                match self.commands.last().unwrap().split(" ").nth(1).unwrap() {
                                                    "ports" => self.messages.push(c_rack_ref.lock().unwrap().print_ports(
                                                            Some(self.commands.last().unwrap().split(" ").nth(2).unwrap()))),
                                                    _ => self.messages.push("usage: print <modules|module-order|ports [module_id]>".into()),
                                                }
                                            } else {
                                                self.messages.push("usage: print <modules|module-order|ports [module_id]>\n".into());
                                            }
                                        } else {
                                            let mut message = "".to_string();
                                            message.push_str("Unknown command: ".into());
                                            message.push_str(self.commands.last().unwrap());
                                            message.push_str("\n");
                                            self.messages.push(message);
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        self.input.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        self.input.pop();
                                    }
                                    KeyCode::Esc => {
                                        self.input_mode = InputMode::Normal;
                                    }
                                    _ => {}
                                }
                            }
                        },
                    }
                }
            }
        })?;

        Ok(())
    }

    fn ui<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(90),
                    //Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.size());

        let top_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(3)].as_ref())
            .split(chunks[0]);

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to start editing or "),
                    Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to enter control mode."),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    Span::raw("Press "),
                    Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to stop editing, "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to record the message"),
                ],
                Style::default(),
            ),
            InputMode::Control => (
                vec![
                    Span::raw("Press "),
                    Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit control mode, "),
                    Span::raw("or other keys to alter the control"),
                ],
                Style::default(),
            ),
        };
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text);
        f.render_widget(help_message, top_chunks[0]);

        // NEXT WIDGET
        let input = Paragraph::new(self.input.as_ref())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
                InputMode::Control => Style::default().fg(Color::Blue),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, top_chunks[1]);

        // NEXT WIDGET
        match self.input_mode {
            InputMode::Normal => {}
            InputMode::Editing => {
                // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                f.set_cursor(
                    // Put cursor past the end of the input text
                    top_chunks[1].x + self.input.width() as u16 + 1,
                    // Move one line down, from the border to the input line
                    top_chunks[1].y + 1,
                )
            }
            InputMode::Control => {}
        }

        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(chunks[1]);

        let commands: Vec<ListItem> = self
            .commands
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
                ListItem::new(content)
            })
            .collect();

        let command_history = List::new(commands).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Histroy"),
        );
        f.render_widget(command_history, bottom_chunks[0]);

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
                ListItem::new(content)
            })
            .collect();

        let messages =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
        f.render_widget(messages, bottom_chunks[1]);

        let mut modules = { self.rack.lock().unwrap().print_modules() };
        modules.push_str(&self.rack.lock().unwrap().print_module_order());
        modules.push_str(&self.rack.lock().unwrap().print_ports(None));
        // NEXT WIDGET
        // Bottom right block with styled left and right border
        let module_list = Paragraph::new(modules)
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title("Modules"));
        f.render_widget(module_list, bottom_chunks[2]);
    }

    fn setup_audio_thread(audio_rx: Receiver<SampleType>) {
        //-> IoPort {

        let _ = thread::spawn(move || {
            let host = cpal::default_host();
            let device = host.default_output_device().expect("no device available");
            let config = device.default_output_config().unwrap();

            let _ = match config.sample_format() {
                SampleFormat::F32 => App::run::<f32>(&device, &config.into(), audio_rx).unwrap(),
                SampleFormat::I16 => App::run::<i16>(&device, &config.into(), audio_rx).unwrap(),
                SampleFormat::U16 => App::run::<u16>(&device, &config.into(), audio_rx).unwrap(),
            };
        });
    }

    // Build output stream and play audio
    fn run<T: Sample>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        audio_rx: Receiver<SampleType>,
    ) -> Result<(), Box<dyn Error>> {
        let channels = config.channels as usize;

        let err_fn = |err| eprintln!("an error occurred on the stream: {}", err);

        // Build an output stream
        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let next_sample = match audio_rx.recv() {
                        Ok(sample) => sample,
                        Err(_) => break,
                    };
                    let value: T = cpal::Sample::from::<f32>(&next_sample);

                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            err_fn,
        )?;

        stream.play()?;

        std::thread::park();

        Ok(())
    }
}
