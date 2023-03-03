use std::rc::Rc;
use core::cell::RefCell;
use library_chaos::{ServiceHandle, ChannelHandle, StormProcess};
use protocol_console::*;
use std::collections::HashMap;

struct ClientState {
}

impl ClientState {
    pub fn new() -> Self {
        ClientState { }
    }
}

pub struct ServerState {
    process: Rc<RefCell<StormProcess>>,
    console_server: Rc<RefCell<ConsoleServer>>,
    clients: HashMap<ChannelHandle, ClientState>,
}

impl ServerState {
    pub fn new(process: Rc<RefCell<StormProcess>>, console_server: Rc<RefCell<ConsoleServer>>) -> Self {
        Self { 
            process: process,
            console_server: console_server,
            clients: HashMap::new() 
        }
    }

    pub fn add_client(&mut self, handle: ChannelHandle) {
        self.clients.insert(handle, ClientState::new());
    }

    pub fn remove_client(&mut self, handle: ChannelHandle) {
        self.clients.remove(&handle);
    }

    pub fn get_first_client_handle(&self) -> Option<&ChannelHandle> {
        self.clients.keys().next()
    }
}

impl ConsoleServerObserver for ServerState {
    fn handle_console_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        println!("handle_console_client_connected");
        self.add_client(channel_handle);
    }

    fn handle_console_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        println!("handle_console_client_disconnected");
        self.remove_client(channel_handle);
    }

    fn handle_console_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, call_id: u64, request: ConsoleServerRequest) {
        println!("handle_console_request");

        match request {
            ConsoleServerRequest::WriteText(parameters) => {
                println!("console::write_text: {}", parameters.text);
            },
            ConsoleServerRequest::GetCapabilities => {
                let capabilities = GetCapabilitiesReturns {
                    is_framebuffer: true,
                    framebuffer_size: Size {
                        width: 1024,
                        height: 768
                    },
                    text_size: Size {
                        width: 80,
                        height: 50
                    }
                };
                self.console_server.borrow_mut().get_capabilities_reply(channel_handle, call_id, &capabilities);
            }
            _ => {}
        }
    }
}
