use std::env;
use metrics::timing::estimate_cpu_frequency;
use crate::testing::cache_tests::cache_test_loop;

mod testing;

#[allow(unused_imports)]
use crate::testing::write_tests::{WRITE_ASM_TESTS, WRITE_PORT_TESTS};
#[allow(unused_imports)]
use crate::testing::simd_tests::READ_WIDTH_TESTS;

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

        let size: u64 = 1024 * 1024 * 1024;
        cache_test_loop(size, cpu_freq);
        
        //eprintln!("Usage: {} [existing filename/data size]", env::current_exe()?.display());
    }

    Ok(())
}
