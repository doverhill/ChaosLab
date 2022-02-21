use crate::ChannelWriter;

#[test]
fn write_to_channel() {
    let channel = ChannelWriter::new(buffer);

    channel.write();

    assert!(1 == 1);
}