use library_chaos::{ChannelHandle, ServiceHandle, StormProcess, StormEvent};
use protocol_console::*;
use sdl2::Sdl;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, EventSubsystem};
use std::collections::HashMap;
use std::thread;

struct StormEventWrapper {
    event: StormEvent,
    quit: bool,
}

struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct ServerApplication {
    process: StormProcess,
    console_server: ConsoleServer,
    clients: HashMap<ChannelHandle, Client>,
    sdl: Sdl,
    canvas: Canvas<Window>,
}

impl ServerApplication {
    pub fn new(process: StormProcess, console_server: ConsoleServer) -> Self {
        let scale_factor: usize = 2;
        let width = 800;
        let height = 600;
        let glyph_width = 8;
        let glyph_height = 16;
        let text_width = width / glyph_width;
        let text_height = height / glyph_height;

        let window_width = (scale_factor * width) as u32;
        let window_height = (scale_factor * height) as u32;
        let window_title = format!(
            "Console framebuffer: {}x{} text: {}x{} - 1 / 1",
            width, height, text_width, text_height
        );

        // set up video
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window(window_title.as_str(), window_width, window_height)
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();

        // canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 128, 255));
        // canvas.clear();

        // canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        // let r = Rect::new(
        //     0i32,
        //     0i32,
        //     (glyph_width * scale_factor) as u32,
        //     (glyph_height * scale_factor) as u32,
        // );
        // canvas.draw_rect(r);

        // canvas.with_texture_canvas(texture, f)

        canvas.present();

        Self {
            process: process,
            console_server: console_server,
            clients: HashMap::new(),
            sdl: sdl,
            canvas: canvas,
        }
    }

    pub fn run(&mut self) {
        // hack to get events from both sdl and storm:
        // spawn thread doing storm event wait - posting events to sdl event queue
        // on main thread, loop on sdl event queue and handle incoming events from both sources
        let events = self.sdl.event().unwrap();
        events.register_custom_event::<StormEventWrapper>().unwrap();
        let sender = events.event_sender();
        thread::spawn(move || loop {
            let event = StormProcess::wait_for_event().unwrap();
            sender.push_custom_event(StormEventWrapper {
                event: event,
                quit: false,
            });
        });

        // main loop
        let mut pump = self.sdl.event_pump().unwrap();
        'main_loop: loop {
            let event = pump.wait_event();

            if let Some(wrapper) = event.as_user_event_type::<StormEventWrapper>() {
                self.console_server.register_event(wrapper.event);
                while let Some(console_server_event) = self.console_server.get_event(&mut self.process) {
                    self.process_console_server_event(console_server_event);
                }
            } else {
                match event {
                    Event::MouseMotion { x, y, .. } => {
                        if let Some(channel_handle) = self.get_first_client_handle() {
                            self.console_server.pointer_moved(
                                *channel_handle,
                                &PointerMovedParameters {
                                    position: Point {
                                        x: x as i64,
                                        y: y as i64,
                                    },
                                },
                            );
                        }
                    }
                    Event::Quit { .. } => {
                        break 'main_loop;
                    }
                    _ => {}
                };
            }
        }
    }

    fn process_console_server_event(&mut self, event: ConsoleServerChannelEvent) {
        match event {
            ConsoleServerChannelEvent::ClientConnected(service_handle, channel_handle) => {
                self.clients.insert(channel_handle, Client::new());
            }
            ConsoleServerChannelEvent::ClientDisconnected(service_handle, channel_handle) => {
                self.clients.remove(&channel_handle);
            }
            ConsoleServerChannelEvent::ClientRequest(service_handle, channel_handle, call_id, request) => {
                match request {
                    ConsoleServerRequest::WriteText(parameters) => {
                        println!("console::WriteText: {}", parameters.text);
                    },
                    ConsoleServerRequest::GetCapabilities => {
                        println!("console::GetCapabilities");
                        self.console_server.get_capabilities_reply(channel_handle, call_id, &GetCapabilitiesReturns {
                            is_framebuffer: true,
                            framebuffer_size: Size {
                                width: 1000,
                                height: 800,
                            },
                            text_size: Size {
                                width: 80,
                                height: 50
                            }
                        });
                    },
                    _ => {
                        // not implemented
                    }
                }
            }
        }
    }

    pub fn get_first_client_handle(&self) -> Option<&ChannelHandle> {
        self.clients.keys().next()
    }
}

// impl ConsoleServerObserver for ServerState {
//     fn handle_console_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
//         println!("handle_console_client_connected");
//         self.add_client(channel_handle);
//     }

//     fn handle_console_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
//         println!("handle_console_client_disconnected");
//         self.remove_client(channel_handle);
//     }

//     fn handle_console_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, call_id: u64, request: ConsoleServerRequest) {
//         println!("handle_console_request");

//         match request {
//             ConsoleServerRequest::WriteText(parameters) => {
//                 println!("console::write_text: {}", parameters.text);
//             },
//             ConsoleServerRequest::GetCapabilities => {
//                 let capabilities = GetCapabilitiesReturns {
//                     is_framebuffer: true,
//                     framebuffer_size: Size {
//                         width: 1024,
//                         height: 768
//                     },
//                     text_size: Size {
//                         width: 80,
//                         height: 50
//                     }
//                 };
//                 self.console_server.borrow_mut().get_capabilities_reply(channel_handle, call_id, &capabilities);
//             }
//             _ => {}
//         }
//     }
// }
