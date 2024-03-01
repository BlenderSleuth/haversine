mod read_tests;

use std::env;
use metrics::repetition_tester::RepetitionTester;
use metrics::timing::estimate_cpu_frequency;

const TEST_CPU_FREQ_MILLIS: u64 = 100;
const TRY_FOR_SECONDS: u32 = 10;

fn main() -> std::io::Result<()> {
    let cpu_freq = estimate_cpu_frequency(TEST_CPU_FREQ_MILLIS);
    
    if env::args().len() == 2 {
        let filename = env::args().nth(1).unwrap();
        let file = std::fs::File::open(&filename)?;
        let size = file.metadata()?.len();
        
        if size > 0 {
            let mut buffer = vec![0u8; size as usize];

            let mut read_params = read_tests::ReadTestParameters {
                dest: &mut buffer,
                filename: &filename,
            };

            let mut testers = [RepetitionTester::new(size, cpu_freq); read_tests::TESTS.len()];
            
            loop {
                for (test_func, tester) in read_tests::TESTS.iter().zip(testers.iter_mut()) {
                    print!("\n--- {} ---\n", test_func.name);
                    tester.new_test_wave(read_params.dest.len() as u64, cpu_freq, TRY_FOR_SECONDS);
                    (test_func.func)(tester, &mut read_params);
                };
            }
        } else {
            eprintln!("ERROR: Test data size must be non-zero.");
        }
    } else {
        eprintln!("Usage: {} [existing filename]", env::current_exe()?.display());
    }


    Ok(())
}
