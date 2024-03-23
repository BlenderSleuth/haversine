use std::mem::MaybeUninit;
use metrics::repetition_tester::RepetitionTester;
use metrics::test_block;
use crate::testing::{AllocType, TestParameters, TRY_FOR_SECONDS};

extern "C" {
    #[cfg(target_feature = "avx")]
    pub fn CacheTest(count: u64, data: *mut u8, mask: u64);
    #[cfg(target_feature = "avx")]
    pub fn CacheTest2(data: *mut u8, inner_count: u64, outer_count: u64);
}

const MIN_CACHE_IDX: usize = 10; // 1GB
const MAX_CACHE_IDX: usize = 30; // 1GB

fn print_csv(testers: &[RepetitionTester], cpu_freq: u64, size: u64) {
    println!();
    println!("Region Size,GB/s");
    for (cache_size_idx, tester) in (MIN_CACHE_IDX..=MAX_CACHE_IDX).zip(testers.iter()) {
        let cache_size = 1u64 << cache_size_idx;
        let seconds = tester.get_result().min.time as f64 / cpu_freq as f64;
        const GB: f64 = 1024.0 * 1024.0 * 1024.0;
        let bandwidth = size as f64 / GB / seconds;
        println!("{cache_size},{bandwidth}");
    }
}

#[allow(dead_code)]
#[cfg(target_feature = "avx")]
pub fn cache_test_loop(size: u64, cpu_freq: u64) {
    let mut params = TestParameters::new(AllocType::None, size as usize, "");
    let dest = params.handle_allocation();
    for (i, byte) in dest.iter_mut().enumerate() {
        *byte = MaybeUninit::new(i as u8);
    }
    
    let mut testers = [RepetitionTester::new(size, cpu_freq); MAX_CACHE_IDX-MIN_CACHE_IDX+1];
    
    for (cache_size_idx, tester) in (MIN_CACHE_IDX..=MAX_CACHE_IDX).zip(testers.iter_mut()) {
        let len = dest.len() as u64;

        let cache_size = 1u64 << cache_size_idx;
        print!("\n--- Cache Test: {}KB ---\n", cache_size/1024);

        tester.new_test_wave(len, cpu_freq, TRY_FOR_SECONDS);
        
        let mask = cache_size.next_power_of_two() - 1;

        while tester.testing() {
            unsafe {
                test_block!(tester);
                CacheTest(len, dest.as_mut_ptr() as *mut u8, mask);
            }
            tester.count_bytes(len);
        }

        if tester.has_error() {
            break;
        }
    }

    print_csv(&testers, cpu_freq, size);
}

pub struct CacheSizeTest{
    // Number of 256byte blocks to move
    pub inner_count: u64,
    pub outer_count: u64,
    pub real_size: u64
}

impl CacheSizeTest {
    pub fn new(full_size: u64, mut cache_size: u64) -> Self {
        
        let inner_count = cache_size / 256;
        cache_size = inner_count * 256;
        
        let outer_count = full_size / cache_size;
        let real_size = cache_size * outer_count;
        
        Self {
            inner_count,
            outer_count,
            real_size
        }
    }
    
    pub fn cache_size(&self) -> u64 {
        self.inner_count * 256
    }
}

// Allows testing of cache sizes of arbitrary multiples of 256 bytes
#[allow(dead_code)]
#[cfg(target_feature = "avx")]
pub fn cache_test_loop2(size: u64, cpu_freq: u64, tests: &[CacheSizeTest]) {
    let mut params = TestParameters::new(AllocType::None, size as usize, "");
    let dest = params.handle_allocation();
    for (i, byte) in dest.iter_mut().enumerate() {
        *byte = MaybeUninit::new(i as u8);
    }
    
    let mut testers: Vec<_> = tests.iter().map(|test| RepetitionTester::new(test.real_size, cpu_freq)).collect();

    for (size_test, tester) in tests.iter().zip(testers.iter_mut()) {
        print!("\n--- Cache Test: {}KB ---\n", size_test.cache_size() / 1024);

        tester.new_test_wave(size_test.real_size, cpu_freq, TRY_FOR_SECONDS);
        
        while tester.testing() {
            unsafe {
                test_block!(tester);
                CacheTest2(dest.as_mut_ptr() as *mut u8, size_test.inner_count, size_test.outer_count);
            }
            tester.count_bytes(size_test.real_size);
        }

        if tester.has_error() {
            break;
        }
    }

    print_csv(&testers, cpu_freq, size);
}