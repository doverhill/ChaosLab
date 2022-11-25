use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    alloc::{self, Layout},
    mem::{self, ManuallyDrop},
};

use protocol_console::*;

#[test]
fn test_get_capabilities_returns() {
    unsafe {
        let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
        let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

        let size_write = GetCapabilitiesReturns::create_at_address(raw, true, 1024, 768, 80, 50);
        assert!(size_write > 0);

        let (size_read, result) = GetCapabilitiesReturns::get_from_address(raw);
        assert_eq!(size_write, size_read);
        assert_eq!(true, (*result).is_framebuffer);
        assert_eq!(1024, (*result).framebuffer_size.width);
        assert_eq!(768, (*result).framebuffer_size.height);
        assert_eq!(80, (*result).text_size.width);
        assert_eq!(50, (*result).text_size.height);
    }
}

#[test]
fn test_write_objects_parameters_empty() {
    unsafe {
        let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
        let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

        let objects: Vec<WriteObjectsParametersObjectsEnum> = Vec::new();
        let size_write = WriteObjectsParameters::create_at_address(raw, objects);
        assert!(size_write > 0);

        let (size_read, result) = WriteObjectsParameters::get_from_address(raw);
        assert_eq!(size_write, size_read);
        assert_eq!(0, (*result).objects.len());
    }
}

// FIXME: create a version of create_at_address that don't take vecs of pointers
// FIXME: fix get_from_address to use ManuallyDrop where necessary to prevent crash

#[test]
fn test_write_objects_parameters_nonempty() {
    unsafe {
        let layout = Layout::from_size_align(512 * 1024, 4 * 1024).expect("Invalid layout");
        let raw: *mut u8 = mem::transmute(alloc::alloc(layout));

        let mut map_field1 = MapField {
            name: "field".to_string(),
            value: MapFieldValueEnum::TypeI64(77 as i64)
        };
        let mut map = Map { 
            name: "map".to_string(), 
            description: "mapdescr".to_string(),
            fields: vec!(&mut map_field1 as *mut MapField),
        };
        let mut table_rows = vec!(&mut map as *mut Map);
        let mut table = Table {
            name: "name".to_string(),
            description: "description".to_string(),
            columns: vec!("a".to_string(), "b".to_string()),
            rows: table_rows,
        };
        let mut objects: Vec<WriteObjectsParametersObjectsEnum> = Vec::new();
        objects.push(WriteObjectsParametersObjectsEnum::TypeTable(&mut table as *mut Table));
        objects.push(WriteObjectsParametersObjectsEnum::TypeMap(&mut map as *mut Map));
        let size_write = WriteObjectsParameters::create_at_address(raw, objects);
        assert!(size_write > 0);

        let (size_read, result) = WriteObjectsParameters::get_from_address(raw);

panic!("do we get here");

        // assert_eq!(size_write, size_read);
        // assert_eq!(2, (*result).objects.len());
    }
}
