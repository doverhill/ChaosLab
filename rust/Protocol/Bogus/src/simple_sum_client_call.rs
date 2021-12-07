use library_chaos::{ Error, Channel, ChannelObject };

pub const BOGUS_SIMPLE_SUM_ARGUMENTS_OBJECT_ID: usize = 1;
struct SimpleSumArguments {
    x: i32,
    y: i32
}

impl ChannelObject for SimpleSumArguments {
    unsafe fn write_to_channel(self, pointer: *mut u8) {
        *(pointer as *mut SimpleSumArguments) = self;
    }

    unsafe fn from_channel(pointer: *const u8) -> &'static mut Self {
        &mut *(pointer as *mut SimpleSumArguments)
    }
}

pub const BOGUS_SIMPLE_SUM_RESULT_OBJECT_ID: usize = 2;
struct SimpleSumResult {
    result: i32
}

impl ChannelObject for SimpleSumResult {
    unsafe fn write_to_channel(self, pointer: *mut u8) {
        *(pointer as *mut SimpleSumResult) = self;
    }

    unsafe fn from_channel(pointer: *const u8) -> &'static mut Self {
        &mut *(pointer as *mut SimpleSumResult)
    }
}

pub fn call(channel: &mut Channel, x: i32, y: i32) -> Result<i32, Error> {
    channel.start();
    let arguments = SimpleSumArguments {
        x: x,
        y: y
    };
    channel.add_object(BOGUS_SIMPLE_SUM_ARGUMENTS_OBJECT_ID, arguments);
    
    match channel.call_sync(crate::client::BOGUS_SIMPLE_SUM_CLIENT_MESSAGE, 1000) {
        Ok(()) => {
            if channel.get_object_count() != 1 {
                println!("Error: Expected object count 1 in simple_sum result!");
                Err(Error::General)
            }
            else {
                // read result
                if channel.get_object_id(0) != BOGUS_SIMPLE_SUM_RESULT_OBJECT_ID {
                    println!("Error: Expected object id {} in simple_sum result!", BOGUS_SIMPLE_SUM_RESULT_OBJECT_ID);
                    Err(Error::General)
                }
                else {
                    Ok(channel.get_object::<SimpleSumResult>(0).result)
                }
            }
        },
        Err(error) => {
            Err(error)
        }
    }
}