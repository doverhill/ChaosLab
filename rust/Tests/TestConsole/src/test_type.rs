use core::ptr::addr_of_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    alloc::{self, Layout},
    mem::{self, ManuallyDrop},
};

pub struct ProtocolVersion {
    major: u16,
    minor: u16,
    patch: u16,
    is_preview: bool,
    preview_version: u16,
}

pub struct ChannelHeader {
    lock: AtomicBool,
    channel_magic: u64,
    protocol_name: [u8; 32],
    protocol_version: ProtocolVersion,
    first_message_offset: usize,
    last_message_offset: usize,
    number_of_messages: usize,
    is_writing: bool,
}

impl ChannelHeader {
    pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'C' as u8, 'H' as u8, 'A' as u8, 'N' as u8, 'N' as u8, 'E' as u8, 'L' as u8]);
}

pub struct ChannelMessageHeader {
    message_magic: u64,
    message_id: usize,
    message_length: usize,
    previous_message_offset: usize,
    next_message_offset: usize,
    replace_pending: bool,
}

impl ChannelMessageHeader {
    pub const MAGIC: u64 = u64::from_be_bytes([
        'C' as u8, 'M' as u8, 'E' as u8, 'S' as u8, 'S' as u8, 'A' as u8, 'G' as u8, 'E' as u8,
    ]);
}


struct ChannelLock {
    channel_header: *const ChannelHeader,
}

impl ChannelLock {
    pub unsafe fn get(channel: &ConsoleChannel) -> Self {
        let channel_header = channel.channel_address as *const ChannelHeader;
        while (*channel_header).lock.swap(true, Ordering::Acquire) {}
        Self {
            channel_header: channel_header,
        }
    }
}

impl Drop for ChannelLock {
    fn drop(&mut self) {
        unsafe {
            (*self.channel_header).lock.swap(false, Ordering::Release);
        }
    }
}

pub struct ConsoleChannel {
    channel_address: *mut u8,
}

impl ConsoleChannel {
    pub unsafe fn new(channel_address: *mut u8, is_server: bool) -> Self {
        if !is_server {
            let channel_header = channel_address as *mut ChannelHeader;
            (*channel_header).lock.store(false, Ordering::Relaxed);
            (*channel_header).channel_magic = ChannelHeader::MAGIC;
            (*channel_header).protocol_name[0] = 7;
            (*channel_header).protocol_name[1] = 'c' as u8;
            (*channel_header).protocol_name[2] = 'o' as u8;
            (*channel_header).protocol_name[3] = 'n' as u8;
            (*channel_header).protocol_name[4] = 's' as u8;
            (*channel_header).protocol_name[5] = 'o' as u8;
            (*channel_header).protocol_name[6] = 'l' as u8;
            (*channel_header).protocol_name[7] = 'e' as u8;
            (*channel_header).protocol_version = ProtocolVersion {
                major: 1,
                minor: 0,
                patch: 0,
                is_preview: false,
                preview_version: 0,
            };
            (*channel_header).first_message_offset = 0;
            (*channel_header).last_message_offset = 0;
            (*channel_header).number_of_messages = 0;
            (*channel_header).is_writing = false;
        }

        ConsoleChannel {
            channel_address: channel_address,
        }
    }

    pub unsafe fn check_compatible(&self) -> bool {
        let channel_header = self.channel_address as *mut ChannelHeader;

        (*channel_header).channel_magic == ChannelHeader::MAGIC
            && (*channel_header).protocol_version.major == 1
            && (*channel_header).protocol_name[0] == 7
            && (*channel_header).protocol_name[1] == 'c' as u8
            && (*channel_header).protocol_name[2] == 'o' as u8
            && (*channel_header).protocol_name[3] == 'n' as u8
            && (*channel_header).protocol_name[4] == 's' as u8
            && (*channel_header).protocol_name[5] == 'o' as u8
            && (*channel_header).protocol_name[6] == 'l' as u8
            && (*channel_header).protocol_name[7] == 'e' as u8
    }

