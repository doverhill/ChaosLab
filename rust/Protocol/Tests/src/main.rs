fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::{
        alloc::{self, Layout},
        mem::{self, ManuallyDrop},
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
                let object: *mut Color = mem::transmute(pointer);
                (*object).a = a;
                (*object).r = r;
                (*object).g = g;
                (*object).b = b;
                mem::size_of::<Color>()
            }
        }

        pub fn get_from_address(pointer: *mut u8) -> (usize, &'static mut Color) {
            unsafe {
                let object: *mut Color = mem::transmute(pointer);
                (mem::size_of::<Color>(), object.as_mut().unwrap())
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
                let object: *mut File = mem::transmute(pointer);

                let pointer: *mut u8 = pointer.offset(mem::size_of::<File>() as isize);

                let name_length = name.len();
                *(pointer as *mut usize) = name_length;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                core::ptr::copy(name.as_ptr(), pointer, name_length);
                let pointer = pointer.offset(name_length as isize);

                let path_length = path.len();
                *(pointer as *mut usize) = path_length;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                core::ptr::copy(path.as_ptr(), pointer, path_length);
                // str_pointer = str_pointer.offset(path_length);

                (*object).size = size;

                mem::size_of::<File>()
                    + mem::size_of::<usize>()
                    + name_length
                    + mem::size_of::<usize>()
                    + path_length
            }
        }

        pub fn get_from_address(pointer: *mut u8) -> (usize, &'static mut File) {
            unsafe {
                let object: *mut File = mem::transmute(pointer);

                let pointer: *mut u8 = pointer.offset(mem::size_of::<File>() as isize);

                let name_length = *(pointer as *mut usize);
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                (*object).name = core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                    pointer as *const u8,
                    name_length,
                ))
                .to_owned();
                let pointer = pointer.offset(name_length as isize);

                let path_length = *(pointer as *mut usize);
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                (*object).path = core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                    pointer as *const u8,
                    path_length,
                ))
                .to_owned();
                // str_pointer = str_pointer.offset(name_length);

                (
                    mem::size_of::<File>()
                        + mem::size_of::<usize>()
                        + name_length
                        + mem::size_of::<usize>()
                        + path_length,
                    object.as_mut().unwrap(),
                )
            }
        }
    }

    struct Size {
        width: i64,
        height: i64,
    }

    struct Point {
        x: i64,
        y: i64,
    }

    struct Image {
        size: Size,
        pixels: ManuallyDrop<Vec<Color>>,
    }

    struct ImagePatch {
        position: Point,
        image: Image,
    }

    impl ImagePatch {
        pub fn create_at_address(
            pointer: *mut u8,
            x: i64,
            y: i64,
            width: i64,
            height: i64,
            pixels_count: usize,
        ) -> (usize, ManuallyDrop<Vec<Color>>) {
            unsafe {
                let object: *mut ImagePatch = mem::transmute(pointer);

                (*object).position.x = x;
                (*object).position.y = y;
                (*object).image.size.width = width;
                (*object).image.size.height = height;

                let pointer: *mut u8 = pointer.offset(mem::size_of::<ImagePatch>() as isize);

                *(pointer as *mut usize) = pixels_count;
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let pixels =
                    Vec::<Color>::from_raw_parts(pointer as *mut Color, pixels_count, pixels_count);

                (
                    mem::size_of::<ImagePatch>()
                        + mem::size_of::<usize>()
                        + pixels_count * mem::size_of::<Color>(),
                    mem::ManuallyDrop::new(pixels),
                )
            }
        }

        pub fn get_from_address(pointer: *mut u8) -> (usize, &'static mut ImagePatch) {
            unsafe {
                let object: *mut ImagePatch = mem::transmute(pointer);

                let pointer: *mut u8 = pointer.offset(mem::size_of::<ImagePatch>() as isize);

                let pixels_count = *(pointer as *mut usize);
                let pointer = pointer.offset(mem::size_of::<usize>() as isize);
                let pixels = Vec::from_raw_parts(pointer as *mut Color, pixels_count, pixels_count);
                (*object).image.pixels = ManuallyDrop::new(pixels);

                (
                    mem::size_of::<ImagePatch>()
                        + mem::size_of::<usize>()
                        + pixels_count * mem::size_of::<Color>(),
                    object.as_mut().unwrap(),
                )
            }
        }
    }

    #[test]
    fn test_imagepatch_create_at_and_get_from() {
        unsafe {
            let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
            let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

            let (size, mut pixels) = ImagePatch::create_at_address(raw, 10, 15, 4, 2, 8);
            assert_eq!(
                mem::size_of::<ImagePatch>()
                    + mem::size_of::<usize>()
                    + 8 * mem::size_of::<Color>(),
                size
            );

            pixels[0].a = 88;
            pixels[7].b = 99;

            let (size, patch) = ImagePatch::get_from_address(raw);
            assert_eq!(
                mem::size_of::<ImagePatch>()
                    + mem::size_of::<usize>()
                    + 8 * mem::size_of::<Color>(),
                size
            );

            assert_eq!(10, patch.position.x);
            assert_eq!(15, patch.position.y);
            assert_eq!(4, patch.image.size.width);
            assert_eq!(2, patch.image.size.height);
            assert_eq!(8, patch.image.pixels.len());
            assert_eq!(88, patch.image.pixels[0].a);
            assert_eq!(99, patch.image.pixels[7].b);
        }
    }

    #[test]
    fn test_color_create_at_and_get_from() {
        unsafe {
            let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
            let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

            let size = Color::create_at_address(raw, 1, 2, 3, 4);
            assert_eq!(mem::size_of::<Color>(), size);

            let (size, color) = Color::get_from_address(raw);
            assert_eq!(mem::size_of::<Color>(), size);

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
            assert_eq!(
                mem::size_of::<File>()
                    + 2 * mem::size_of::<usize>()
                    + "this is the name".len()
                    + "this is the path".len(),
                size
            );

            let (size, file) = File::get_from_address(raw);
            assert_eq!(
                mem::size_of::<File>()
                    + 2 * mem::size_of::<usize>()
                    + "this is the name".len()
                    + "this is the path".len(),
                size
            );

            assert_eq!("this is the name", file.name);
            assert_eq!("this is the path", file.path);
            assert_eq!(4, file.size);
        }
    }
}
