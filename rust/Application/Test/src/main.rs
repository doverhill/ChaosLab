extern crate library_chaos;
extern crate protocol_bogus;

use library_chaos::Process;
use protocol_bogus::BogusClient;

fn main() {
    // to be nice, set a name for our application
    Process::set_info("Application.Test").unwrap();

    // create client (protocol handler) and call it
    let client = BogusClient::default().unwrap();
    let result = client.simple_sum(19, 33).unwrap();
    println!("got result {}", result);

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}
