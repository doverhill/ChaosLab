extern crate library_chaos;
extern crate protocol_filesystem;

mod application;
use application::ServerApplication;

use library_chaos::*;
use protocol_filesystem::*;
use uuid::Uuid;

fn main() {
    let mut process = StormProcess::new("HostServer.Filesystem").unwrap();

    // set up service
    let filesystem_server = FilesystemServer::create(
        &mut process,
        "Chaos",
        "Filesystem host server",
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
    ).unwrap();

    let mut server_application = ServerApplication::new(process, filesystem_server);
    server_application.run();

    // process.borrow().end();
}
