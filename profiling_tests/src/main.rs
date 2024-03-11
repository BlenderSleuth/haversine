mod read_tests;
mod write_tests;

use std::env;
use enum_iterator::all;
use metrics::repetition_tester::RepetitionTester;
use metrics::timing::estimate_cpu_frequency;
use read_tests::{AllocType, ReadTestParameters};
use write_tests::WriteTestParameters;

const TEST_CPU_FREQ_MILLIS: u64 = 100;
const TRY_FOR_SECONDS: u32 = 10;

fn read_tests(size: u64, filename: &str, cpu_freq: u64) {
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
}

fn write_tests(size: usize, cpu_freq: u64) {
    
    let mut write_params = WriteTestParameters::new(AllocType::None, size, 256);

    let mut testers = [[RepetitionTester::new(size as u64, cpu_freq); AllocType::NUM_ALLOC_TYPES]; write_tests::TESTS.len()];

    'test_loop: loop {
        for (test_func, testers) in write_tests::TESTS.iter().zip(testers.iter_mut()) {
            for (tester, alloc_type) in testers.iter_mut().zip(all::<AllocType>()) {
                write_params.alloc_type = alloc_type;
                write_params.realloc();

                print!("\n--- {}{} ---\n", write_params.alloc_type.to_str(), test_func.name);

                tester.new_test_wave(write_params.get_dest().len() as u64, cpu_freq, TRY_FOR_SECONDS);
                (test_func.func)(tester, &mut write_params);

                if tester.has_error() {
                    break 'test_loop;
                }
            }
        };
    }
}

fn main() -> std::io::Result<()> {
    let cpu_freq = estimate_cpu_frequency(TEST_CPU_FREQ_MILLIS);
    
    if false {
        if env::args().len() == 2 {
            let filename = env::args().nth(1).unwrap();
            let file = std::fs::File::open(&filename)?;
            let size = file.metadata()?.len();

            read_tests(size, &filename, cpu_freq);
        } else {
            eprintln!("Usage: {} [existing filename]", env::current_exe()?.display());
        }
    } else {
        write_tests(1024 * 1024 * 1024, cpu_freq);
    }
    
    Ok(())
}
