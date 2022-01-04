use library_chaos::{ Error, Channel, ChannelObject };
use core::{ mem, ptr, str, slice };
use std::{ iter::Iterator, Arc, Mutex };

pub const BOGUS_AUTO_RENDER_CLIENT_TO_SERVER_MESSAGE: u64 = 3;

