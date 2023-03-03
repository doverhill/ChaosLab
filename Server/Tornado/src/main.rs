// #![no_std]
extern crate alloc;
extern crate library_chaos;
extern crate protocol_console;
extern crate protocol_tornado;

mod state;
use state::ServerState;

use library_chaos::StormProcess;
use protocol_console::*;
use protocol_tornado::*;
use uuid::Uuid;
use core::cell::RefCell;
use alloc::rc::Rc;

fn main() {
    // set up process and server state
    let mut process = StormProcess::new("Server.Tornado").unwrap();

    // connect to console service
    let mut console_client = ConsoleClient::connect_first(&mut process.borrow_mut()).unwrap();
    
    console_client.borrow_mut().write_text(&WriteTextParameters { text: "hello console".to_string(), });

    {
        let mut b = console_client.borrow_mut();
        let console_info = b.get_capabilities(&process.borrow()).unwrap();
        println!(
            "tornado: {}x{}",
            console_info.framebuffer_size.width, console_info.framebuffer_size.height
        );
    }

    // create tornado service
    let tornado_server = TornadoServer::create(
        &mut process.borrow_mut(),
        "Chaos",
        "Tornado server",
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
    ).unwrap();

    let mut state = ServerState::new(process.clone(), tornado_server.clone(), console_client.clone());

    // main event loop
    loop {
        let event = StormProcess::wait_for_event().unwrap();
        console_client.borrow().process_event(&process.borrow(), &event, &mut state);
        tornado_server.borrow_mut().process_event(&mut process.borrow_mut(), &event, &mut state);
    }

    process.borrow().end();
}
