// pub struct ChannelIterator<T> {
//     channel_pointer: *mut u8,
//     index: usize,
//     count: usize,
//     item_size: usize,
//     refill_handler: fn() -> ()
// }

// impl ChannelIterator<T> {
//     pub fn new(channel_pointer: *mut u8, count: usize, item_size: usize, refill_handler: Option<fn() -> ()>) -> ChannelIterator<T> {
//         ChannelIterator {
//             channel_pointer: channel_pointer,
//             index: 0,
//             count: count,
//             item_size: item_size,
//             refill_handler: refill_handler
//         }
//     }

//     unsafe pub fn next_raw(self) -> Option<*mut T> {
//         println!("returning pointer to item {}", self.index);
//         (*mut T)(self.channel_pointer + self.index++ * self.item_size);
//     }
// }

// impl Iterator<T> for ChannelIterator<T> {
//     pub fn next(self) -> Option<T> {
//         unsafe {
//             *self.next_raw();
//         }
//     }
// }