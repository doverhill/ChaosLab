use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use core::cell::RefCell;
use library_chaos::{ChannelHandle, ServiceHandle, StormProcess};
use protocol_console::{ConsoleClient, ConsoleClientEvent, ConsoleClientObserver};
use protocol_tornado::{TornadoServer, TornadoServerObserver, TornadoServerRequest};

struct ClientState {}

impl ClientState {
    pub fn new() -> Self {
        ClientState {}
    }
}

pub struct ServerState {
    process: Rc<RefCell<StormProcess>>,
    tornado_server: Rc<RefCell<TornadoServer>>,
    console_client: Rc<RefCell<ConsoleClient>>,
    clients: BTreeMap<ChannelHandle, ClientState>,
}

impl ServerState {
    pub fn new(process: Rc<RefCell<StormProcess>>, tornado_server: Rc<RefCell<TornadoServer>>, console_client: Rc<RefCell<ConsoleClient>>) -> Self {
        Self {
            process: process,
            tornado_server: tornado_server,
            console_client: console_client,
            clients: BTreeMap::new(),
        }
    }

    pub fn run(&mut self) {
    }

    pub fn add_client(&mut self, handle: ChannelHandle) {
        self.clients.insert(handle, ClientState::new());
    }

    pub fn remove_client(&mut self, handle: ChannelHandle) {
        self.clients.remove(&handle);
    }
}

impl ConsoleClientObserver for ServerState {
    fn handle_console_event(&mut self, channel_handle: ChannelHandle, event: ConsoleClientEvent) {
        match event {
            ConsoleClientEvent::PointerMoved(parameters) => {
                println!(
                    "moved to {} {}",
                    parameters.position.x, parameters.position.y
                );
            }
            _ => {}
        }
    }
}

impl TornadoServerObserver for ServerState {
    fn handle_tornado_client_connected(
        &mut self,
        service_handle: ServiceHandle,
        channel_handle: ChannelHandle,
    ) {
        println!("handle_tornado_client_connected");
        self.add_client(channel_handle);
    }

    fn handle_tornado_client_disconnected(
        &mut self,
        service_handle: ServiceHandle,
        channel_handle: ChannelHandle,
    ) {
        println!("handle_tornado_client_disconnected");
        self.remove_client(channel_handle);
    }

    fn handle_tornado_request(
        &mut self,
        service_handle: ServiceHandle,
        channel_handle: ChannelHandle,
        call_id: u64,
        request: TornadoServerRequest,
    ) {
        println!("handle_tornado_request");

        match request {
            TornadoServerRequest::SetRenderTree(parameters) => {
                println!("setting render tree");
            }
            _ => {}
        }
    }
}
