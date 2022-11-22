enum WriteObjectsParametersObjectsEnum {
    TypeTable(Table),
    TypeStruct(Struct),
}

impl WriteObjectsParametersObjectsEnum {
    pub const OPTION_TABLE: usize = 1;
    pub const OPTION_STRUCT: usize = 2;

    pub unsafe fn create_at_address(&self, pointer: *mut u8) -> usize {
        let mut size: usize = mem::size_of::<WriteObjectsParametersObjectsEnum>();
        core::ptr::copy(self as *const WriteObjectsParametersObjectsEnum, pointer as *mut WriteObjectsParametersObjectsEnum, 1);

        match self {
            WriteObjectsParametersObjectsEnum::TypeTable(value) => {
                size
            },
            WriteObjectsParametersObjectsEnum::TypeStruct(value) => {
                size
            },
        }
    }
}


struct WriteObjectsParameters {
    objects: Vec<WriteObjectsParametersObjectsEnum>,
}


impl WriteObjectsParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, objects: Vec<WriteObjectsParametersObjectsEnum>) -> usize {
        let object: *mut WriteObjectsParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<WriteObjectsParameters>() as isize);

        // objects
        *(pointer as *mut usize) = objects.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _objects_size: usize = 0;
        for item in objects.iter() {
            let item_size = item.create_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _objects_size += item_size;
        }

        // return
        mem::size_of::<WriteObjectsParameters>() + _objects_size
    }
}



