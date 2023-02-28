extern crate protocol_console;

use std::{
    alloc::{self, Layout},
    mem::{self, ManuallyDrop},
};
use protocol_console::*;

fn dump_mem(pointer: *const u64, size: usize) {
    (0..size).for_each(|x| unsafe {
        let iter = pointer.offset(x as isize);
        println!("{:p} {:x}", iter, core::ptr::read_volatile(iter));
    });
}

// #[test]
fn test_write_and_read() {
    unsafe {
        let layout = Layout::from_size_align(4096, 8).expect("Invalid layout");
        let raw: *mut u8 = alloc::alloc_zeroed(layout);

        println!("buffer at {:p}", raw);

        let message = WriteTextParameters {
            text: "hejsan".to_string()
        };

        let written = message.write_at(raw);
        println!("wrote {}", written);

        dump_mem(raw as *const u64, 3);

        let read = WriteTextParameters::reconstruct_at_inline(raw);
        println!("read {}", read);

        let reconstructed = (raw as *const WriteTextParameters).as_ref().unwrap();

        println!("{}", reconstructed.text);

        // assert_eq!(reconstructed.text, "hejsan");

        // assert_eq!(1, 1);
    }
}

fn main() {
    test_write_and_read();
}
