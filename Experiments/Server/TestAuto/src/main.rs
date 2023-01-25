extern crate library_chaos;
extern crate protocol_bogus_auto;

use library_chaos::Process;
use protocol_bogus_auto::*;

struct ServerImplementation {
    counter: usize
}

impl BogusAutoServerImplementation for ServerImplementation {
    fn simple_sum(&mut self, x: i32, y: i32) -> i32 {
        x + y + 3
    }

    fn get_files(&mut self, _path: &str) -> Vec<FileInfo> {
        vec!(FileInfo::new("test.txt", 199), FileInfo::new("imba.jpg", 74765))
    }

    fn render(&mut self, objects: RenderMixedArgumentsIterator) {
        for component in objects {
            match component {
                RenderArgumentsEnum::Button(button) => {
                    println!("  rendering button {}:{} with icon={} and text={}", button.component_id, button.parent_component_id, button.icon_name, button.text);
                },
                RenderArgumentsEnum::Window(window) => {
                    println!("  rendering window {}:{} with title={}", window.component_id, window.parent_component_id, window.title);
                }
            }
        }
    }

    fn get_next(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    fn both_mixed(&mut self, objects: BothMixedMixedArgumentsIterator) -> Vec<BothMixedResultEnum> {
        vec!()
    }
}

fn main() {
    // to be nice, set a name for our application
    Process::set_info("Server.TestAuto").unwrap();

    // create server (protocol handler) and provide it with a way of calling our implementation
    // create a unique handler for each connection
    let _ = BogusAutoServer::default("Henrik", "Henriks testserver", || Box::new(ServerImplementation { counter: 0 })).unwrap();

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

