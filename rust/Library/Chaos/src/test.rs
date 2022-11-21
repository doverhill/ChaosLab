use crate::ChannelWriter;
use std::alloc::{alloc, Layout};

fn get_buffer() -> *mut u8 {
    let layout = Layout::from_size_align(4096, 8).unwrap();
    unsafe { alloc(layout) }
}

const SIMPLE_MESSAGE_ID: u64 = 1;
struct SimpleMessage {
    x: i32,
    y: u64
}

#[test]
fn write_simple_message_to_channel() {
    let buffer = get_buffer();
    
    let mut writer = ChannelWriter::new(buffer);
    let mut message = writer.allocate::<SimpleMessage>(1);
    message.x = -44;
    message.y = 827;
    writer.done(message);

    let reader = ChannelReader::new(buffer);


    assert!(1 == 1);
}
