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
macro_rules! time_bandwidth {
    ($name:expr, $idx:expr, $bytes:expr) => {
        let _time_block = $crate::profile::TimeBlock::new($name, $idx as usize, $bytes as u64);
    };
}
pub use time_bandwidth;

#[macro_export]
macro_rules! time_block {
    ($name:expr, $idx:expr) => {
        $crate::time_bandwidth!($name, $idx, 0);
    };
}
pub use time_block;

#[macro_export]
macro_rules! time_function {
    ($idx:expr) => {
        $crate::time_block!($crate::function_name!(), $idx);
    };
}
pub use time_function;

struct TimeRecord {
    label: &'static str,
    elapsed_exclusive: u64, // Does not include children
    elapsed_inclusive: u64, // Does include children
    byte_count: u64,
    hit_count: u64,
}

impl TimeRecord {
    fn new(label: &'static str) -> TimeRecord {
        TimeRecord { label, elapsed_exclusive: 0, elapsed_inclusive: 0, byte_count: 0, hit_count: 0 }
    }
}

// Completely un-threadsafe, but it's fine for this use case. Don't need the locks overhead.
static mut PARENT_TIME_RECORD: Option<usize> = None;

const NUM_TIME_RECORDS: usize = 256;
fn get_time_records() -> &'static mut [Option<TimeRecord>; NUM_TIME_RECORDS] {
    const ARRAY_REPEAT_VALUE: Option<TimeRecord> = None;
    
    static mut TIME_BLOCK_RECORDS: [Option<TimeRecord>; NUM_TIME_RECORDS] = [ARRAY_REPEAT_VALUE; NUM_TIME_RECORDS];
    unsafe {
        &mut TIME_BLOCK_RECORDS
    }
}

pub fn print_time_records(total: u64, timer_freq: u64) {
    let total_rcp = 100.0 / total as f64;
    let time_records = get_time_records();
    for record in time_records.iter().flatten() {
        print!("  {}[{}]: {} ({:.2}%", record.label, record.hit_count, record.elapsed_exclusive, record.elapsed_exclusive as f64 * total_rcp);
        
        if record.elapsed_exclusive != record.elapsed_inclusive {
            print!(", {:.2}% w/children", record.elapsed_inclusive as f64 * total_rcp);
        }
        
        if record.byte_count != 0 {
            const MEGABYTE: f64 = 1024.0 * 1024.0;
            const GIGABYTE: f64 = 1024.0 * MEGABYTE;
            
            let seconds = record.elapsed_inclusive as f64 / timer_freq as f64;
            let bytes_per_second = record.byte_count as f64 / seconds;
            let megabytes = record.byte_count as f64 / MEGABYTE;
            let gigabytes_per_second = bytes_per_second / GIGABYTE;
            
            print!(" {megabytes:.3}MB at {gigabytes_per_second:.2}GB/s");
        }
        
        println!(")");
    }
}

pub struct TimeBlock {
    start: u64,
    old_elapsed_inclusive: u64,
    record: usize,
    parent: Option<usize>,
}

impl TimeBlock {
    pub fn new(label: &'static str, record: usize, byte_count: u64) -> Self {
        let time_records = get_time_records();
        let time_record = time_records[record].get_or_insert(TimeRecord::new(label));
        time_record.byte_count += byte_count;
        
        let parent = unsafe { PARENT_TIME_RECORD };
        unsafe { PARENT_TIME_RECORD = Some(record) }
        let old_elapsed_inclusive = time_records[record].as_ref().unwrap().elapsed_inclusive;
        TimeBlock { start: read_cpu_timer(), old_elapsed_inclusive, record, parent }
    }
}

impl Drop for TimeBlock {
    fn drop(&mut self) {
        let elapsed = read_cpu_timer() - self.start;
        unsafe { PARENT_TIME_RECORD = self.parent }
        
        let time_records = get_time_records();
        if let Some(parent_record) = self.parent {
            let child_elapsed = &mut time_records[parent_record].as_mut().unwrap().elapsed_exclusive;
            *child_elapsed = child_elapsed.wrapping_sub(elapsed);
        }
        let time_record = time_records[self.record].as_mut().unwrap();
        time_record.elapsed_exclusive += elapsed;
        time_record.elapsed_inclusive = self.old_elapsed_inclusive + elapsed;
        time_record.hit_count += 1;
    }
}