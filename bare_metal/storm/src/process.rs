//! Process and thread management.
//!
//! Each process owns an [`AddressSpace`] and contains one or more [`Thread`]s.
//! The first thread is created automatically when the process is created.

use alloc::vec::Vec;

use crate::address_space::AddressSpace;
use crate::{log, log_println};

static mut NEXT_PROCESS_ID: u64 = 1;
static mut NEXT_THREAD_ID: u64 = 1;

fn allocate_process_id() -> u64 {
    unsafe {
        let id = NEXT_PROCESS_ID;
        NEXT_PROCESS_ID += 1;
        id
    }
}

fn allocate_thread_id() -> u64 {
    unsafe {
        let id = NEXT_THREAD_ID;
        NEXT_THREAD_ID += 1;
        id
    }
}

/// Saved CPU register state for context switching.
/// TODO: populate with actual register fields when we implement scheduling.
#[derive(Debug)]
pub struct SavedState {
    pub instruction_pointer: u64,
    pub stack_pointer: u64,
    pub rflags: u64,
    // TODO: general purpose registers, FPU/SSE state, etc.
}

impl SavedState {
    fn empty() -> Self {
        SavedState {
            instruction_pointer: 0,
            stack_pointer: 0,
            rflags: 0,
        }
    }
}

/// A thread of execution within a process.
#[derive(Debug)]
pub struct Thread {
    pub thread_id: u64,
    pub saved_state: SavedState,
    // TODO: kernel stack for this thread, scheduling state, priority, etc.
}

impl Thread {
    fn new() -> Self {
        let thread_id = allocate_thread_id();
        Thread {
            thread_id,
            saved_state: SavedState::empty(),
        }
    }
}

/// A process with its own address space and one or more threads.
pub struct Process {
    pub process_id: u64,
    pub address_space: AddressSpace,
    pub threads: Vec<Thread>,
}

impl Process {
    /// Create a new process with a fresh address space and one initial thread.
    pub fn create() -> Self {
        let process_id = allocate_process_id();
        let address_space = AddressSpace::new();
        let initial_thread = Thread::new();

        log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
            "Created process {} with address space L4={:#x}, thread {}",
            process_id, address_space.l4_physical_address(), initial_thread.thread_id);

        Process {
            process_id,
            address_space,
            threads: alloc::vec![initial_thread],
        }
    }

    /// Add a new thread to this process.
    pub fn create_thread(&mut self) -> &Thread {
        let thread = Thread::new();
        log_println!(log::SubSystem::Kernel, log::LogLevel::Debug,
            "Created thread {} in process {}", thread.thread_id, self.process_id);
        self.threads.push(thread);
        self.threads.last().unwrap()
    }
}
