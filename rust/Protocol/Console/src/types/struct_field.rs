enum StructFieldValueEnum {
    TypeI64(i64),
    TypeBool(bool),
    TypeString(String),
    TypeNone,
}

impl StructFieldValueEnum {
    pub const OPTION_I64: usize = 1;
    pub const OPTION_BOOL: usize = 2;
    pub const OPTION_STRING: usize = 3;
    pub const OPTION_NONE: usize = 4;

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