    pub unsafe fn prepare_message(&self, message_id: usize, replace_pending: bool) -> *mut u8 {
        let channel_header = self.channel_address as *mut ChannelHeader;

        let lock = ChannelLock::get(self);
        #[cfg(debug)]
        assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
        assert!(!(*channel_header).is_writing);

        let mut message: *mut ChannelMessageHeader;
        if (*channel_header).number_of_messages == 0 { 
            // create first message in channel
            (*channel_header).first_message_offset = mem::size_of::<ChannelHeader>();
            (*channel_header).last_message_offset = mem::size_of::<ChannelHeader>();
            message = self.channel_address.offset(mem::size_of::<ChannelHeader>() as isize) as *mut ChannelMessageHeader;
            (*message).previous_message_offset = 0;
        }
        else {
            let last_message_offset = (*channel_header).last_message_offset;
            let last_message = self.channel_address.offset((*channel_header).last_message_offset as isize) as *mut ChannelMessageHeader;
            (*last_message).next_message_offset = (*channel_header).last_message_offset + (*last_message).message_length;
            message = self.channel_address.offset((*last_message).next_message_offset as isize) as *mut ChannelMessageHeader;
            (*message).previous_message_offset = last_message_offset;
        }

        (*channel_header).is_writing = true;

        (*message).message_magic = ChannelMessageHeader::MAGIC;
        (*message).message_id = message_id;
        (*message).replace_pending = replace_pending;
        (*message).message_length = 0;
        (*message).next_message_offset = 0;

        message as *mut u8
    }

    pub unsafe fn commit_message(&self, message_payload_size: usize) {
        let channel_header = self.channel_address as *mut ChannelHeader;

        let lock = ChannelLock::get(self);
        let last_message = self.channel_address.offset((*channel_header).last_message_offset as isize) as *mut ChannelMessageHeader;
        #[cfg(debug)]
        assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
        assert!((*channel_header).is_writing);
        #[cfg(debug)]
        assert!((*last_message).message_magic == ChannelMessageHeader::MAGIC);

        (*channel_header).is_writing = false;
        (*channel_header).number_of_messages = (*channel_header).number_of_messages + 1;
        (*last_message).message_length = mem::size_of::<ChannelMessageHeader>() + message_payload_size;
    }

    pub unsafe fn find_message(&self) -> Option<*mut ChannelMessageHeader> {
        let channel_header = self.channel_address as *mut ChannelHeader;

        let lock = ChannelLock::get(self);
        let first_message = self.channel_address.offset((*channel_header).last_message_offset as isize) as *mut ChannelMessageHeader;
        #[cfg(debug)]
        assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
        #[cfg(debug)]
        assert!((*first_message).message_magic == ChannelMessageHeader::MAGIC);

        if (*channel_header).number_of_messages == 0 {
            return None;
        }
        else {
            if !(*first_message).replace_pending {
                return Some(first_message);
            }
            else {
                let mut last_of_kind = first_message;
                let iter = first_message;
                while (*iter).next_message_offset != 0 {
                    let iter = self.channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;
                    if (*iter).message_id == (*first_message).message_id {
                        last_of_kind = iter;
                    }
                }
                let iter = first_message;
                while (*iter).next_message_offset != 0 {
                    if (*iter).message_id == (*first_message).message_id && iter != last_of_kind {
                        assert!((*channel_header).number_of_messages > 1);
                        self.unlink_message(iter, true);
                    }
                    let iter = self.channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;
                }
                Some(last_of_kind)
            }
        }

    }

    pub unsafe fn unlink_message(&self, message: *mut ChannelMessageHeader, lock_held: bool) {
        let channel_header = self.channel_address as *mut ChannelHeader;

        let lock = if lock_held { None } else { Some(ChannelLock::get(self)) };
        #[cfg(debug)]
        assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);
        #[cfg(debug)]
        assert!((*message).message_magic == ChannelMessageHeader::MAGIC);

        if (*message).previous_message_offset == 0 {
            // first message
            (*channel_header).first_message_offset = (*message).next_message_offset;
        }
        else {
            let previous_message = self.channel_address.offset((*message).previous_message_offset as isize) as *mut ChannelMessageHeader;
            (*previous_message).next_message_offset = (*message).next_message_offset;
        }

        if (*message).next_message_offset == 0 {
            // last message
            (*channel_header).last_message_offset = (*message).previous_message_offset;
        }
        else {
            let next_message = self.channel_address.offset((*message).next_message_offset as isize) as *mut ChannelMessageHeader;
            (*next_message).previous_message_offset = (*message).previous_message_offset;
        }

        (*channel_header).number_of_messages = (*channel_header).number_of_messages - 1;
    }
}

// OtherType IDL
// paths: TestType|i64|string[]
// include: bool
// offset: i64

#[repr(C, u64)]
pub enum OtherTypePathsEnum {
    TypeTestType(TestType),
    TypeI64(i64),
    TypeString(String),
}

#[repr(C)]
struct OtherTypePathsEnumStruct {
    tag: OtherTypePathsEnumStructTag,
    payload: OtherTypePathsEnumStructPayload,
}

#[repr(u64)]
enum OtherTypePathsEnumStructTag {
    TypeTestType,
    TypeI64,
    TypeString,
}

#[repr(C)]
union OtherTypePathsEnumStructPayload {
    payload_test_type: ManuallyDrop<TestType>,
    payload_i64: i64,
    payload_string: ManuallyDrop<String>,
}

