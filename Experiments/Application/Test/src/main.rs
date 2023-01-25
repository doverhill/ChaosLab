extern crate library_chaos;
extern crate protocol_bogus;

use library_chaos::Process;
use protocol_bogus::{ Window, Button, BogusClient, RenderTypeArguments };

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

    client.render_start();
    client.render_add(RenderTypeArguments::Window(Window::new( 1, 0, "This is the window title" )));
    client.render_add(RenderTypeArguments::Button(Button::new( 2, 1, "none", "Click me" )));
    client.render_done();

    let result = client.get_next().unwrap();
    println!("got next result {:?}", result);
    let result = client.get_next().unwrap();
    println!("got next result {:?}", result);

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}
