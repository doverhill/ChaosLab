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

    console_client.write_text(&WriteTextParameters { text: format!("Welcome to Chaos and Cluido {}\n\n", env!("CARGO_PKG_VERSION")) });

    'repl: loop {
        console_client.write_text(&WriteTextParameters { text: "> ".to_string() });

        let command = read_line(&mut process, &mut console_client);

        console_client.write_text(&WriteTextParameters { text: command });
        //break 'repl;
    }

    process.end();
}

fn read_line(process: &mut StormProcess, console_client: &mut ConsoleClient) -> String {
    let mut line = String::new();

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
                        },
                        ConsoleClientEvent::KeyPressed(parameters) => 
                        {
                            match parameters.key_code {
                                KeyCode::Enter => {
                                    break 'read;
                                },
                                KeyCode::Backspace => {

                                },
                                _ => {
                                    // FIXME implement Home, End, Arrows and Delete
                                }
                            }
                        },
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