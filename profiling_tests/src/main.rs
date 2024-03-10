mod read_tests;
mod nop_loop;

use std::env;
use enum_iterator::all;
use metrics::repetition_tester::RepetitionTester;
use metrics::timing::estimate_cpu_frequency;
use read_tests::{AllocType, ReadTestParameters};

const TEST_CPU_FREQ_MILLIS: u64 = 100;
const TRY_FOR_SECONDS: u32 = 10;

fn main() -> std::io::Result<()> {
    let cpu_freq = estimate_cpu_frequency(TEST_CPU_FREQ_MILLIS);

    // let memory = unsafe { std::alloc::alloc(std::alloc::Layout::new::<u8>()) };
    
    if env::args().len() == 2 {
        let filename = env::args().nth(1).unwrap();
        let file = std::fs::File::open(&filename)?;
        let size = file.metadata()?.len();
        
        if size > 0 {
            let mut read_params = ReadTestParameters {
                alloc_type: AllocType::None,
                dest: vec![0u8; size as usize],
                filename: &filename,
            };

            let mut testers = [[RepetitionTester::new(size, cpu_freq); AllocType::NUM_ALLOC_TYPES]; read_tests::TESTS.len()];
            
            'test_loop: loop {
                for (test_func, testers) in read_tests::TESTS.iter().zip(testers.iter_mut()) {
                    for (tester, alloc_type) in testers.iter_mut().zip(all::<AllocType>()) {
                        read_params.alloc_type = alloc_type;
                        
                        print!("\n--- {}{} ---\n", read_params.alloc_type.to_str(), test_func.name);
                        
                        tester.new_test_wave(read_params.dest.len() as u64, cpu_freq, TRY_FOR_SECONDS);
                        (test_func.func)(tester, &mut read_params);

                        if tester.has_error() {
                            break 'test_loop;
                        }
                    }
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
