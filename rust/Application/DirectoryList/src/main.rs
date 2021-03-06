extern crate chaos;
use chaos::{ process::Process, service::Service };

#[allow(dead_code)]
struct ChannelCall {
    msg: [u8; 100],
    x: i32
}

#[allow(dead_code)]
impl ChannelCall {
    pub fn new(msg: &str, x: i32) -> ChannelCall {
        let tmp = ChannelCall {
            msg: [0u8; 100],
            x: x
        };
        unsafe {
            core::ptr::copy(msg.as_ptr(), core::ptr::addr_of!(tmp.msg) as *mut u8, core::cmp::min(99, msg.len()));
        }
        tmp
    }

    pub fn get_msg(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.msg) }
    }
}

#[allow(dead_code)]
struct ChannelResponse {
    length: i32,
    new_msg: [u8; 100]
}

#[allow(dead_code)]
impl ChannelResponse {
    pub fn new(length: i32, new_msg: &str) -> ChannelResponse {
        let tmp = ChannelResponse {
            length: length,
            new_msg: [0u8; 100]
        };
        unsafe {
            core::ptr::copy(new_msg.as_ptr(), core::ptr::addr_of!(tmp.new_msg) as *mut u8, core::cmp::min(99, new_msg.len()));
        }
        tmp
    }

    pub fn get_new_msg(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.new_msg) }
    }
}

fn main() {
    // to be nice, set a name for our application
    Process::set_info("Application.DirectoryList").unwrap();

    // attempt to connect to the vfs service
    match Service::connect("vfs", None, None, None, 4096) {
        Ok(channel_wrap) => {
            let channel = channel_wrap.lock().unwrap();
            channel.set(ChannelCall::new("test string", 77));
            channel.call_sync(1, 1000).unwrap();
            let result = channel.get::<ChannelResponse>();
            Process::emit_information(&format!("got result '{}' with len {}", result.get_new_msg(), result.length)).unwrap();
        },
        Err(error) => {
            Process::emit_error(error, "Failed to connect to VFS service").unwrap();
        }
    }

    // this is needed for now at the end of every program to clean up correctly
    Process::end();
}
