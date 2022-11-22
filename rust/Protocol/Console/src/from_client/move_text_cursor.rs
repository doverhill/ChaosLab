struct MoveTextCursorParameters {
    position: Point,
}


impl MoveTextCursorParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, position_x: i64, position_y: i64) -> usize {
        let object: *mut MoveTextCursorParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<MoveTextCursorParameters>() as isize);

        // position
        (*object).position.x = position_x;
        (*object).position.y = position_y;

        // return
        mem::size_of::<MoveTextCursorParameters>()
    }
}



