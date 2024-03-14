use std::arch::asm;
use metrics::repetition_tester::{RepetitionTester, test_block};

use crate::testing::{AllocType, ASMFunction, TestParameters, TRY_FOR_SECONDS};

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

extern "C" {
    pub fn MOVAllBytesASM(count: u64, data: *mut u8);
    pub fn NOPAllBytesASM(count: u64, data: *mut u8);
    pub fn CMPAllBytesASM(count: u64, data: *mut u8);
    pub fn DECAllBytesASM(count: u64, data: *mut u8);
    
    pub fn NOP3x1AllBytes(count: u64, data: *mut u8);
    pub fn NOP1x3AllBytes(count: u64, data: *mut u8);
    pub fn NOP1x9AllBytes(count: u64, data: *mut u8);
}

#[allow(dead_code)]
pub const WRITE_ASM_TESTS: &[ASMFunction] = &[
    ASMFunction { name: "MOVAllBytesASM", func: MOVAllBytesASM },
    ASMFunction { name: "NOPAllBytesASM", func: NOPAllBytesASM },
    ASMFunction { name: "CMPAllBytesASM", func: CMPAllBytesASM },
    ASMFunction { name: "DECAllBytesASM", func: DECAllBytesASM },
    ASMFunction { name: "NOP3x1AllBytes", func: NOP3x1AllBytes },
    ASMFunction { name: "NOP1x3AllBytes", func: NOP1x3AllBytes },
    ASMFunction { name: "NOP1x9AllBytes", func: NOP1x9AllBytes },
];

extern "C" {
    pub fn Write_x1(count: u64, data: *mut u8);
    pub fn Write_x2(count: u64, data: *mut u8);
    pub fn Write_x3(count: u64, data: *mut u8);
    pub fn Write_x4(count: u64, data: *mut u8);
}

#[allow(dead_code)]
pub const WRITE_PORT_TESTS: &[ASMFunction] = &[
    ASMFunction { name: "Write_x1", func: Write_x1 },
    ASMFunction { name: "Write_x2", func: Write_x2 },
    ASMFunction { name: "Write_x3", func: Write_x3 },
    ASMFunction { name: "Write_x4", func: Write_x4 },
];

#[allow(dead_code)]
pub fn asm_test_loop(size: u64, cpu_freq: u64, filename: &str, tests: &[ASMFunction]) {
    let mut params = TestParameters::new(AllocType::None, size as usize, &filename);
    let mut testers = vec![RepetitionTester::new(size, cpu_freq); tests.len()];

    'test_loop: loop {
        for (test_func, tester) in tests.iter().zip(testers.iter_mut()) {
            let dest = params.handle_allocation();
            let len = dest.len() as u64;

            print!("\n--- {} ---\n", test_func.name);

            tester.new_test_wave(len, cpu_freq, TRY_FOR_SECONDS);
            
            while tester.testing() {
                unsafe {
                    test_block!(tester);
                    (test_func.func)(len, dest.as_mut_ptr() as *mut u8);
                }
                tester.count_bytes(len);
            }
            
            if tester.has_error() {
                break 'test_loop;
            }
        }
    }
}