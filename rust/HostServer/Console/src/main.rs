extern crate library_chaos;
extern crate protocol_console;
extern crate sdl2;

use library_chaos::Process;
use protocol_console::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

struct ServerHandler {
}

impl ConsoleServerImplementation for ServerHandler {
}

fn main() {
    // to be nice, set a name for our application
    Process::set_info("HostServer.Console").unwrap();

    // set up video
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Console 1024x768", 1024, 768).build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 128, 255));
    canvas.clear();
    canvas.present();

    // create a unique handler for each connection
    let _ = ConsoleServer::default("Chaos", "Hosted console", || Box::new(ServerHandler { })).unwrap();

    // run server
    let error = Process::run();
    Process::emit_error(&error, "Event loop error").unwrap();

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}

