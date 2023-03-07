#![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_console;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use library_chaos::*;
use protocol_console::*;

fn main() {
    // set up process and server state
    let mut process = StormProcess::new("Cluido").unwrap();

    // connect to console service
    let mut console_client = ConsoleClient::connect_first(&mut process).unwrap();

    console_client.write_text(&WriteTextParameters {
        text: format!("Welcome to Chaos and Cluido {}\n\n", env!("CARGO_PKG_VERSION")),
    });

    'repl: loop {
        console_client.write_text(&WriteTextParameters { text: "> ".to_string() });

        let command = read_line(&mut process, &mut console_client);
        console_client.write_text(&WriteTextParameters { text: "\n".to_string() });

        let exit = handle_command(&mut process, &mut console_client, command);

        if (exit) {
            break 'repl;
        }
    }

    process.end();
}

fn handle_command(process: &mut StormProcess, console_client: &mut ConsoleClient, command_line: String) -> bool {
    let mut parts = command_line.split_whitespace();

    if let Some(command) = parts.next() {
        match command {
            "help" => {
                console_client.write_text(&WriteTextParameters {
                    text: "This is a help string\n".to_string(),
                });
            }
            "exit" => {
                return true;
            }
            _ => {
                console_client.write_text(&WriteTextParameters {
                    text: "Unknown command. Try the help command.\n".to_string(),
                });
            }
        }
    }

    false
}

fn read_line(process: &mut StormProcess, console_client: &mut ConsoleClient) -> String {
    let mut line = String::new();

    console_client.save_text_cursor_position();

    'read: loop {
        let event = StormProcess::wait_for_event().unwrap();
        console_client.register_event(event);
        while let Some(console_client_event) = console_client.get_event(process) {
            match console_client_event {
                ConsoleClientChannelEvent::ServerDisconnected(channel_handle) => {
                    // not implemented
                }
                ConsoleClientChannelEvent::ServerEvent(channel_handle, event) => {
                    match event {
                        ConsoleClientEvent::CharacterInput(parameters) => {
                            line.push(char::from_u32(parameters.character as u32).unwrap());
                            console_client.load_text_cursor_position();
                            console_client.write_text(&WriteTextParameters { text: line.clone() });
                        }
                        ConsoleClientEvent::KeyPressed(parameters) => {
                            match parameters.key_code {
                                KeyCode::Enter => {
                                    break 'read;
                                }
                                KeyCode::Backspace => {
                                    line.pop();
                                    console_client.load_text_cursor_position();
                                    console_client.write_text(&WriteTextParameters { text: line.clone() });
                                    console_client.write_text(&WriteTextParameters { text: " ".to_string() });
                                }
                                _ => {
                                    // FIXME implement Home, End, Arrows and Delete
                                }
                            }
                        }
                        _ => {
                            // not interested in any other events from console server
                        }
                    }
                }
            }
        }
    }

    line
}
