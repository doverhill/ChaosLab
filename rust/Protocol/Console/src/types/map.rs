#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Map {
    pub name: String,
    pub description: String,
    pub fields: Vec<MapField>,
}


