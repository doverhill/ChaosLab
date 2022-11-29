// extern crate library_chaos;
extern crate protocol_console;
extern crate sdl2;

// use library_chaos::Process;
use protocol_console::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

// struct ServerHandler {
// }

// impl ConsoleServerImplementation for ServerHandler {
// }

fn main() {
    // to be nice, set a name for our application
    // Process::set_info("HostServer.Console").unwrap();

    // set up video
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Console 1024x768", 1024, 768).build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 128, 255));
    canvas.clear();
    canvas.present();


    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    // create a unique handler for each connection
    // let _ = ConsoleServer::default("Chaos", "Hosted console", || Box::new(ServerHandler { })).unwrap();

    // // run server
    // let error = Process::run();
    // Process::emit_error(&error, "Event loop error").unwrap();

    // // this is needed for now at the end of every program to clean up correctly
    // Process::end();
}

