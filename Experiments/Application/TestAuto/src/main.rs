extern crate library_chaos;
extern crate protocol_bogus_auto;

use library_chaos::Process;
use protocol_bogus_auto::*;

struct Client {
}

impl BogusAutoClientImplementation for Client {
    fn notify(&mut self, message: &str) {
        println!("notified with {}", message);
    }
}

fn main() {
    // to be nice, set a name for our application
    Process::set_info("Application.TestAuto").unwrap();

    // create client (protocol handler) and call it
    let implementation = Client {};
    let client_reference = BogusAutoClient::default(Box::new(implementation)).unwrap();
    let client = client_reference.lock().unwrap();

    let result = client.simple_sum(1, 2).unwrap();
    println!("got simple_sum result {}", result);

    let result = client.get_files("//some_path").unwrap();
    // println!("got {} files", result.item_count());
    for file in result {
        println!("  got file {}, size={}", file.path, file.size);
    }

    client.render(vec!(
        RenderArgumentsEnum::Window(Window::new( 1, 0, "This is the window title" )),
        RenderArgumentsEnum::Button(Button::new( 2, 1, "none", "Click me" ))
    ));

    let result = client.get_next().unwrap();
    println!("got next result {:?}", result);
    let result = client.get_next().unwrap();
    println!("got next result {:?}", result);

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}
