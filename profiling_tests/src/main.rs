use std::env;
use metrics::timing::estimate_cpu_frequency;

mod testing;

#[allow(unused_imports)]
use crate::testing::write_tests::{WRITE_ASM_TESTS, WRITE_PORT_TESTS};
#[allow(unused_imports)]
use crate::testing::simd_tests::READ_WIDTH_TESTS;
#[allow(unused_imports)]
use crate::testing::cache_tests::{cache_test_loop, cache_test_loop2, CacheSizeTest};

const TEST_CPU_FREQ_MILLIS: u64 = 100;

fn main() -> std::io::Result<()> {
    let cpu_freq = estimate_cpu_frequency(TEST_CPU_FREQ_MILLIS);
    
    if env::args().len() == 2 {
        let arg = env::args().nth(1).unwrap();
        
        let (size, filename) =
        if let Ok(parsed_size) = arg.parse::<u64>() {
            (parsed_size, String::new())
        } else {
            let file = std::fs::File::open(&arg)?;
            (file.metadata()?.len(), arg)
        };
        
        if size > 0 {
            //testing::pf_test_loop(size, cpu_freq, &filename);
            //testing::bandwidth_test_loop(size, cpu_freq, &filename);
            //testing::asm_test_loop(size, cpu_freq, &filename, WRITE_ASM_TESTS);
            //testing::branch_predictor_test_loop(size, cpu_freq, &filename);
            //testing::asm_test_loop(size, cpu_freq, &filename, WRITE_PORT_TESTS);
            testing::asm_test_loop(size, cpu_freq, &filename, READ_WIDTH_TESTS);
        } else {
            eprintln!("ERROR: Test data size must be non-zero.");
        }
    } else {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;
        const FULL_BUFFER_SIZE: u64 = 1*GB;
        //cache_test_loop(FULL_BUFFER_SIZE, cpu_freq);
        
        const CACHE_SIZES: &[u64] = &[4*KB, 16*KB, 24*KB, 28*KB, 
            30*KB, 31*KB, 32*KB, 33*KB, 34*KB, 40*KB, 64*KB, 128*KB, 
            196*KB, 232*KB, 250*KB, 254*KB, 255*KB, 256*KB, 257*KB, 258*KB, 264*KB, 512*KB, 
            1*MB, 2*MB, 4*MB, 6*MB, 7*MB, 8*MB, 9*MB, 10*MB, 16*MB, 64*MB, 
            256*MB, 512*MB, 1*GB];//, 8*GB, 32*GB, 64*GB];
        
        let tests = CACHE_SIZES.iter().map(|size| CacheSizeTest::new(FULL_BUFFER_SIZE, *size)).collect::<Vec<_>>();
        
        cache_test_loop2(FULL_BUFFER_SIZE, cpu_freq, &tests);
        
        //eprintln!("Usage: {} [existing filename/data size]", env::current_exe()?.display());
    }

    Ok(())
}
