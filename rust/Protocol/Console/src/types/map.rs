use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

struct Map {
    name: String,
    description: String,
    fields: Vec<MapField>,
}


