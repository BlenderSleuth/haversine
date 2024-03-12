use std::mem::MaybeUninit;

use enum_iterator::{all, cardinality, Sequence};
use metrics::repetition_tester::RepetitionTester;

const TRY_FOR_SECONDS: u32 = 10;

mod read_tests;
mod write_tests;

#[derive(Copy, Clone, PartialEq, Sequence)]
#[repr(u8)]
pub enum AllocType {
    None,
    Malloc,
}

impl AllocType {
    pub const NUM_ALLOC_TYPES: usize = cardinality::<AllocType>();

    pub fn to_str(&self) -> &'static str {
        match self {
            AllocType::None => "",
            AllocType::Malloc => "malloc + ",
        }
    }
}

pub struct TestParameters<'a> {
    pub alloc_type: AllocType,
    len: usize,
    dest: Option<Box<[MaybeUninit<u8>]>>,
    filename: &'a str,
}

impl<'a> TestParameters<'a> {
    pub fn new(alloc_type: AllocType, len: usize, filename: &'a str) -> Self {
        Self {
            alloc_type,
            len,
            dest: None,
            filename,
        }
    }

    pub fn handle_allocation(&mut self) -> &mut [MaybeUninit<u8>] {
        // Reallocate destination buffer for page fault testing
        if self.alloc_type == AllocType::Malloc || self.dest.is_none() {
            self.alloc();
        }
        self.dest.as_mut().unwrap()
    }

    fn alloc(&mut self) {
        unsafe {
            // Used instead of Box::new_uninit_slice in nightly
            let layout = std::alloc::Layout::array::<u8>(self.len).unwrap();
            let ptr = std::alloc::alloc(layout) as *mut MaybeUninit<u8>;
            let slice = std::slice::from_raw_parts_mut(ptr, self.len);
            self.dest = Some(Box::from_raw(slice));
        }
    }
}

struct TestFunction {
    pub name: &'static str,
    pub func: fn(&mut RepetitionTester, &mut TestParameters),
}

const WRITE_TESTS: &[TestFunction] = &[
    TestFunction { name: "write_to_all_bytes", func: write_tests::write_to_all_bytes },
    TestFunction { name: "write_to_all_bytes_inl_asm", func: write_tests::write_to_all_bytes_inl_asm },
];

#[allow(dead_code)]
pub fn bandwidth_test_loop(size: u64, cpu_freq: u64, filename: &str) {
    let mut params = TestParameters::new(AllocType::None, size as usize, &filename);

    let mut testers = [RepetitionTester::new(size, cpu_freq); WRITE_TESTS.len()];

    'test_loop: loop {
        for (test_func, tester) in WRITE_TESTS.iter().zip(testers.iter_mut()) {
            let dest = params.handle_allocation();

            print!("\n--- {} ---\n", test_func.name);

            tester.new_test_wave(dest.len() as u64, cpu_freq, TRY_FOR_SECONDS);
            (test_func.func)(tester, &mut params);

            if tester.has_error() {
                break 'test_loop;
            }
        }
    }
}


const PF_TESTS: &[TestFunction] = &[
    TestFunction { name: "write_to_all_bytes", func: write_tests::write_to_all_bytes },
    TestFunction { name: "read", func: read_tests::test_read },
    TestFunction { name: "fread", func: read_tests::test_fread },
    TestFunction { name: "ReadFile", func: read_tests::test_readfile },
];

#[allow(dead_code)]
pub fn pf_test_loop(size: u64, cpu_freq: u64, filename: &str) {
    let mut params = TestParameters::new(AllocType::None, size as usize, &filename);

    let mut testers = [[RepetitionTester::new(size, cpu_freq); AllocType::NUM_ALLOC_TYPES]; PF_TESTS.len()];

    'test_loop: loop {
        for (test_func, testers) in PF_TESTS.iter().zip(testers.iter_mut()) {
            for (tester, alloc_type) in testers.iter_mut().zip(all::<AllocType>()) {
                params.alloc_type = alloc_type;
                let dest = params.handle_allocation();

                print!("\n--- {}{} ---\n", alloc_type.to_str(), test_func.name);

                tester.new_test_wave(dest.len() as u64, cpu_freq, TRY_FOR_SECONDS);
                (test_func.func)(tester, &mut params);

                if tester.has_error() {
                    break 'test_loop;
                }
            }
        }
    }
}

pub use write_tests::asm_test_loop;