impl OtherTypePathsEnum {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut OtherTypePathsEnum, 1);
        pointer = pointer.offset(mem::size_of::<OtherTypePathsEnum>() as isize);

        mem::size_of::<OtherTypePathsEnum>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;

        println!("writing OtherTypePathsEnum at {:p}", pointer);

        match self {
            OtherTypePathsEnum::TypeTestType(value) => value.write_references_at(pointer),
            OtherTypePathsEnum::TypeI64(value) => 0,
            OtherTypePathsEnum::TypeString(value) => {
                let mut len = value.len();
                *(pointer as *mut usize) = len;
                pointer = pointer.offset(mem::size_of::<usize>() as isize);
                core::ptr::copy(value.as_ptr(), pointer as *mut u8, len);
                len = ((len + 7) / 8) * 8;
                mem::size_of::<usize>() + len
            }
        }
    }

    pub unsafe fn reconstruct_at(
        object_pointer: *mut OtherTypePathsEnum,
        references_pointer: *mut u8,
    ) -> usize {
        let object = object_pointer as *mut OtherTypePathsEnumStruct;

        println!("reading OtherTypePathsEnum at {:p}", references_pointer);

        match (*object).tag {
            OtherTypePathsEnumStructTag::TypeTestType => TestType::reconstruct_at(
                addr_of_mut!((*object).payload.payload_test_type) as *mut TestType,
                references_pointer,
            ),
            OtherTypePathsEnumStructTag::TypeI64 => 0,
            OtherTypePathsEnumStructTag::TypeString => {
                let mut pointer = references_pointer;
                let mut len = *(pointer as *const usize);
                pointer = pointer.offset(mem::size_of::<usize>() as isize);
                (*object).payload.payload_string =
                    ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
                len = ((len + 7) / 8) * 8;
                mem::size_of::<usize>() + len
            }
        }
    }
}

pub struct OtherType {
    pub include: bool,
    pub offset: i64,
    pub paths: Vec<OtherTypePathsEnum>,
}

impl OtherType {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut OtherType, 1);
        pointer = pointer.offset(mem::size_of::<OtherType>() as isize);

        mem::size_of::<OtherType>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // pointer is after TestType, write strings

        // pointer is after strings, write arrays

        // paths - rust will align for us
        println!("writing OtherType.paths at {:p}", pointer);
        let len = self.paths.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.paths.as_ptr(), pointer as *mut OtherTypePathsEnum, len);
        pointer = pointer.offset(len as isize * mem::size_of::<OtherTypePathsEnum>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherTypePathsEnum>();

        for item in self.paths.iter() {
            println!("  writing OtherType.paths item at {:p}", pointer);
            let item_size = item.write_references_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        let size = Self::reconstruct_at(
            object_pointer as *mut OtherType,
            object_pointer.offset(mem::size_of::<OtherType>() as isize),
        );
        mem::size_of::<OtherType>() + size
    }

    pub unsafe fn reconstruct_at(
        object_pointer: *mut OtherType,
        references_pointer: *mut u8,
    ) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // pointer is after TestType, read strings

        // pointer is after strings, read arrays

        // paths - rust will align for us
        println!("reading OtherType.paths from {:p}", pointer);
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(
            pointer as *mut OtherTypePathsEnum,
            len,
            len,
        ));
        core::ptr::write(
            addr_of_mut!((*object_pointer).paths),
            ManuallyDrop::take(&mut assign),
        );
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherTypePathsEnum>();

        // pointer is at first item in array
        // set references pointer to after array
        let mut references_pointer =
            pointer.offset(len as isize * mem::size_of::<OtherTypePathsEnum>() as isize);
        for item in (*object_pointer).paths.iter() {
            println!(
                "  reading OtherType.paths item from {:p}",
                references_pointer
            );
            let item_size = OtherTypePathsEnum::reconstruct_at(
                pointer as *mut OtherTypePathsEnum,
                references_pointer,
            );
            pointer = pointer.offset(mem::size_of::<OtherTypePathsEnum>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        size
    }
}

// TestType IDL
// name: string
// size: u64
// objects: OtherType[]

pub struct TestType {
    pub size: u64,
    pub name: String,
    pub objects: Vec<OtherType>,
    pub other: OtherType,
    pub cities: Vec<String>,
    pub numbers: Vec<u32>,
}

