#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub struct Image {
    pub size: Size,
    pub pixels: Vec<Color>,
}


