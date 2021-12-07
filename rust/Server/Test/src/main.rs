extern crate library_chaos;
extern crate protocol_bogus;

use library_chaos::Process;
use protocol_bogus::BogusServer;

fn main() {
    // to be nice, set a name for our application
    Process::set_info("Server.Test").unwrap();

    // create server (protocol handler) and hook call handlers
    let mut server = BogusServer::default().unwrap();
    server.on_simple_sum(|x, y| { x + y + 7 });

    // run server
    let error = Process::run();
    Process::emit_error(&error, "Event loop error").unwrap();

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}

