use alloc::format;
use library_chaos::*;
use library_graphics::*;
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
    clients: ClientStore<Client>,
}

impl ServerApplication {
    pub fn new(process: StormProcess, tornado_server: TornadoServer, console_client: ConsoleClient) -> Self {
        Self {
            process: process,
            tornado_server: tornado_server,
            console_client: console_client,
            clients: ClientStore::new(),
        }
    }

    pub fn run(&mut self) {
        let console_info = self.console_client.get_capabilities(&self.process).unwrap();
        self.console_client.write_text(&WriteTextParameters {
            text: format!(
                "Tornado running at {}x{}, {}x{} text",
                console_info.framebuffer_size.width, console_info.framebuffer_size.height, console_info.text_size.width, console_info.text_size.height
            ),
        });

        // main event loop
        loop {
            let event = StormProcess::wait_for_event().unwrap();
            self.console_client.register_event(event);
            while let Some(event) = self.console_client.get_event(&mut self.process) {
                self.process_console_client_event(event);
            }
            self.tornado_server.register_event(event);
            while let Some(event) = self.tornado_server.get_event(&mut self.process) {
                self.process_tornado_server_event(event);
            }
        }
    }

    fn process_console_client_event(&mut self, event: ConsoleClientChannelEvent) {
        match event {
            ConsoleClientChannelEvent::ServerDisconnected(_channel_handle) => {}
            ConsoleClientChannelEvent::ServerEvent(_channel_handle, event) => {
                match event {
                    ConsoleClientEvent::PointerMoved(parameters) => {
                        let mut painter = ImagePainter::new(
                            100,
                            100,
                            Color {
                                alpha: 255,
                                red: 128,
                                green: 128,
                                blue: 128,
                            },
                        );
                        painter.draw_filled_box_sized(
                            Point { x: 20, y: 20 },
                            Size { width: 50, height: 20 },
                            Color {
                                alpha: 255,
                                red: 200,
                                green: 0,
                                blue: 0,
                            },
                        );
                        painter.draw_frame_sized(
                            Point { x: 40, y: 30 },
                            Size { width: 50, height: 20 },
                            Color {
                                alpha: 255,
                                red: 0,
                                green: 200,
                                blue: 0,
                            },
                        );
                
                        self.console_client.draw_image_patch(&DrawImagePatchParameters {
                            image_patch: ImagePatch {
                                image: painter.to_image(),
                                position: parameters.position,
                            },
                        });

                        // self.console_client.draw_pixel_debug(&DrawPixelDebugParameters {
                        //     position: parameters.position,
                        //     color: Color {
                        //         alpha: 255,
                        //         red: 255,
                        //         green: 0,
                        //         blue: 0,
                        //     },
                        // });
                    }
                    ConsoleClientEvent::PointerPressed(parameters) => {
                        self.console_client.draw_pixel_debug(&DrawPixelDebugParameters {
                            position: parameters.position,
                            color: Color {
                                alpha: 255,
                                red: 0,
                                green: 255,
                                blue: 0,
                            },
                        });
                        self.console_client.write_text(&WriteTextParameters {
                            text: format!("tornado: pointer clicked at {}, {}", parameters.position.x, parameters.position.y),
                        });
                    }
                    _ => {
                        // not implemented
                    }
                }
            }
        }
    }

    fn process_tornado_server_event(&mut self, event: TornadoServerChannelEvent) {
        match event {
            TornadoServerChannelEvent::ClientConnected(service_handle, channel_handle) => {
                self.clients.add_client(service_handle, channel_handle, Client::new());
            }
            TornadoServerChannelEvent::ClientDisconnected(service_handle, channel_handle) => {
                self.clients.remove_client(service_handle, channel_handle);
            }
            TornadoServerChannelEvent::ClientRequest(_service_handle, _channel_handle, _call_id, request) => {
                match request {
                    TornadoServerRequest::SetRenderTree(_parameters) => {} // _ => {
                                                                           //     // not implemented
                                                                           // }
                }
            }
        }
    }
}
