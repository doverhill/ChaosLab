enum WriteObjectsParametersObjectsEnum {
    TypeTable(Table),
    TypeStruct(Struct),
}

struct WriteObjectsParameters {
    objects: Vec<WriteObjectsParametersObjectsEnum>,
}

impl WriteObjectsParameters {
}

