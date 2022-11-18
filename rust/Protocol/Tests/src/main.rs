fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::{
        alloc::{self, Layout},
        mem,
    };

    struct Color {
        a: u8,
        r: u8,
        g: u8,
        b: u8,
    }

    impl Color {
        pub fn create_at_address(pointer: *mut u8, a: u8, r: u8, g: u8, b: u8) -> usize {
            unsafe {
                let color: *mut Color = mem::transmute(pointer);
                (*color).a = a;
                (*color).r = r;
                (*color).g = g;
                (*color).b = b;
                mem::size_of::<Color>()
            }
        }

        pub fn get_from_address(pointer: *mut u8) -> &'static mut Color {
            unsafe {
                let color: *mut Color = mem::transmute(pointer);
                color.as_mut().unwrap()
            }
        }
    }

    struct File {
        name: String,
        path: String,
        size: u64,
    }

    impl File {
        pub fn create_at_address(pointer: *mut u8, name: &str, path: &str, size: u64) -> usize {
            unsafe {
                let file: *mut File = mem::transmute(pointer);
                let str_pointer: *mut u8 = pointer.offset(mem::size_of::<File>() as isize);

                let name_length = name.len();
                *(str_pointer as *mut usize) = name_length;
                let str_pointer = str_pointer.offset(mem::size_of::<usize>() as isize);
                core::ptr::copy(name.as_ptr(), str_pointer, name_length);
                let str_pointer = str_pointer.offset(name_length as isize);

                let path_length = path.len();
                *(str_pointer as *mut usize) = path_length;
                let str_pointer = str_pointer.offset(mem::size_of::<usize>() as isize);
                core::ptr::copy(path.as_ptr(), str_pointer, path_length);
                // str_pointer = str_pointer.offset(path_length);

                (*file).size = size;

                mem::size_of::<File>() + mem::size_of::<usize>() + name_length + mem::size_of::<usize>() + path_length
            }
        }

        pub fn get_from_address(pointer: *mut u8) -> &'static mut File {
            unsafe {
                let file: *mut File = mem::transmute(pointer);
                let str_pointer: *mut u8 = pointer.offset(mem::size_of::<File>() as isize);

                let name_length = *(str_pointer as *mut usize);
                let str_pointer = str_pointer.offset(mem::size_of::<usize>() as isize);                
                (*file).name = core::str::from_utf8_unchecked(core::slice::from_raw_parts(str_pointer as *const u8, name_length)).to_owned();
                let str_pointer = str_pointer.offset(name_length as isize);

                let path_length = *(str_pointer as *mut usize);
                let str_pointer = str_pointer.offset(mem::size_of::<usize>() as isize);               
                (*file).path = core::str::from_utf8_unchecked(core::slice::from_raw_parts(str_pointer as *const u8, path_length)).to_owned();
                // str_pointer = str_pointer.offset(name_length);

                file.as_mut().unwrap()
            }
        }
    }

    #[test]
    fn test_color_create_at_and_get_from() {
        unsafe {
            let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
            let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

            let size = Color::create_at_address(raw, 1, 2, 3, 4);
            
            assert_eq!(mem::size_of::<Color>(), size);
            
            let color = Color::get_from_address(raw);

            assert_eq!(1, color.a);
            assert_eq!(2, color.r);
            assert_eq!(3, color.g);
            assert_eq!(4, color.b);
        }
    }

    #[test]
    fn test_file_create_at_and_get_from() {
        unsafe {
            let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
            let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

            let size = File::create_at_address(raw, "this is the name", "this is the path", 4);

            assert_eq!(mem::size_of::<File>() + 2 * mem::size_of::<usize>() + "this is the name".len() + "this is the path".len(), size);

            let file = File::get_from_address(raw);

            assert_eq!("this is the name", file.name);
            assert_eq!("this is the path", file.path);
            assert_eq!(4, file.size);
        }
    }
}
