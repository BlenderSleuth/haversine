use std::mem::MaybeUninit;
use metrics::repetition_tester::RepetitionTester;
use metrics::test_block;
use crate::testing::{AllocType, TestParameters, TRY_FOR_SECONDS};

extern "C" {
    #[cfg(target_feature = "avx")]
    pub fn CacheTest(count: u64, data: *mut u8, mask: u64);
}

const MIN_CACHE_IDX: usize = 10; // 1GB
const MAX_CACHE_IDX: usize = 30; // 1GB

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