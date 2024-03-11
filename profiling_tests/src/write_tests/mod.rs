use metrics::repetition_tester::RepetitionTester;
use crate::read_tests::{AllocType}; // TODO: put AllocType in tests module

pub mod nop_loop;
pub mod write_to_all_bytes;

pub struct WriteTestParameters {
    pub alloc_type: AllocType,
    len: usize,
    align: usize,
    dest: *mut u8,
}

impl WriteTestParameters {
    pub fn new(alloc_type: AllocType, len: usize, align: usize) -> Self {
        Self {
            alloc_type,
            len,
            align,
            dest: std::ptr::null_mut(),
        }
    }

    pub fn get_dest(&self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.dest, self.len) }
    }

    pub fn realloc(&mut self) {
        if !self.dest.is_null() {
            unsafe { self.dealloc(); }
        }
        unsafe { self.alloc(); }
    }
    
    unsafe fn handle_allocation(&mut self) {
        // Reallocate destination buffer for page fault testing
        if self.dest.is_null() {
            self.alloc();
        }
    }

    unsafe fn handle_deallocation(&mut self) {
        if self.alloc_type == AllocType::Malloc {
            self.dealloc();
        }
    }

    unsafe fn alloc(&mut self) {
        let layout = std::alloc::Layout::from_size_align(self.len, self.align.next_power_of_two()).unwrap();
        self.dest = std::alloc::alloc(layout);
    }

    unsafe fn dealloc(&mut self) {
        let layout = std::alloc::Layout::from_size_align(self.len, self.align.next_power_of_two()).unwrap();
        std::alloc::dealloc(self.dest, layout);
        self.dest = std::ptr::null_mut();
    }
}

impl Drop for WriteTestParameters {
    fn drop(&mut self) {
        if self.alloc_type != AllocType::Malloc {
            unsafe { self.dealloc(); }
        }
    }
}

struct WriteTestAllocBlock<'a> {
    params: &'a mut WriteTestParameters
}

impl<'a> WriteTestAllocBlock<'a> {
    pub fn new(params: &'a mut WriteTestParameters) -> Self {
        unsafe { params.handle_allocation() }
        Self { params }
    }
    
    pub fn get_dest(&self) -> &mut [u8] {
        self.params.get_dest()
    }
}

impl Drop for WriteTestAllocBlock<'_> {
    fn drop(&mut self) {
        unsafe { self.params.handle_deallocation(); }
    }
}

type TestFunction = fn(&mut RepetitionTester, &mut WriteTestParameters);

pub struct WriteTestFunction {
    pub name: &'static str,
    pub func: TestFunction,
}

pub const TESTS: &[WriteTestFunction] = &[
    WriteTestFunction { name: "write_to_all_bytes", func: write_to_all_bytes::write_to_all_bytes },
];