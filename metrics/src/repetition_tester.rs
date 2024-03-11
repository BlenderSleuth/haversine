use std::io;
use std::io::Write;
use std::num::NonZeroU64;
use std::ops::{Add, AddAssign, Div};

use crate::timing::{cpu_time_to_seconds, read_cpu_timer};
use crate::memory::read_os_page_fault_count;

#[derive(Clone, Copy, PartialEq)]
enum TestMode {
    Testing,
    Completed,
    Error,
}

#[derive(Clone, Copy)]
pub struct RepetitionTestValue {
    pub num_tests: u64,
    pub time: u64,
    pub bytes: u64,
    pub page_faults: u64,
}

impl RepetitionTestValue {
    pub fn zero() -> Self {
        Self {
            num_tests: 0,
            time: 0,
            bytes: 0,
            page_faults: 0,
        }
    }
    pub fn max() -> Self {
        Self {
            num_tests: 0,
            time: u64::MAX,
            bytes: u64::MAX,
            page_faults: u64::MAX,
        }
    }
    
    pub fn print(&self, label: &str, cpu_freq: u64) {
        let avg = *self / self.num_tests.max(1);

        // Print cpu time
        print!("{label}: {}", avg.time);
        
        // Print real time and bandwidth
        if let Some(freq) = NonZeroU64::new(cpu_freq) {
            let time_seconds = cpu_time_to_seconds(avg.time, freq);
            let time_ms = time_seconds * 1000.0;
            print!(" ({time_ms:.6}ms)");
            if avg.bytes > 0 {
                const GIGABYTE: f64 = 1024.0 * 1024.0 * 1024.0;
                let bandwidth = avg.bytes as f64 / (time_seconds * GIGABYTE);
                print!(" {bandwidth:.6} GB/s");
            }
        }
        
        // Print page faults
        if avg.page_faults > 0 {
            print!(" PF: {:.4} ({:.4}KB/fault)", avg.page_faults, avg.bytes as f64 / avg.page_faults as f64 * 1024.0);
        }
        
        io::stdout().flush().unwrap();
    }
}

impl Add<RepetitionTestValue> for RepetitionTestValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            num_tests: self.num_tests + other.num_tests,
            time: self.time + other.time,
            bytes: self.bytes + other.bytes,
            page_faults: self.page_faults + other.page_faults,
        }
    }
}

impl AddAssign<RepetitionTestValue> for RepetitionTestValue {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Div<u64> for RepetitionTestValue {
    type Output = Self;

    fn div(self, other: u64) -> Self {
        Self {
            num_tests: self.num_tests / other,
            time: self.time / other,
            bytes: self.bytes / other,
            page_faults: self.page_faults / other,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RepetitionTestResult {
    total: RepetitionTestValue,
    max: RepetitionTestValue,
    min: RepetitionTestValue,
}

impl RepetitionTestResult {
    pub fn print(&self, cpu_freq: u64) {
        self.min.print("Min", cpu_freq);
        println!();
        self.max.print("Max", cpu_freq);
        println!();
        self.total.print("Avg", cpu_freq);
        println!();
    }
}

impl Default for RepetitionTestResult {
    fn default() -> Self {
        Self {
            total: RepetitionTestValue::zero(),
            max: RepetitionTestValue::zero(),
            min: RepetitionTestValue::max(),
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
    accumulated: RepetitionTestValue,
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
            accumulated: RepetitionTestValue::zero(),
            result: RepetitionTestResult::default(),
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
        self.accumulated.bytes += num_bytes;
    }
    
    pub fn testing(&mut self) -> bool {
        if self.mode == TestMode::Testing {
            let current_time = read_cpu_timer();
            
            if self.block_count > 0 {
                if self.target_accumulated_bytes != self.accumulated.bytes {
                    self.error("Processed byte count mismatch");
                } else {
                    let result = &mut self.result;
                    
                    // Add this test to the total
                    self.accumulated.num_tests = 1;
                    result.total += self.accumulated;

                    let elapsed_time = self.accumulated.time;
                    
                    if result.max.time < elapsed_time {
                        result.max = self.accumulated;
                    }
                    
                    if result.min.time > elapsed_time {
                        result.min = self.accumulated;

                        // Whenever we get a new minimum time, we reset the clock to the full trial time
                        self.test_start_time = current_time;
                        if self.print_new_minimums {
                            result.min.print("Min", self.cpu_timer_freq);
                            print!("                                   \r");
                        }
                    }

                    self.block_count = 0;
                    self.accumulated = RepetitionTestValue::zero();
                }
            }
            
            if current_time - self.test_start_time > self.try_for_time {
                self.mode = TestMode::Completed;

                print!("                                                          \r");
                self.result.print(self.cpu_timer_freq);
            }
        }
        
        self.mode == TestMode::Testing
    }
    
    pub fn get_result(&self) -> &RepetitionTestResult {
        &self.result
    }
    
    pub fn has_error(&self) -> bool {
        self.mode == TestMode::Error
    }
}

pub struct RepetitionTesterBlock<'a> {
    tester: &'a mut RepetitionTester,
}

impl<'a> RepetitionTesterBlock<'a> {
    pub fn new(tester: &'a mut RepetitionTester) -> Self {
        tester.accumulated.time = tester.accumulated.time.wrapping_sub(read_cpu_timer());
        tester.accumulated.page_faults = tester.accumulated.page_faults.wrapping_sub(read_os_page_fault_count());
        tester.block_count += 1;
        Self { tester }
    }
}

impl Drop for RepetitionTesterBlock<'_> {
    fn drop(&mut self) {
        self.tester.accumulated.time = self.tester.accumulated.time.wrapping_add(read_cpu_timer());
        self.tester.accumulated.page_faults = self.tester.accumulated.page_faults.wrapping_add(read_os_page_fault_count());
    }
}

#[macro_export]
macro_rules! test_block {
    ($tester:ident) => {
        let _block = $crate::repetition_tester::RepetitionTesterBlock::new($tester);
    };
}
pub use test_block;
