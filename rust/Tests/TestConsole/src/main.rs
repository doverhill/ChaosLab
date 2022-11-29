use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    alloc::{self, Layout},
    mem::{self, ManuallyDrop},
};

extern crate protocol_console;

mod test_type;
use test_type::*;

use protocol_console::*;

// fn main() {
//     test_type();
// }

#[test]
fn test_type() {
    unsafe {
        let layout = Layout::from_size_align(4096, 8).expect("Invalid layout");
        let raw1: *mut u8 = mem::transmute(alloc::alloc(layout));
        let raw2: *mut u8 = mem::transmute(alloc::alloc(layout));

        let q = TestType {
            name: "q".to_string(),
            size: 9,
            objects: vec![],
            cities: vec![],
            other: OtherType { include: false, offset: 7000, paths: vec![] },
            numbers: vec![ 3, 4, 5 ],
        };

        let other1 = OtherType {
            include: true,
            offset: -5,
            paths: vec![ OtherTypePathsEnum::TypeTestType(q), OtherTypePathsEnum::TypeI64(99), OtherTypePathsEnum::TypeString("hejsan".to_string()) ],
        };

        let other2 = OtherType { include: false, offset: 767, paths: vec![] };

        let test = TestType {
            name: "apa".to_string(),
            size: 77,
            objects: vec![ other1, other2 ],
            cities: vec![ "uppsala".to_string(), "stockholm".to_string() ],
            other: OtherType { include: true, offset: -9999, paths: vec![ OtherTypePathsEnum::TypeI64(-23) ] },
            numbers: vec![ 99, 23, 4, 54, 23, 5 ],
        };

        let size_write = test.write_at(raw1);
        core::ptr::copy(raw1, raw2, 4096);
        let size_read = TestType::reconstruct_at_inline(raw1);
        let test_read = raw2 as *const TestType;

        assert_eq!(size_read, size_write);
        assert_eq!(test.name, (*test_read).name);
        assert_eq!(test.name, "apa");
        assert_eq!((*test_read).name, "apa");
        assert_eq!(test.size, (*test_read).size);
        assert_eq!(test.size, 77);
        assert_eq!((*test_read).size, 77);
        assert_eq!(test.objects.len(), (*test_read).objects.len());
        assert_eq!(test.objects.len(), 2);
        assert_eq!((*test_read).objects.len(), 2);

        assert_eq!(test.objects[0].include, (*test_read).objects[0].include);
        assert_eq!(test.objects[0].include, true);
        assert_eq!((*test_read).objects[0].include, true);
        assert_eq!(test.objects[1].include, (*test_read).objects[1].include);
        assert_eq!(test.objects[1].include, false);
        assert_eq!((*test_read).objects[1].include, false);

        assert_eq!(test.objects[0].offset, (*test_read).objects[0].offset);
        assert_eq!(test.objects[0].offset, -5);
        assert_eq!((*test_read).objects[0].offset, -5);
        assert_eq!(test.objects[1].offset, (*test_read).objects[1].offset);
        assert_eq!(test.objects[1].offset, 767);
        assert_eq!((*test_read).objects[1].offset, 767);

        assert_eq!(test.objects[0].paths.len(), (*test_read).objects[0].paths.len());
        assert_eq!(test.objects[0].paths.len(), 3);
        assert_eq!((*test_read).objects[0].paths.len(), 3);

        assert!(if let OtherTypePathsEnum::TypeTestType(a) = &test.objects[0].paths[0] { assert_eq!(a.name, "q"); assert_eq!(a.size, 9); assert_eq!(a.objects.len(), 0); true } else { false });
        assert!(if let OtherTypePathsEnum::TypeI64(a) = &test.objects[0].paths[1] { assert_eq!(*a, 99); true } else { false });
        assert!(if let OtherTypePathsEnum::TypeString(a) = &test.objects[0].paths[2] { assert_eq!(a, "hejsan"); true } else { false });

        let other3 = OtherType { include: true, offset: -334, paths: vec![] };
        let other4 = OtherType { include: false, offset: -33, paths: vec![] };
        let other5 = OtherType { include: false, offset: -3, paths: vec![] };

        let test = TestType {
            name: "xyz".to_string(),
            size: 3,
            objects: vec![ other3, other4, other5 ],
            cities: vec![],
            other: OtherType { include: false, offset: 1, paths: vec![] },
            numbers: vec![ 34, 2, 3, 4, 1, 2, 0 ],
        };

        let size_write = test.write_at(raw1);
        core::ptr::copy(raw1, raw2, 1024);
        let size_read = TestType::reconstruct_at_inline(raw2);
        let test_read = raw2 as *const TestType;

        assert_eq!(size_read, size_write);
        assert_eq!(test.name, (*test_read).name);
        assert_eq!(test.name, "xyz");
        assert_eq!((*test_read).name, "xyz");
        assert_eq!(test.size, (*test_read).size);
        assert_eq!(test.size, 3);
        assert_eq!((*test_read).size, 3);

        assert_eq!(test.objects.len(), (*test_read).objects.len());
        assert_eq!(test.objects.len(), 3);
        assert_eq!((*test_read).objects.len(), 3);

        assert_eq!(test.objects[0].include, (*test_read).objects[0].include);
        assert_eq!(test.objects[1].include, (*test_read).objects[1].include);
        assert_eq!(test.objects[2].include, (*test_read).objects[2].include);

        assert_eq!(test.objects[0].offset, (*test_read).objects[0].offset);
        assert_eq!(test.objects[1].offset, (*test_read).objects[1].offset);
        assert_eq!(test.objects[2].offset, (*test_read).objects[2].offset);

    }
}

