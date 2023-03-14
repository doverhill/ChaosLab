#![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_filesystem;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use library_chaos::*;
use protocol_console::*;
use protocol_filesystem::*;

struct Environment {
    pub path: String,
}

struct Command {
    pub name: String,
    pub short_help_text: String,
    pub handler: dyn FnMut(&Environment, &String) -> CommandResult,
}

enum CommandResult {
    Success,
    Fail
}

fn main() {
    // set up process and server state
    let mut process = StormProcess::new("Cluido").unwrap();

    // connect to console service
    let mut console_client = ConsoleClient::connect_first(&mut process).unwrap();

    console_client.write_text(&WriteTextParameters {
        text: format!("Welcome to Chaos and Cluido {}\n\n", env!("CARGO_PKG_VERSION")),
    });

    let mut environment = Environment {
        path: "//".to_string()
    };

    let commands: Vec<Command> = vec![
        Command { name: "exit".to_string(), short_help_text: "Exit cluido".to_string(), handler: exit_handler },
        Command { name: "help".to_string(), short_help_text: "Show list of commands".to_string(), handler: help_handler },
        Command { name: "list-items".to_string(), short_help_text: "List items in current path".to_string(), handler: list_items_handler },
        Command { name: "set-path".to_string(), short_help_text: "Change current path".to_string(), handler: set_path_handler },
    ];
    
    'repl: loop {
        console_client.write_text(&WriteTextParameters { text: "> ".to_string() });

        let command = read_line(&mut process, &mut console_client);
        console_client.write_text(&WriteTextParameters { text: "\n".to_string() });

        let exit = handle_command(&mut process, &mut console_client, &mut environment, &commands, command);

        if exit {
            break 'repl;
        }
    }

    process.end();
}

fn exit_handler(environment: &mut Environment, command: &String) -> CommandResult {
    CommandResult::Fail
}

fn help_handler(environment: &mut Environment, command: &String) -> CommandResult {
    CommandResult::Fail
}

fn list_items_handler(environment: &mut Environment, command: &String) -> CommandResult {
    CommandResult::Fail
}

fn set_path_handler(environment: &mut Environment, command: &String) -> CommandResult {
    CommandResult::Fail
}

fn handle_command(process: &mut StormProcess, console_client: &mut ConsoleClient, environment: &mut Environment, commands: &Vec<Command>, command_line: String) -> bool {
    let mut parts = command_line.split_whitespace();

    if let Some(command) = parts.next() {
        match command {
            "help" => {
                console_client.write_text(&WriteTextParameters {
                    text: "This is a help string\n".to_string(),
                });
            }
            "list-items" => {
                let mut filesystem_client = FilesystemClient::connect_first(process).unwrap();
                let pattern = "*".to_string();
                let recursive = false;
                let list_result = filesystem_client.list_objects(process, &ListObjectsParameters { path: environment.path.clone(), pattern: pattern, recursive: recursive }).unwrap();
                for result in list_result.objects.iter() {
                    match result {
                        ListObjectsReturnsObjectsEnum::TypeDirectory(directory) => {
                            console_client.write_text(&WriteTextParameters { text: format!("D {}\n", directory.name) });
                        }
                        ListObjectsReturnsObjectsEnum::TypeFile(file) => {
                            console_client.write_text(&WriteTextParameters { text: format!("F {} size: {}\n", file.name, file.size) });
                        }
                    }
                }
            }
            "set-path" => {

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
                ConsoleClientChannelEvent::ServerDisconnected(_) => {
                    // not implemented
                }
                ConsoleClientChannelEvent::ServerEvent(_, event) => {
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
