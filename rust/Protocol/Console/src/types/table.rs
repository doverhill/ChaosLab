use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

struct Table {
    name: String,
    description: String,
    columns: Vec<String>,
    rows: Vec<Map>,
}


