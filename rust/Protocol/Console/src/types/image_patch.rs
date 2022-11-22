use std::mem;
use std::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

struct ImagePatch {
    image: Image,
    position: Point,
}


