#![no_std]
#![no_main]

extern crate alloc;
extern crate library_chaos;

use alloc::format;
use library_chaos::EmitType;

library_chaos::main! {
    library_chaos::process_emit(EmitType::Information, "Hello from userspace!");

    // test heap allocation via the MemoryAllocate syscall
    let message = format!("Heap works! 2 + 2 = {}", 2 + 2);
    library_chaos::process_emit(EmitType::Information, &message);
}
