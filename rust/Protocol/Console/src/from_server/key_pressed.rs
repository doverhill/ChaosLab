use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

struct KeyPressedParameters {
    key_code: KeyCode,
}
impl KeyPressedParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, key_code: KeyCode) -> usize {
        let object: *mut KeyPressedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<KeyPressedParameters>() as isize);

        // key_code

        // return
        mem::size_of::<KeyPressedParameters>()
    }
}


