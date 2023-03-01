extern crate protocol_console;

use std::{
    alloc::{self, Layout},
    mem::{self, ManuallyDrop},
};
use protocol_console::*;

// fn dump_mem(pointer: *const u64, size: usize) {
//     (0..size).for_each(|x| unsafe {
//         let iter = pointer.offset(x as isize);
//         println!("{:p} {:x}", iter, core::ptr::read_volatile(iter));
//     });
// }

#[test]
fn test_write_and_read() {
    unsafe {
        let layout = Layout::from_size_align(4096, 8).expect("Invalid layout");
        let raw: *mut u8 = alloc::alloc_zeroed(layout);

        let message = WriteTextParameters {
            text: "hejsan".to_string()
        };

        let written = message.write_at(raw);
        let read = WriteTextParameters::reconstruct_at_inline(raw);
        let reconstructed = (raw as *const WriteTextParameters).as_ref().unwrap();

        assert_eq!(reconstructed.text, "hejsan");
    }
}

#[test]
fn test_channel_replacing() {
    unsafe {
        let layout = Layout::from_size_align(4096, 8).expect("Invalid layout");
        let raw0: *mut u8 = alloc::alloc_zeroed(layout);
        let raw1: *mut u8 = alloc::alloc_zeroed(layout);

        let server_channel = ConsoleChannel::new(raw0, raw1, true);
        let mut client_channel = ConsoleChannel::new(raw0, raw1, false);

        assert!(server_channel.check_compatible());
        assert!(client_channel.check_compatible());

        client_channel.prepare_message(10, true);
        client_channel.commit_message(100);

        let message = FromChannel::new(server_channel.find_message().unwrap().as_ref().unwrap());
        
        assert_eq!(message.message_id, 10);
        assert_eq!(message.call_id, 1);
    }
}

// fn main() {
//     test_write_and_read();
// }
