use std::num::NonZeroU64;
use crate::timing::{print_time, read_cpu_timer};

#[derive(Clone, Copy, PartialEq)]
enum TestMode {
    Testing,
    Completed,
    Error,
}

#[derive(Clone, Copy)]
pub struct RepetitionTestResult {
    num_tests: u64,
    total_time: u64,
    max_time: u64,
    min_time: u64,
}

impl RepetitionTestResult {
    pub fn print(&self, cpu_freq: u64, num_bytes: Option<NonZeroU64>) {
        print_time("Min", self.min_time, cpu_freq, num_bytes, false);
        print_time("Max", self.max_time, cpu_freq, num_bytes, false);

        if self.num_tests > 0 {
            let avg_time = self.total_time as f64 / self.num_tests as f64;
            print_time("Avg", avg_time as u64, cpu_freq, num_bytes, false);
        }
    }
}

#[derive(Clone, Copy)]
pub struct RepetitionTester {
    target_accumulated_bytes: u64,
    cpu_timer_freq: u64,
    try_for_time: u64,
    test_start_time: u64,
    mode: TestMode,
    pub print_new_minimums: bool,
    block_count: u32,
    accumulated_time: u64,
    accumulated_bytes: u64,
    result: RepetitionTestResult,
}

impl RepetitionTester {
    pub fn new(target_accumulated_bytes: u64, cpu_timer_freq: u64) -> Self {
        Self {
            target_accumulated_bytes,
            cpu_timer_freq,
            try_for_time: 0,
            test_start_time: 0,
            mode: TestMode::Testing,
            print_new_minimums: true,
            block_count: 0,
            accumulated_time: 0,
            accumulated_bytes: 0,
            result: RepetitionTestResult {
                num_tests: 0,
                total_time: 0,
                max_time: 0,
                min_time: u64::MAX,
            },
        }
    }
    
    pub fn error(&mut self, message: &str) {
        self.mode = TestMode::Error;
        eprintln!("ERROR: {message}");
    }
    
    pub fn new_test_wave(&mut self, target_accumulated_bytes: u64, cpu_timer_freq: u64, seconds_to_try: u32) {
        if self.mode == TestMode::Completed {
            self.mode = TestMode::Testing;
            
            if self.target_accumulated_bytes != target_accumulated_bytes
            {
                self.error("Test byte count changed");
            }

            if self.cpu_timer_freq != cpu_timer_freq
            {
                self.error("CPU frequency changed");
            }
        }
        
        self.try_for_time = seconds_to_try as u64 * cpu_timer_freq;
        self.test_start_time = read_cpu_timer();
    }
    
    pub fn count_bytes(&mut self, num_bytes: u64) {
        self.accumulated_bytes += num_bytes;
    }
    
    pub fn testing(&mut self) -> bool {
        if self.mode == TestMode::Testing {
            let current_time = read_cpu_timer();
            
            if self.block_count > 0 {
                if self.target_accumulated_bytes != self.accumulated_bytes {
                    self.error("Processed byte count mismatch");
                } else {
                    let result = &mut self.result;
                    let elapsed_time = self.accumulated_time;
                    result.num_tests += 1;
                    result.total_time += elapsed_time;
                    result.max_time = result.max_time.max(elapsed_time);
                    
                    if result.min_time > elapsed_time {
                        result.min_time = elapsed_time;

                        // Whenever we get a new minimum time, we reset the clock to the full trial time
                        self.test_start_time = current_time;
                        if self.print_new_minimums {
                            print_time("Min", result.min_time, self.cpu_timer_freq, NonZeroU64::new(self.accumulated_bytes), true);
                        }
                    }
                    
                    self.accumulated_bytes = 0;
                    self.accumulated_time = 0;
                    self.block_count = 0;
                }
            }
            
            if current_time - self.test_start_time > self.try_for_time {
                self.mode = TestMode::Completed;

                print!("                                                          \r");
                self.result.print(self.cpu_timer_freq, NonZeroU64::new(self.target_accumulated_bytes));
            }
        }
        
        self.mode == TestMode::Testing
    }
    
    pub fn get_result(&self) -> &RepetitionTestResult {
        &self.result
    }
}

pub struct RepetitionTesterBlock<'a> {
    tester: &'a mut RepetitionTester,
}

impl<'a> RepetitionTesterBlock<'a> {
    pub fn new(tester: &'a mut RepetitionTester) -> Self {
        tester.accumulated_time = tester.accumulated_time.wrapping_sub(read_cpu_timer());
        tester.block_count += 1;
        Self { tester }
    }
}

impl Drop for RepetitionTesterBlock<'_> {
    fn drop(&mut self) {
        self.tester.accumulated_time = self.tester.accumulated_time.wrapping_add(read_cpu_timer());
    }
}

#[macro_export]
macro_rules! test_block {
    ($tester:ident) => {
        let _block = $crate::repetition_tester::RepetitionTesterBlock::new($tester);
    };
}
pub use test_block;