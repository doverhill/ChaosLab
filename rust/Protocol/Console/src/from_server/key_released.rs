struct KeyReleasedParameters {
    key_code: KeyCode,
}


impl KeyReleasedParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, key_code: KeyCode) -> usize {
        let object: *mut KeyReleasedParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<KeyReleasedParameters>() as isize);

        // key_code

        // return
        mem::size_of::<KeyReleasedParameters>()
    }
}



