use std::env;
use metrics::timing::estimate_cpu_frequency;

mod testing;

use crate::testing::write_tests::{WRITE_ASM_TESTS, WRITE_PORT_TESTS};
use crate::testing::simd_tests::READ_WIDTH_TESTS;

const TEST_CPU_FREQ_MILLIS: u64 = 100;

fn main() -> std::io::Result<()> {
    let cpu_freq = estimate_cpu_frequency(TEST_CPU_FREQ_MILLIS);
    
    if env::args().len() == 2 {
        let filename = env::args().nth(1).unwrap();
        let file = std::fs::File::open(&filename)?;
        let size = file.metadata()?.len();

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
        eprintln!("Usage: {} [existing filename]", env::current_exe()?.display());
    }

    Ok(())
}
