use crate::helpers;
use core::cell::RefCell;
use library_chaos::ServiceHandle;
use library_chaos::{ChannelHandle, ClientStore, StormEvent, StormProcess};
use protocol_console::*;
use crate::dirty_patches::DirtyPatches;
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
    pub saved_text_position: Point,
}

impl<'a> Client<'a> {
    pub fn new(channel_handle: ChannelHandle, name: String, surface: Surface<'a>) -> Self {
        Self {
            channel_handle: channel_handle,
            console_number: -1,
            _name: name,
            surface: surface,
            text_position: Point { x: 0, y: 0 },
            saved_text_position: Point { x: 0, y: 0 },
        }
    }
}

pub struct ServerApplication<'a> {
    process: StormProcess,
    console_server: ConsoleServer,
    clients: ClientStore<RefCell<Client<'a>>>, // HashMap<ChannelHandle, RefCell<Client<'a>>>,
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
        let window = video_subsystem.window("Chaos console", 1600, 1000).fullscreen_desktop().build().unwrap();

        let mut canvas = window.into_canvas().accelerated().build().unwrap();

        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas.clear();
        canvas.present();

        Self {
            process: process,
            console_server: console_server,
            clients: ClientStore::new(),
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
        let glyph_size = Size {
            width: glyph_width as u64,
            height: glyph_height as u64,
        };

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
            sender.push_custom_event(StormEventWrapper { event: event, _quit: false }).unwrap();
        });

        // main loop
        let mut pump = self.sdl.event_pump().unwrap();
        'main_loop: loop {
            let event = pump.wait_event();

            if let Some(wrapper) = event.as_user_event_type::<StormEventWrapper>() {
                self.console_server.register_event(wrapper.event);

                let mut dirty_patches = DirtyPatches::new();
                while let Some(console_server_event) = self.console_server.get_event(&mut self.process) {
                    if let Some(patch) = self.process_console_server_event(console_server_event, glyph_size, framebuffer_size, text_size, &font) {
                        dirty_patches.add_dirty(patch);
                    }
                }

                if dirty_patches.patches.len() > 0 {
                    self.refresh_active(self.console_server.service_handle, Some(dirty_patches));
                }
            } else {
                match event {
                    Event::MouseMotion { x, y, .. } => {
                        if let Some(channel_handle) = self.active_client_channel_handle {
                            self.console_server.pointer_moved(
                                channel_handle,
                                &PointerMovedParameters {
                                    position: Point { x: x as i64, y: y as i64 },
                                },
                            );
                        }
                    }
                    Event::MouseButtonDown { mouse_btn, clicks, x, y, .. } => {
                        if let Some(channel_handle) = self.active_client_channel_handle {
                            self.console_server.pointer_pressed(
                                channel_handle,
                                &PointerPressedParameters {
                                    position: Point { x: x as i64, y: y as i64 },
                                    buttons: vec![helpers::convert_mount_button_sdl_to_console(mouse_btn)],
                                },
                            );
                        }
                    }
                    Event::MouseButtonUp { mouse_btn, clicks, x, y, .. } => {
                        if let Some(channel_handle) = self.active_client_channel_handle {
                            self.console_server.pointer_released(
                                channel_handle,
                                &PointerReleasedParameters {
                                    position: Point { x: x as i64, y: y as i64 },
                                    buttons: vec![helpers::convert_mount_button_sdl_to_console(mouse_btn)],
                                },
                            );
                        }
                    }
                    Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'main_loop;
                    }
                    Event::TextInput { text, .. } => {
                        if let Some(channel_handle) = self.active_client_channel_handle {
                            let character = text.chars().nth(0).unwrap() as u64;
                            self.console_server.character_input(channel_handle, &CharacterInputParameters { character: character });
                        }
                    }
                    Event::KeyDown { keycode: Some(keycode), keymod, .. } => {
                        // reserved console command
                        if keymod.contains(Mod::LSHIFTMOD | Mod::LCTRLMOD | Mod::LALTMOD) {
                            match keycode {
                                Keycode::Left => {
                                    self.focus_previous_client(self.console_server.service_handle);
                                }
                                Keycode::Right => {
                                    self.focus_next_client(self.console_server.service_handle);
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

    fn add_client(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, framebuffer_size: Size) {
        let mut surface = Surface::new(framebuffer_size.width as u32, framebuffer_size.height as u32, PixelFormatEnum::ARGB32).unwrap();
        surface
            .fill_rect(
                Rect::new(0, 0, framebuffer_size.width as u32, framebuffer_size.height as u32),
                helpers::convert_color_console_to_sdl(Color {
                    alpha: 255,
                    red: 0,
                    green: 0,
                    blue: 0,
                }),
            )
            .unwrap();
        self.clients
            .add_client(service_handle, channel_handle, RefCell::new(Client::new(channel_handle, "unnamed".to_string(), surface)));
        self.reindex_clients();
        self.active_client_channel_handle = Some(channel_handle);
        self.active_console_number = self.get_console_number(channel_handle);
    }

    fn remove_client(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle) {
        self.clients.remove_client(service_handle, channel_handle);
        self.reindex_clients();

        if let Some(active_channel_handle) = self.active_client_channel_handle {
            if active_channel_handle == channel_handle {
                if self.clients.client_count() > 0 {
                    let (_, active_channel_handle, _) = self.clients.first().unwrap();
                    self.active_client_channel_handle = Some(*active_channel_handle);
                    self.active_console_number = self.get_console_number(*active_channel_handle);
                    self.refresh_active(service_handle, None);
                    // if let Some(active_client) = self.get_active_client() {
                    //     self.refresh(active_client.borrow_mut());
                    // }
                } else {
                    self.active_client_channel_handle = None;
                    self.active_console_number = -1;
                }
            }
        }
    }

    fn get_console_number(&self, for_channel_handle: ChannelHandle) -> isize {
        if let Some((_, _, client)) = self.clients.first_matching(|_, channel_handle, _| for_channel_handle == *channel_handle) {
            client.borrow().console_number
        } else {
            0
        }
    }

    fn get_channel_handle(&self, for_console_number: isize) -> Option<ChannelHandle> {
        if let Some((_, channel_handle, _)) = self.clients.first_matching(|_, _, client| for_console_number == client.borrow().console_number) {
            Some(*channel_handle)
        } else {
            None
        }
    }

    // fn get_active_client(&self) -> Option<&RefCell<Client<'a>>> {
    //     if let Some((_, _, client)) = self.clients.first_matching(|_, _, client| self.active_console_number == client.borrow().console_number) {
    //         Some(client)
    //     } else {
    //         None
    //     }
    // }

    fn reindex_clients(&mut self) {
        let mut index: isize = 0;
        let assign = |client: &mut RefCell<Client>| {
            client.borrow_mut().console_number = index;
            index += 1;
        };
        self.clients.for_each_mut(assign);
    }

    fn focus_previous_client(&mut self, service_handle: ServiceHandle) {
        if self.clients.client_count() > 1 {
            self.active_console_number -= 1;
            if self.active_console_number < 0 {
                self.active_console_number = self.clients.client_count() as isize - 1;
            }
            self.active_client_channel_handle = self.get_channel_handle(self.active_console_number);
            self.refresh_active(service_handle, None);
            // if let Some(active_client) = self.get_active_client() {
            //     self.refresh(active_client.borrow_mut());
            // }
        }
    }

    fn focus_next_client(&mut self, service_handle: ServiceHandle) {
        if self.clients.client_count() > 1 {
            self.active_console_number += 1;
            if self.active_console_number == self.clients.client_count() as isize {
                self.active_console_number = 0;
            }
            self.active_client_channel_handle = self.get_channel_handle(self.active_console_number);
            self.refresh_active(service_handle, None);
            // if let Some(active_client) = self.get_active_client() {
            //     self.refresh(active_client.borrow_mut());
            // }
        }
    }

    fn process_console_server_event(&mut self, event: ConsoleServerChannelEvent, glyph_size: Size, framebuffer_size: Size, text_size: Size, font: &Font) -> Option<Rect> {
        let mut dirty_patch: Option<Rect> = None;

        match event {
            ConsoleServerChannelEvent::ClientConnected(service_handle, channel_handle) => {
                self.add_client(service_handle, channel_handle, framebuffer_size);
            }
            ConsoleServerChannelEvent::ClientDisconnected(service_handle, channel_handle) => {
                self.remove_client(service_handle, channel_handle);
            }
            ConsoleServerChannelEvent::ClientRequest(service_handle, channel_handle, call_id, request) => {
                if let Some(client) = self.clients.get_client(service_handle, channel_handle) {
                    match request {
                        ConsoleServerRequest::WriteText(parameters) => {
                            let patch = helpers::draw_text(client.borrow_mut(), glyph_size, text_size, font, &parameters.text);
                            if let Some(active_channel_handle) = self.active_client_channel_handle {
                                if active_channel_handle == channel_handle {
                                    dirty_patch = Some(patch);
                                }
                            } 
                        }
                        ConsoleServerRequest::SaveTextCursorPosition => {
                            let mut borrowed = client.borrow_mut();
                            borrowed.saved_text_position = borrowed.text_position;
                        }
                        ConsoleServerRequest::LoadTextCursorPosition => {
                            let mut borrowed = client.borrow_mut();
                            borrowed.text_position = borrowed.saved_text_position;
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
                            let patch = helpers::draw_pixel(client.borrow_mut(), parameters.color, parameters.position);
                            if let Some(active_channel_handle) = self.active_client_channel_handle {
                                if active_channel_handle == channel_handle {
                                    dirty_patch = Some(patch);
                                }
                            } 
                        }
                        _ => {
                            // not implemented
                        }
                    }
                }
            }
        }

        dirty_patch
    }

    fn refresh_active(&self, service_handle: ServiceHandle, dirty: Option<DirtyPatches>) {
        if let Some(active_channel_handle) = self.active_client_channel_handle {
            if let Some(client) = self.clients.get_client(service_handle, active_channel_handle) {
                let mut canvas = self.canvas.borrow_mut();
                let texture_creator = canvas.texture_creator();
                let texture = texture_creator.create_texture_from_surface(&client.borrow().surface).unwrap();
                if let Some(dirty_patches) = dirty {
                    for patch in dirty_patches.patches {
                        canvas.copy(&texture, patch, patch).unwrap();
                    }
                }
                else {
                    canvas.copy(&texture, None, None).unwrap();
                }
                canvas.present();
            }
        }
    }
}
