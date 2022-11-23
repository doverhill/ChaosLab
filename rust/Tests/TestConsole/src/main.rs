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
        let (size_read, result) = GetCapabilitiesReturns::get_from_address(raw);

        assert_eq!(size_write, size_read);
        assert_eq!(true, (*result).is_framebuffer);
        assert_eq!(1024, (*result).framebuffer_size.width);
        assert_eq!(768, (*result).framebuffer_size.height);
        assert_eq!(80, (*result).text_size.width);
        assert_eq!(50, (*result).text_size.height);
    }
}
