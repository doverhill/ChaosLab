extern crate protocol_console;

use protocol_console::*;
use std::{
    alloc::{self, Layout},
    mem::{self, ManuallyDrop},
};

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
            text: "hejsan".to_string(),
        };

        let written = message.write_at(raw);
        let read = WriteTextParameters::reconstruct_at_inline(raw);
        let reconstructed = (raw as *const WriteTextParameters).as_ref().unwrap();

        assert_eq!(reconstructed.text, "hejsan");
    }
}

#[test]
fn test_from_channel_wrapper() {
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

        assert_eq!(server_channel.number_of_messages_available(), 1);
        {
            let raw_message = server_channel.find_message().unwrap();
            let message = ChannelMessageHeader::get_payload_address(raw_message)
                as *const WriteTextParameters;
            let result = FromChannel::new(&server_channel, raw_message, message);

            assert_eq!((*raw_message).message_id, 10);
            assert_eq!((*raw_message).call_id, 1);
        }
        assert_eq!(server_channel.number_of_messages_available(), 0);
    }
}

#[test]
fn test_queue_replacing() {
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

        client_channel.prepare_message(10, true);
        client_channel.commit_message(100);

        assert_eq!(server_channel.number_of_messages_available(), 2);
        let raw_message = server_channel.find_message().unwrap();
        assert_eq!(server_channel.number_of_messages_available(), 1);
        server_channel.unlink_message(raw_message, false);
        assert_eq!(server_channel.number_of_messages_available(), 0);
    }
}

#[test]
fn test_queue_not_replacing() {
    unsafe {
        let layout = Layout::from_size_align(4096, 8).expect("Invalid layout");
        let raw0: *mut u8 = alloc::alloc_zeroed(layout);
        let raw1: *mut u8 = alloc::alloc_zeroed(layout);

        let server_channel = ConsoleChannel::new(raw0, raw1, true);
        let mut client_channel = ConsoleChannel::new(raw0, raw1, false);

        assert!(server_channel.check_compatible());
        assert!(client_channel.check_compatible());

        client_channel.prepare_message(10, false);
        client_channel.commit_message(100);

        client_channel.prepare_message(10, false);
        client_channel.commit_message(100);

        assert_eq!(server_channel.number_of_messages_available(), 2);
        let raw_message = server_channel.find_message().unwrap();
        assert_eq!(server_channel.number_of_messages_available(), 2);
        server_channel.unlink_message(raw_message, false);
        assert_eq!(server_channel.number_of_messages_available(), 1);
    }
}

#[test]
fn test_race() {
    unsafe {
        let layout = Layout::from_size_align(4096, 8).expect("Invalid layout");
        let raw0: *mut u8 = alloc::alloc_zeroed(layout);
        let raw1: *mut u8 = alloc::alloc_zeroed(layout);

        let server_channel = ConsoleChannel::new(raw0, raw1, true);
        let mut client_channel = ConsoleChannel::new(raw0, raw1, false);

        assert!(server_channel.check_compatible());
        assert!(client_channel.check_compatible());

        client_channel.prepare_message(10, false);
        let raw_message1 = server_channel.find_message();
        client_channel.commit_message(100);
        let raw_message2 = server_channel.find_message();

        assert!(raw_message1.is_none());
        assert!(raw_message2.is_some());
        assert_eq!(server_channel.number_of_messages_available(), 1);
        server_channel.unlink_message(raw_message2.unwrap(), false);
        assert_eq!(server_channel.number_of_messages_available(), 0);
    }
}

// fn main() {
//     test_write_and_read();
// }
