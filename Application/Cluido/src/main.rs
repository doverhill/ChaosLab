#![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_data;
extern crate protocol_filesystem;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use library_chaos::*;
use protocol_data::*;
use protocol_filesystem::*;

struct Environment {
    pub path: String,
}

struct Command {
    pub name: String,
    pub short_help_text: String,
    pub handler: Box<dyn Fn(&mut StormProcess, &mut DataClient, &mut Environment, &Vec<Command>, &String) -> CommandResult>,
}

enum CommandResult {
    None,
    Success,
    Fail,
    ExitCluido,
}

fn main() {
    // set up process and server state
    let mut process = StormProcess::new("Cluido").unwrap();

    // connect to console service
    let mut data_client = DataClient::connect_first(&mut process).unwrap();

    data_client.write_text(&WriteTextParameters {
        text: format!("Welcome to Chaos and Cluido {}\n\n", env!("CARGO_PKG_VERSION")),
    });

    let mut environment = Environment {
        path: "//".to_string()
    };

    let commands: Vec<Command> = vec![
        Command { name: "exit".to_string(), short_help_text: "Exit cluido".to_string(), handler: Box::new(exit_handler) },
        Command { name: "help".to_string(), short_help_text: "Show list of commands".to_string(), handler: Box::new(help_handler) },
        Command { name: "list-items".to_string(), short_help_text: "List items in current path".to_string(), handler: Box::new(list_items_handler) },
        Command { name: "set-path".to_string(), short_help_text: "Change current path".to_string(), handler: Box::new(set_path_handler) },
    ];
    
    'repl: loop {
        data_client.write_text(&WriteTextParameters { text: "> ".to_string() });

        let command = read_line(&mut process, &mut data_client);
        data_client.write_text(&WriteTextParameters { text: "\n".to_string() });

        let result = handle_command(&mut process, &mut data_client, &mut environment, &commands, command);

        match result {
            CommandResult::None => {}
            CommandResult::Success => {}
            CommandResult::Fail => {
                data_client.write_text(&WriteTextParameters { text: "Command caused an error\n".to_string() });
            }
            CommandResult::ExitCluido => {
                break 'repl;
            }
        }
    }

    process.end();
}

fn exit_handler(process: &mut StormProcess, data_client: &mut DataClient, environment: &mut Environment, commands: &Vec<Command>, command_line: &String) -> CommandResult {
    CommandResult::ExitCluido
}

fn help_handler(process: &mut StormProcess, data_client: &mut DataClient, environment: &mut Environment, commands: &Vec<Command>, command_line: &String) -> CommandResult {
    data_client.write_text(&WriteTextParameters {
        text: "Available commands are:\n".to_string(),
    });

    for command in commands.iter() {
        data_client.write_text(&WriteTextParameters { text: format!("{} - {}\n", command.name, command.short_help_text) });
    }

    CommandResult::Success
}

fn list_items_handler(process: &mut StormProcess, data_client: &mut DataClient, environment: &mut Environment, commands: &Vec<Command>, command_line: &String) -> CommandResult {
    let mut filesystem_client = FilesystemClient::connect_first(process).unwrap();
    let pattern = "*".to_string();
    let recursive = false;
    let list_result = filesystem_client.list_objects(process, &ListObjectsParameters { path: environment.path.clone(), pattern: pattern, recursive: recursive }).unwrap();
    for result in list_result.objects.iter() {
        match result {
            ListObjectsReturnsObjectsEnum::TypeDirectory(directory) => {
                data_client.write_text(&WriteTextParameters { text: format!("D {}\n", directory.name) });
            }
            ListObjectsReturnsObjectsEnum::TypeFile(file) => {
                data_client.write_text(&WriteTextParameters { text: format!("F {} size: {}\n", file.name, file.size) });
            }
        }
    }

    CommandResult::Success
}

fn set_path_handler(process: &mut StormProcess, data_client: &mut DataClient, environment: &mut Environment, commands: &Vec<Command>, command_line: &String) -> CommandResult {
    CommandResult::Fail
}

fn handle_command(process: &mut StormProcess, data_client: &mut DataClient, environment: &mut Environment, commands: &Vec<Command>, command_line: String) -> CommandResult {
    let mut parts = command_line.split_whitespace();

    if let Some(first_word) = parts.next() {
        if let Some(command) = commands.iter().find(|c| c.name == first_word) {
            (command.handler)(process, data_client, environment, commands, &command_line)
        }
        else {
            help_handler(process, data_client, environment, commands, &command_line)
        }
    }
    else {
        CommandResult::None
    }
}

fn read_line(process: &mut StormProcess, data_client: &mut DataClient) -> String {
    let mut line = String::new();

    data_client.save_text_cursor_position();

    'read: loop {
        let event = StormProcess::wait_for_event().unwrap();
        data_client.register_event(event);
        while let Some(console_client_event) = data_client.get_event(process) {
            match console_client_event {
                DataClientChannelEvent::ServerDisconnected(_) => {
                    // not implemented
                }
                DataClientChannelEvent::ServerEvent(_, event) => {
                    match event {
                        DataClientEvent::Characters(parameters) => {
                            for character in parameters.characters.iter() {
                                line.push(char::from_u32(*character as u32).unwrap());
                            }
                            data_client.load_text_cursor_position();
                            data_client.write_text(&WriteTextParameters { text: line.clone() });
                        }
                        DataClientEvent::Commands(parameters) => {
                            for command in parameters.commands.iter() {
                                match command {
                                    DataCommand::Enter => {
                                        break 'read;
                                    }
                                    DataCommand::Backspace => {
                                        line.pop();
                                        data_client.load_text_cursor_position();
                                        data_client.write_text(&WriteTextParameters { text: line.clone() });
                                        data_client.write_text(&WriteTextParameters { text: " ".to_string() });
                                    }
                                    _ => {
                                        // FIXME implement Home, End, Arrows and Delete
                                    }
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
