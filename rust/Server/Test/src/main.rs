extern crate library_chaos;
extern crate protocol_bogus;

use library_chaos::Process;
use protocol_bogus::{ BogusServer, BogusServerImplementation, FileInfo, RenderTypeArguments, RenderHandleIterator };

struct ServerHandler {
    counter: usize
}

impl BogusServerImplementation for ServerHandler {
    fn simple_sum(&mut self, x: i32, y: i32) -> i32 {
        x + y + 3
    }

    fn get_files(&mut self, path: &str) -> Vec<FileInfo> {
        vec!(FileInfo::new("test.txt", 199), FileInfo::new("imba.jpg", 74765))
    }

    fn render(&mut self, components: RenderHandleIterator) {
        println!("rendering {} components", components.item_count());
        for component in components {
            match component {
                RenderTypeArguments::Button(button) => {
                    println!("  rendering button {}:{} with icon={} and text={}", button.base.component_id, button.base.parent_component_id, button.icon_name, button.text);
                },
                RenderTypeArguments::Window(window) => {
                    println!("  rendering window {}:{} with title={}", window.base.component_id, window.base.parent_component_id, window.title);
                }
            }
        }
    }

    fn get_next(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
}

fn main() {
    // to be nice, set a name for our application
    Process::set_info("Server.Test").unwrap();

    // create server (protocol handler) and provide it with a way of calling our implementation
    // create a unique handler for each connection
    let _ = BogusServer::default(|| Box::new(ServerHandler { counter: 0 })).unwrap();

    // create server (protocol handler) and provide it with a way of calling our implementation
    // share the same handler for each connection
    // let handler = ServerHandler { counter: 0 };
    // let server_reference = BogusServer::default(|| handler).unwrap();
    
    // run server
    let error = Process::run();
    Process::emit_error(&error, "Event loop error").unwrap();

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}

