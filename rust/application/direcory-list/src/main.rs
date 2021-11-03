extern crate chaos;

use chaos::{ process::Process, service::Service };

#[derive(Clone, Copy)]
struct ChannelCall {
    x: i32,
    y: i32
}

// #[derive(Clone, Copy)]
struct ChannelResponse {
    result: i32,
    diff: i32
}

fn main() {
    // to be nice, lets set a name for our application
    Process::set_info("Application.DirectoryList");

    // attempt to connect to the vfs service
    match Service::connect("vfs", None, None, None, 4096) {
        Ok(channel_wrap) => {
            // we are connected and got a channel, lets call something on the channel asynchronously
            Process::emit_information("Connected to VFS service");

            let mut channel = channel_wrap.lock().unwrap();
            channel.set(ChannelCall { x: 12, y: 66 });
            channel.call_sync(1, 1, 1000);
            let result = channel.get::<ChannelResponse>();
            Process::emit_information(&format!("got result {} and diff {}", result.result, result.diff));
        },
        Err(error) => {
            Process::emit_error(error, "Failed to connect to VFS service");
        }
    }

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}


// fn list(channel_handle: Handle) -> () {
//     directory_list(channel_handle, "/")
//         .then(|item_iterator| {
//             for item in item_iterator {
//                 process::emit_information(format!("{} {}", item.is_directory ? "D" : "F", item.name));
//             }
//             process::end();
//         })
//         .orelse(|error| {
//             process::emit_error(error, "Call failed");
//             process::end();
//         })
//         .call();

//     process::run();
// }
