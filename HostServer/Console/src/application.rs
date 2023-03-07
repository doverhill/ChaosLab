use crate::helpers;
use core::cell::RefCell;
use library_chaos::{ChannelHandle, StormEvent, StormProcess};
use protocol_console::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use sdl2::video::Window;
use sdl2::Sdl;
use std::cell::RefMut;
use std::collections::HashMap;
use std::path::Path;
use std::thread;

struct StormEventWrapper {
    event: StormEvent,
    _quit: bool,
}

pub struct Client<'a> {
    channel_handle: ChannelHandle,
    console_number: isize,
    _name: String,
    pub surface: Surface<'a>,
    pub text_position: Point,
}

impl<'a> Client<'a> {
    pub fn new(channel_handle: ChannelHandle, name: String, surface: Surface<'a>) -> Self {
        Self {
            channel_handle: channel_handle,
            console_number: -1,
            _name: name,
            surface: surface,
            text_position: Point { x: 0, y: 0 },
        }
    }
}

pub struct ServerApplication<'a> {
    process: StormProcess,
    console_server: ConsoleServer,
    clients: HashMap<ChannelHandle, RefCell<Client<'a>>>,
    sdl: Sdl,
    canvas: RefCell<Canvas<Window>>,
    active_client_channel_handle: Option<ChannelHandle>,
    active_console_number: isize,
}

impl<'a> ServerApplication<'a> {
    pub fn new(process: StormProcess, console_server: ConsoleServer) -> Self {
        // set up video
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window("Chaos console", 1600, 900)
            // .fullscreen_desktop()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().accelerated().build().unwrap();

        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas.clear();
        canvas.present();

        Self {
            process: process,
            console_server: console_server,
            clients: HashMap::new(),
            sdl: sdl,
            canvas: RefCell::new(canvas),
            active_client_channel_handle: None,
            active_console_number: -1,
        }
    }

