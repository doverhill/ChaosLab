enum StructFieldValueEnum {
    TypeI64(i64),
    TypeBool(bool),
    TypeString(String),
    TypeNone,
}

struct StructField {
    name: String,
    value: StructFieldValueEnum,
}

