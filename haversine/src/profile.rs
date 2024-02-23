use std::sync::{Mutex, OnceLock};
use crate::timer::read_cpu_timer;

#[macro_export]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);

        // Find and cut the rest of the path
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        }
    }};
}
pub use function_name;

#[macro_export]
macro_rules! time_block {
    ($name:expr, $idx:literal) => {
        let _time_block = $crate::profile::TimeBlock::new($name, $idx);
    };
}
pub use time_block;

#[macro_export]
macro_rules! time_function {
    ($idx:literal) => {
        $crate::time_block!($crate::function_name!(), $idx);
    };
}
pub use time_function;

struct TimeRecord {
    label: &'static str,
    elapsed: u64,
    hit_count: u64,
}

impl TimeRecord {
    fn new(label: &'static str) -> TimeRecord {
        TimeRecord { label, elapsed: 0, hit_count: 0 }
    }
}

const NUM_TIME_RECORDS: usize = 256;
fn get_time_records() -> &'static Mutex<[Option<TimeRecord>; NUM_TIME_RECORDS]> {
    static TIME_BLOCK_RECORDS: OnceLock<Mutex<[Option<TimeRecord>; NUM_TIME_RECORDS]>> = OnceLock::new();
    const ARRAY_REPEAT_VALUE: Option<TimeRecord> = None;
    TIME_BLOCK_RECORDS.get_or_init(|| Mutex::new([ARRAY_REPEAT_VALUE; NUM_TIME_RECORDS]))
}

pub fn print_time_records(total: u64) {
    let total_rcp = 100.0 / total as f64;
    let time_records = get_time_records().lock().unwrap();
    for record in time_records.iter().flatten() {
        println!("  {}[{}]: {} ({:.2}%)", record.label, record.hit_count, record.elapsed, record.elapsed as f64 * total_rcp)
    }
}

pub struct TimeBlock {
    start: u64,
    record: usize,
}

impl TimeBlock {
    pub fn new(label: &'static str, record: usize) -> Self {
        let mut time_records = get_time_records().lock().unwrap();
        let time_record = time_records[record].get_or_insert(TimeRecord::new(label));
        time_record.hit_count += 1;
        TimeBlock { start: read_cpu_timer(), record }
    }
}

impl Drop for TimeBlock {
    fn drop(&mut self) {
        let end = read_cpu_timer();
        let elapsed = end - self.start;
        let mut time_record = get_time_records().lock().unwrap();
        time_record[self.record].as_mut().unwrap().elapsed += elapsed;
    }
}