    pub fn run(&mut self) {
        let font_context = sdl2::ttf::init().unwrap();
        let font_path = Path::new("ShareTechMono-Regular.ttf");
        let font = font_context.load_font(font_path, 13).unwrap();
        let (glyph_width, glyph_height) = font.size_of_char('M').unwrap();

        let (width, height) = self.canvas.borrow().output_size().unwrap();
        let text_width = width / glyph_width;
        let text_height = height / glyph_height;

        let framebuffer_size = Size {
            width: width as u64,
            height: height as u64,
        };
        let text_size = Size {
            width: text_width as u64,
            height: text_height as u64,
        };

        // hack to get events from both sdl and storm:
        // spawn thread doing storm event wait - posting events to sdl event queue
        // on main thread, loop on sdl event queue and handle incoming events from both sources
        let events = self.sdl.event().unwrap();
        events.register_custom_event::<StormEventWrapper>().unwrap();
        let sender = events.event_sender();
        thread::spawn(move || loop {
            let event = StormProcess::wait_for_event().unwrap();
            sender
                .push_custom_event(StormEventWrapper {
                    event: event,
                    _quit: false,
                })
                .unwrap();
        });

        // main loop
        let mut pump = self.sdl.event_pump().unwrap();
        'main_loop: loop {
            let event = pump.wait_event();

            if let Some(wrapper) = event.as_user_event_type::<StormEventWrapper>() {
                self.console_server.register_event(wrapper.event);
                while let Some(console_server_event) =
                    self.console_server.get_event(&mut self.process)
                {
                    self.process_console_server_event(
                        console_server_event,
                        framebuffer_size,
                        text_size,
                        &font,
                    );
                }
            } else {
                match event {
                    Event::MouseMotion { x, y, .. } => {
                        if let Some(channel_handle) = self.active_client_channel_handle {
                            self.console_server.pointer_moved(
                                channel_handle,
                                &PointerMovedParameters {
                                    position: Point {
                                        x: x as i64,
                                        y: y as i64,
                                    },
                                },
                            );
                        }
                    }
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        break 'main_loop;
                    }
                    Event::KeyDown {
                        keycode: Some(keycode),
                        keymod,
                        ..
                    } => {
                        // reserved console command
                        if keymod.contains(Mod::LSHIFTMOD | Mod::LCTRLMOD | Mod::LALTMOD) {
                            match keycode {
                                Keycode::Left => {
                                    self.focus_previous_client();
                                }
                                Keycode::Right => {
                                    self.focus_next_client();
                                }
                                _ => {}
                            }
                        } else {
                            // pass to active client
                            if let Some(channel_handle) = self.active_client_channel_handle {
                                self.console_server.key_pressed(
                                    channel_handle,
                                    &KeyPressedParameters {
                                        key_code: helpers::convert_key_code_sdl_to_console(keycode),
                                    },
                                );
                            }
                        }
                    }
                    _ => {}
                };
            }
        }
    }

    fn add_client(&mut self, channel_handle: ChannelHandle, framebuffer_size: Size) {
        let mut surface = Surface::new(
            framebuffer_size.width as u32,
            framebuffer_size.height as u32,
            PixelFormatEnum::ARGB32,
        )
        .unwrap();
        surface.fill_rect(
            Rect::new(0, 0, framebuffer_size.width as u32, framebuffer_size.height as u32),
            helpers::convert_color_console_to_sdl(Color {
                alpha: 255,
                red: 0,
                green: 0,
                blue: 0,
            }),
        ).unwrap();
        self.clients.insert(
            channel_handle,
            RefCell::new(Client::new(channel_handle, "unnamed".to_string(), surface)),
        );
        self.reindex_clients();
        self.active_client_channel_handle = Some(channel_handle);
        self.active_console_number = self.get_console_number(channel_handle);
    }

    fn remove_client(&mut self, channel_handle: ChannelHandle) {
        self.clients.remove(&channel_handle);
        self.reindex_clients();

        if let Some(active_channel_handle) = self.active_client_channel_handle {
            if active_channel_handle == channel_handle {
                if self.clients.len() > 0 {
                    let active_channel_handle = self
                        .clients
                        .iter()
                        .nth(0)
                        .unwrap()
                        .1
                        .borrow()
                        .channel_handle;
                    self.active_client_channel_handle = Some(active_channel_handle);
                    self.active_console_number = self.get_console_number(active_channel_handle);
                } else {
                    self.active_client_channel_handle = None;
                    self.active_console_number = -1;
                }
            }
        }
    }

    fn get_console_number(&self, channel_handle: ChannelHandle) -> isize {
        for (handle, client) in self.clients.iter() {
            if *handle == channel_handle {
                return client.borrow().console_number;
            }
        }
        0
    }

    fn get_channel_handle(&self, console_number: isize) -> Option<ChannelHandle> {
        for (handle, client) in self.clients.iter() {
            if client.borrow().console_number == console_number {
                return Some(*handle);
            }
        }
        None
    }

    fn get_active_client(&self) -> Option<&RefCell<Client<'a>>> {
        for (_, client) in self.clients.iter() {
            if client.borrow().console_number == self.active_console_number {
                return Some(client);
            }
        }
        None
    }

    fn reindex_clients(&mut self) {
        let mut index: isize = 0;
        for (_, client) in self.clients.iter_mut() {
            client.borrow_mut().console_number = index;
            index += 1;
        }
    }

    fn focus_previous_client(&mut self) {
        if self.clients.len() > 1 {
            self.active_console_number -= 1;
            if self.active_console_number < 0 {
                self.active_console_number = self.clients.len() as isize - 1;
            }
            self.active_client_channel_handle = self.get_channel_handle(self.active_console_number);
            if let Some(active_client) = self.get_active_client() {
                self.refresh(active_client.borrow_mut());
            }
        }
    }

    fn focus_next_client(&mut self) {
        if self.clients.len() > 1 {
            self.active_console_number += 1;
            if self.active_console_number == self.clients.len() as isize {
                self.active_console_number = 0;
            }
            self.active_client_channel_handle = self.get_channel_handle(self.active_console_number);
            if let Some(active_client) = self.get_active_client() {
                self.refresh(active_client.borrow_mut());
            }
        }
    }

    fn process_console_server_event(
        &mut self,
        event: ConsoleServerChannelEvent,
        framebuffer_size: Size,
        text_size: Size,
        font: &Font,
    ) {
        match event {
            ConsoleServerChannelEvent::ClientConnected(_service_handle, channel_handle) => {
                self.add_client(channel_handle, framebuffer_size);
            }
            ConsoleServerChannelEvent::ClientDisconnected(_service_handle, channel_handle) => {
                self.remove_client(channel_handle);
            }
            ConsoleServerChannelEvent::ClientRequest(
                _service_handle,
                channel_handle,
                call_id,
                request,
            ) => {
                if let Some(client) = self.clients.get(&channel_handle) {
                    match request {
                        ConsoleServerRequest::WriteText(parameters) => {
                            println!("console: {}", parameters.text);
                            helpers::draw_text(client.borrow_mut(), font, &parameters.text);
                            self.refresh(client.borrow_mut());
                        }
                        ConsoleServerRequest::GetCapabilities => {
                            self.console_server.get_capabilities_reply(
                                channel_handle,
                                call_id,
                                &GetCapabilitiesReturns {
                                    is_framebuffer: true,
                                    framebuffer_size: framebuffer_size,
                                    text_size: text_size,
                                },
                            );
                        }
                        ConsoleServerRequest::DrawPixelDebug(parameters) => {
                            helpers::draw_pixel(
                                client.borrow_mut(),
                                parameters.color,
                                parameters.position,
                            );
                            self.refresh(client.borrow_mut());
                        }
                        _ => {
                            // not implemented
                        }
                    }
                }
            }
        }
    }

    fn refresh(&self, client: RefMut<Client>) {
        if let Some(active_channel_handle) = self.active_client_channel_handle {
            if client.channel_handle == active_channel_handle {
                let texture_creator = self.canvas.borrow().texture_creator();
                let texture = texture_creator
                    .create_texture_from_surface(&client.surface)
                    .unwrap();
                self.canvas.borrow_mut().copy(&texture, None, None).unwrap();
                self.canvas.borrow_mut().present();
            }
        }
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
