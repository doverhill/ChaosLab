enum StructFieldValueEnum {
    TypeI64(i64),
    TypeBool(bool),
    TypeString(String),
    TypeNone,
}

impl StructFieldValueEnum {
    pub unsafe fn create_at_address(&self, pointer: *mut u8) -> usize {
        let mut size: usize = mem::size_of::<StructFieldValueEnum>();
        core::ptr::copy(self as *const StructFieldValueEnum, pointer as *mut StructFieldValueEnum, 1);

        match self {
            StructFieldValueEnum::TypeI64(value) => {
                size
            },
            StructFieldValueEnum::TypeBool(value) => {
                size
            },
            StructFieldValueEnum::TypeString(value) => {
                let _value_length = value.len();
                *(pointer as *mut usize) = _value_length;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                core::ptr::copy(value.as_ptr(), pointer, _value_length);
                let pointer = pointer.offset(_value_length as isize);
                size += mem::size_of::<usize>() + _value_length;
                size
            },
            StructFieldValueEnum::TypeNone => {
                size
            },
        }
    }
}


struct StructField {
    name: String,
    value: StructFieldValueEnum,
}


