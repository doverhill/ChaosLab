extern crate library_chaos;
extern crate protocol_bogus;

use library_chaos::Process;
use protocol_bogus::BogusClient;

fn main() {
    // to be nice, set a name for our application
    Process::set_info("Application.Test").unwrap();

    // create client (protocol handler) and call it
    let client = BogusClient::default().unwrap();
    let result = client.simple_sum(1, 2).unwrap();
    println!("got simple_sum result {}", result);

    let result = client.get_files("//some_path").unwrap();
    println!("got {} files", result.item_count());
    for file in result {
        println!("  got file {}, size={}", file.path, file.size);
    }

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}