impl TestType {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut TestType, 1);
        pointer = pointer.offset(mem::size_of::<TestType>() as isize);

        mem::size_of::<TestType>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // name - make sure to 8 byte align pointer after writing
        println!("writing TestType.name at {:p}", pointer);
        let mut len = self.name.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.name.as_ptr(), pointer, len);
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // objects - rust will align for us
        println!("writing TestType.objects at {:p}", pointer);
        let len = self.objects.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.objects.as_ptr(), pointer as *mut OtherType, len);
        pointer = pointer.offset(len as isize * mem::size_of::<OtherType>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherType>();

        for item in self.objects.iter() {
            println!("  writing TestType.objects item at {:p}", pointer);
            let item_size = item.write_references_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        // other
        println!("writing TestType.other at {:p}", pointer);
        let len = self.other.write_references_at(pointer);
        println!("===> {} bytes", len);
        pointer = pointer.offset(len as isize);
        size += len;

        // cities
        println!("writing TestType.cities at {:p}", pointer);
        let len = self.cities.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.cities.as_ptr(), pointer as *mut String, len);
        pointer = pointer.offset(len as isize * mem::size_of::<String>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<String>();

        for item in self.cities.iter() {
            println!("writing TestType.cities item at {:p}", pointer);
            let mut len = item.len();
            *(pointer as *mut usize) = len;
            pointer = pointer.offset(mem::size_of::<usize>() as isize);
            core::ptr::copy(item.as_ptr(), pointer, len);
            len = ((len + 7) / 8) * 8;
            pointer = pointer.offset(len as isize);
            size += mem::size_of::<usize>() + len;
        }

        // numbers
        println!("writing TestType.numbers at {:p}", pointer);
        let len = self.numbers.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.numbers.as_ptr(), pointer as *mut u32, len);
        pointer = pointer.offset(len as isize * mem::size_of::<u32>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<u32>();

        // no item loop

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        let size = Self::reconstruct_at(
            object_pointer as *mut TestType,
            object_pointer.offset(mem::size_of::<TestType>() as isize),
        );
        mem::size_of::<TestType>() + size
    }

    pub unsafe fn reconstruct_at(
        object_pointer: *mut TestType,
        references_pointer: *mut u8,
    ) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // name - make sure to 8 byte align pointer after reading
        println!("reading TestType.name from {:p}", pointer);
        let mut len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));
        core::ptr::write(
            addr_of_mut!((*object_pointer).name),
            ManuallyDrop::take(&mut assign),
        );
        len = ((len + 7) / 8) * 8;
        pointer = pointer.offset(len as isize);
        size += mem::size_of::<usize>() + len;

        // objects - rust will align for us
        println!("reading TestType.objects from {:p}", pointer);
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign =
            ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut OtherType, len, len));
        core::ptr::write(
            addr_of_mut!((*object_pointer).objects),
            ManuallyDrop::take(&mut assign),
        );
        size += mem::size_of::<usize>() + len * mem::size_of::<OtherType>();

        // pointer is at first item in array
        // set references pointer to after array
        let mut references_pointer =
            pointer.offset(len as isize * mem::size_of::<OtherType>() as isize);
        for item in (*object_pointer).objects.iter() {
            println!(
                "  reading TestType.objects item from {:p}",
                references_pointer
            );
            let item_size =
                OtherType::reconstruct_at(pointer as *mut OtherType, references_pointer);
            pointer = pointer.offset(mem::size_of::<OtherType>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        // other
        println!("reading TestType.other from {:p}", pointer);
        let len = OtherType::reconstruct_at(addr_of_mut!((*object_pointer).other), pointer);
        println!("===> {} bytes", len);
        pointer = pointer.offset(len as isize);
        size += len;

        // cities
        println!("reading TestType.cities from {:p}", pointer);
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut String, len, len));
        core::ptr::write(
            addr_of_mut!((*object_pointer).cities),
            ManuallyDrop::take(&mut assign),
        );
        size += mem::size_of::<usize>() + len * mem::size_of::<String>();

        let mut references_pointer =
            pointer.offset(len as isize * mem::size_of::<String>() as isize);
        for item in (*object_pointer).cities.iter() {
            println!(
                "  reading TestType.cities item from {:p}",
                references_pointer
            );
            let mut len = *(references_pointer as *const usize);
            references_pointer = references_pointer.offset(mem::size_of::<usize>() as isize);
            let mut assign =
                ManuallyDrop::new(String::from_raw_parts(references_pointer, len, len));
            core::ptr::write(pointer as *mut String, ManuallyDrop::take(&mut assign));
            pointer = pointer.offset(mem::size_of::<String>() as isize);
            len = ((len + 7) / 8) * 8;
            references_pointer = references_pointer.offset(len as isize);
            size += mem::size_of::<usize>() + len;
        }
        pointer = references_pointer;

        // numbers
        println!("reading TestType.numbers from {:p}", pointer);
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut u32, len, len));
        core::ptr::write(
            addr_of_mut!((*object_pointer).numbers),
            ManuallyDrop::take(&mut assign),
        );
        size += mem::size_of::<usize>() + len * mem::size_of::<u32>();

        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<u32>() as isize);
        pointer = references_pointer;

        size
    }
}
