use std::fmt::Display;
use std::mem::MaybeUninit;

use enum_iterator::{all, cardinality, Sequence};
use windows_sys::Win32::Security::Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom};
use libc::rand;

use metrics::repetition_tester::RepetitionTester;
use metrics::test_block;
use crate::testing::{AllocType, TestParameters, TRY_FOR_SECONDS};

const MAX_OS_RANDOM_COUNT: u64 = u32::MAX as u64;

pub fn read_os_random_bytes(dest: &mut [u8]) -> bool {
    if (dest.len() as u64) < MAX_OS_RANDOM_COUNT {
        unsafe { BCryptGenRandom(std::ptr::null_mut(), dest.as_mut_ptr(), dest.len() as u32, BCRYPT_USE_SYSTEM_PREFERRED_RNG) != 0 }
    } else {
        false
    }
}

pub fn fill_with_random_bytes(dest: &mut [u8]) {
    let mut at_offset = 0;
    while at_offset < dest.len() {
        let read_count = (dest.len() - at_offset).min(MAX_OS_RANDOM_COUNT as usize);
        read_os_random_bytes(&mut dest[at_offset..at_offset+read_count]);
        at_offset += read_count;
    }
}

#[derive(Copy, Clone, PartialEq, Sequence)]
pub enum BranchPattern {
    NeverTaken,
    AlwaysTaken,
    Every2,
    Every3,
    Every4,
    CRTRandom,
    OSRandom,
}

impl Display for BranchPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchPattern::NeverTaken => write!(f, "Never Taken"),
            BranchPattern::AlwaysTaken => write!(f, "Always Taken"),
            BranchPattern::Every2 => write!(f, "Every 2"),
            BranchPattern::Every3 => write!(f, "Every 3"),
            BranchPattern::Every4 => write!(f, "Every 4"),
            BranchPattern::CRTRandom => write!(f, "CRTRandom"),
            BranchPattern::OSRandom => write!(f, "OSRandom"),
        }
    }
}

pub fn fill_with_branch_pattern(dest: &mut [u8], pattern: BranchPattern) {
    match pattern {
        BranchPattern::NeverTaken => {
            dest.fill(0);
        }
        BranchPattern::AlwaysTaken => {
            dest.fill(0xFF);
        }
        BranchPattern::Every2 => {
            for i in 0..dest.len() {
                dest[i] = (i & 1) as u8;
            }
        }
        BranchPattern::Every3 => {
            for i in 0..dest.len() {
                dest[i] = (i % 3) as u8;
            }
        }
        BranchPattern::Every4 => {
            for i in 0..dest.len() {
                dest[i] = (i % 4) as u8;
            }
        }
        BranchPattern::CRTRandom => {
            for i in 0..dest.len() {
                dest[i] = unsafe { rand() } as u8;
            }
        }
        BranchPattern::OSRandom => {
            fill_with_random_bytes(dest);
        }
    }
}

extern "C" {
    fn ConditionalNOP(count: u64, data: *mut u8);
}

#[allow(dead_code)]
pub fn branch_predictor_test_loop(size: u64, cpu_freq: u64, filename: &str) {
    let mut params = TestParameters::new(AllocType::None, size as usize, &filename);

    let mut testers = [RepetitionTester::new(size, cpu_freq); cardinality::<BranchPattern>()];

    'test_loop: loop {
        for (pattern, tester) in all::<BranchPattern>().zip(testers.iter_mut()) {
            let dest = params.handle_allocation();
            dest.fill(MaybeUninit::zeroed());
            let dest: &mut [u8] = unsafe { std::mem::transmute(dest) };
            
            let len = dest.len() as u64;

            print!("\n--- ConditionalNOP, {pattern} ---\n");
            fill_with_branch_pattern(dest, pattern);
            
            tester.new_test_wave(len, cpu_freq, TRY_FOR_SECONDS);
            while tester.testing() {
                unsafe {
                    test_block!(tester);
                    ConditionalNOP(len, dest.as_mut_ptr());
                }
                tester.count_bytes(len);
            }

            if tester.has_error() {
                break 'test_loop;
            }
        }
    }
}