use library_chaos::*;
use protocol_filesystem::*;

pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct ServerApplication {
    process: StormProcess,
    filesystem_server: FilesystemServer,
    clients: ClientStore<Client>,
}

impl ServerApplication {
    pub fn new(process: StormProcess, filesystem_server: FilesystemServer) -> Self {
        Self {
            process: process,
            filesystem_server: filesystem_server,
            clients: ClientStore::new(),
        }
    }

    pub fn run(&mut self) {
        // main event loop
        loop {
            let event = StormProcess::wait_for_event().unwrap();
            self.filesystem_server.register_event(event);
            while let Some(event) = self.filesystem_server.get_event(&mut self.process) {
                self.process_filesystem_server_event(event);
            }
        }
    }

    fn process_filesystem_server_event(&mut self, event: FilesystemServerChannelEvent) {
        match event {
            FilesystemServerChannelEvent::ClientConnected(service_handle, channel_handle) => {
                self.clients.add_client(service_handle, channel_handle, Client::new());
            }
            FilesystemServerChannelEvent::ClientDisconnected(service_handle, channel_handle) => {
                self.clients.remove_client(service_handle, channel_handle);
            }
            FilesystemServerChannelEvent::ClientRequest(_service_handle, channel_handle, call_id, request) => {
                match request {
                    FilesystemServerRequest::ListObjects(parameters) => {
                        // self.filesystem_server.list_objects_reply(channel_handle, call_id, result);
                    }
                    _ => {
                        // not implemented
                    }
                }
            }
        }
    }
}
