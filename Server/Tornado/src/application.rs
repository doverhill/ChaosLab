use alloc::collections::BTreeMap;
use library_chaos::{ChannelHandle, ServiceHandle, StormProcess};
use protocol_console::*;
use protocol_tornado::*;

struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct ServerApplication {
    process: StormProcess,
    tornado_server: TornadoServer,
    console_client: ConsoleClient,
    clients: BTreeMap<ChannelHandle, Client>,
}

impl ServerApplication {
    pub fn new(process: StormProcess, tornado_server: TornadoServer, console_client: ConsoleClient) -> Self {
        Self {
            process: process,
            tornado_server: tornado_server,
            console_client: console_client,
            clients: BTreeMap::new(),
        }
    }

    pub fn initialize(&mut self) {
        self.console_client.write_text(&WriteTextParameters { text: "hello console".to_string() });

        let console_info = self.console_client.get_capabilities(&self.process).unwrap();
        println!("tornado: {}x{}", console_info.framebuffer_size.width, console_info.framebuffer_size.height);
    }

    pub fn run(&mut self) {
        // main event loop
        loop {
            let event = StormProcess::wait_for_event().unwrap();
            self.console_client.register_event(event);
            while let Some(event) = self.console_client.get_event(&mut self.process) {
                self.process_console_client_event(event);
            };
            self.tornado_server.register_event(event);
            while let Some(event) = self.tornado_server.get_event(&mut self.process) {
                self.process_tornado_server_event(event);
            };
        }
    }

    fn process_console_client_event(&mut self, event: ConsoleClientChannelEvent) {
        // match event {
        //     ConsoleClientEvent::PointerMoved(parameters) => 
        //     {

        //     }
        // }
    }

    fn process_tornado_server_event(&mut self, event: TornadoServerChannelEvent) {
        match event {
            TornadoServerChannelEvent::ClientConnected(service_handle, channel_handle) => {
                self.clients.insert(channel_handle, Client::new());
            },
            TornadoServerChannelEvent::ClientDisconnected(service_handle, channel_handle) => {
                self.clients.remove(&channel_handle);
            },
            TornadoServerChannelEvent::ClientRequest(request) => {
                match request {
                    TornadoServerRequest::SetRenderTree(parameters) => {
                        println!("console::SetRenderTree");
                    },
                    _ => {
                        // not implemented
                    }
                }
            }
        }
    }

}

// impl TornadoServerObserver for ServerState {
//     fn handle_tornado_client_connected(
//         &mut self,
//         service_handle: ServiceHandle,
//         channel_handle: ChannelHandle,
//     ) {
//         println!("handle_tornado_client_connected");
//         self.add_client(channel_handle);
//     }

//     fn handle_tornado_client_disconnected(
//         &mut self,
//         service_handle: ServiceHandle,
//         channel_handle: ChannelHandle,
//     ) {
//         println!("handle_tornado_client_disconnected");
//         self.remove_client(channel_handle);
//     }

//     fn handle_tornado_request(
//         &mut self,
//         service_handle: ServiceHandle,
//         channel_handle: ChannelHandle,
//         call_id: u64,
//         request: TornadoServerRequest,
//     ) {
//         println!("handle_tornado_request");

//         match request {
//             TornadoServerRequest::SetRenderTree(parameters) => {
//                 println!("setting render tree");
//             }
//             _ => {}
//         }
//     }
// }