#[test]
fn test_get_capabilities_returns() {
    unsafe {
        let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
        let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

        let test = GetCapabilitiesReturns {
            framebuffer_size: Size { width: 1024, height: 768 },
            is_framebuffer: true,
            text_size: Size { width: 80, height: 50 },
        };
        let size_write = test.write_at(raw);
        assert!(size_write > 0);

        let size_read = GetCapabilitiesReturns::reconstruct_at_inline(raw);
        let result = raw as *const GetCapabilitiesReturns;
        assert_eq!(size_write, size_read);

        assert_eq!(true, (*result).is_framebuffer);
        assert_eq!(1024, (*result).framebuffer_size.width);
        assert_eq!(768, (*result).framebuffer_size.height);
        assert_eq!(80, (*result).text_size.width);
        assert_eq!(50, (*result).text_size.height);
    }
}

// #[test]
// fn test_write_objects_parameters_empty() {
//     unsafe {
//         let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
//         let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

//         let objects: Vec<WriteObjectsParametersObjectsEnum> = Vec::new();
//         let size_write = WriteObjectsParameters::create_at_address(raw, objects);
//         assert!(size_write > 0);

//         let (size_read, result) = WriteObjectsParameters::get_from_address(raw);
//         assert_eq!(size_write, size_read);
//         assert_eq!(0, (*result).objects.len());
//     }
// }

// #[test]
// fn test_draw_image_patch_nonempty() {
//     unsafe {
//         let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
//         let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

//         let (write_size, mut pixels) = DrawImagePatchParameters::create_at_address(raw, 7, 8, 56, 1, 2);
//         pixels[0].alpha = 1;
//         pixels[55] = Color { alpha: 4, red: 5, green: 6, blue: 7 };
//         assert!(write_size > 0);
//     }
// }

// FIXME: create a version of create_at_address that don't take vecs of pointers
// FIXME: fix get_from_address to use ManuallyDrop where necessary to prevent crash
//#[test]
// fn test_write_objects_parameters_nonempty() {
//     unsafe {
//         let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
//         let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

//         let mut map_field1 = MapField {
//             name: "field".to_string(),
//             value: MapFieldValueEnum::TypeI64(77 as i64)
//         };
//         let mut map = Map { 
//             name: "map".to_string(), 
//             description: "mapdescr".to_string(),
//             fields: vec!(&mut map_field1 as *mut MapField),
//         };
//         let mut table_rows = vec!(&mut map as *mut Map);
//         let mut table = Table {
//             name: "name".to_string(),
//             description: "description".to_string(),
//             columns: vec!("a".to_string(), "b".to_string()),
//             rows: table_rows,
//         };
//         let mut objects: Vec<WriteObjectsParametersObjectsEnum> = Vec::new();
//         objects.push(WriteObjectsParametersObjectsEnum::TypeTable(&mut table as *mut Table));
//         objects.push(WriteObjectsParametersObjectsEnum::TypeMap(&mut map as *mut Map));
//         let size_write = WriteObjectsParameters::create_at_address(raw, objects);
//         assert!(size_write > 0);

//         let (size_read, result) = WriteObjectsParameters::get_from_address(raw);
//         // assert_eq!(size_write, size_read);
//         // assert_eq!(2, (*result).objects.len());
//     }
// }
