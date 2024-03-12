use std::arch::asm;
use metrics::repetition_tester::{RepetitionTester, test_block};

use crate::testing::TestParameters;

pub fn write_to_all_bytes(tester: &mut RepetitionTester, params: &mut TestParameters) {
    while tester.testing() {
        let dest = params.handle_allocation();
        
        let num_bytes = dest.len();
        let mut i: usize = 0;
        {
            test_block!(tester);
            while i < num_bytes {
                unsafe { *dest[i].as_mut_ptr() = 0 }
                i += 1;
            }
        }
        tester.count_bytes(num_bytes as u64);
    }
}

pub fn write_to_all_bytes_inl_asm(tester: &mut RepetitionTester, params: &mut TestParameters) {
    while tester.testing() {
        let dest = params.handle_allocation();

        let num_bytes = dest.len();
        {
            test_block!(tester);
            unsafe {
                asm!(
                    r#"
                    xor {i}, {i}
                2:
                    mov [{dest} + {i}], {i:l}
                    inc {i}
                    cmp {i}, {count}
                    jb 2b"#,
                    i = in(reg) 0usize,
                    dest = in(reg) dest.as_mut_ptr(),
                    count = in(reg) num_bytes,
                    options(nostack),
                );
            }
        }
        tester.count_bytes(num_bytes as u64);
    }
}

#[allow(dead_code)]
extern "C" {
    pub fn MOVAllBytesASM(count: u64, data: *mut u8);
    pub fn NOPAllBytesASM(count: u64);
    pub fn CMPAllBytesASM(count: u64);
    pub fn DECAllBytesASM(count: u64);
}

pub fn mov_all_bytes(tester: &mut RepetitionTester, params: &mut TestParameters) {
    while tester.testing() {
        let dest = params.handle_allocation();

        let num_bytes = dest.len() as u64;
        {
            test_block!(tester);
            unsafe { MOVAllBytesASM(num_bytes, dest.as_mut_ptr() as *mut u8); }
        }
        tester.count_bytes(num_bytes);
    }
}

pub fn nop_all_bytes(tester: &mut RepetitionTester, params: &mut TestParameters) {
    while tester.testing() {
        let dest = params.handle_allocation();

        let num_bytes = dest.len() as u64;
        {
            test_block!(tester);
            unsafe { NOPAllBytesASM(num_bytes); }
        }
        tester.count_bytes(num_bytes);
    }
}

pub fn cmp_all_bytes(tester: &mut RepetitionTester, params: &mut TestParameters) {
    while tester.testing() {
        let dest = params.handle_allocation();

        let num_bytes = dest.len() as u64;
        {
            test_block!(tester);
            unsafe { CMPAllBytesASM(num_bytes); }
        }
        tester.count_bytes(num_bytes);
    }
}

pub fn dec_all_bytes(tester: &mut RepetitionTester, params: &mut TestParameters) {
    while tester.testing() {
        let dest = params.handle_allocation();

        let num_bytes = dest.len() as u64;
        {
            test_block!(tester);
            unsafe { DECAllBytesASM(num_bytes); }
        }
        tester.count_bytes(num_bytes);
    }
}