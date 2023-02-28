extern crate library_chaos;
extern crate protocol_console;
extern crate sdl2;

mod state;
use state::ServerState;

use library_chaos::{StormEvent, StormProcess, StormError};
use protocol_console::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::{EventPump, EventSubsystem};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use std::sync::Arc;
use core::cell::RefCell;
use std::sync::mpsc::channel;

struct StormEventWrapper {
    event: StormEvent,
    quit: bool,
}

fn main() {
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
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
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

    let mut process = StormProcess::new("HostServer.Console").unwrap();
    let mut state = ServerState::new();

    // set up service
    let mut console_server = ConsoleServer::create(
        &mut process,
        "Chaos",
        "SDL console host server",
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
    ).unwrap();

    // hack to get events from both sdl and storm:
    // spawn thread doing storm event wait - posting events to sdl event queue
    // on main thread, loop on sdl event queue and handle incoming events from both sources
    let events = sdl_context.event().unwrap();
    events.register_custom_event::<StormEventWrapper>().unwrap();
    let sender = events.event_sender();
    thread::spawn(move || loop {
        let event = StormProcess::wait_for_event().unwrap();
        sender.push_custom_event(StormEventWrapper { event: event, quit: false });
    });

    // main loop
    let mut pump = sdl_context.event_pump().unwrap();
    'main_loop: loop {
        let event = pump.wait_event();

        if let Some(wrapper) = event.as_user_event_type::<StormEventWrapper>() {
            console_server.process_event(&mut process, &wrapper.event, &mut state);
        } 
        else {
            match event {
                Event::MouseMotion { x, y, .. } => {
                    if let Some(channel_handle) = state.get_first_client_handle() {
                        console_server.pointer_moved(*channel_handle, PointerMovedParameters { position: Point { x: x as i64, y: y as i64 } });
                    }
                },
                Event::Quit { .. } => {
                    break 'main_loop;
                },
                _ => {},
            };
        }
    }

    process.end();
}
