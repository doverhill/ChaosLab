use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

struct Image {
    size: Size,
    pixels: Vec<Color>,
}


