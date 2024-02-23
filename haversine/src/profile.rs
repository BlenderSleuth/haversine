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
    child_elapsed: Option<u64>,
    hit_count: u64,
}

impl TimeRecord {
    fn new(label: &'static str) -> TimeRecord {
        TimeRecord { label, elapsed: 0, child_elapsed: None, hit_count: 0 }
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

pub fn print_time_records(total: u64) {
    let total_rcp = 100.0 / total as f64;
    let time_records = get_time_records();
    for record in time_records.iter().flatten() {
        let elapsed = record.elapsed - record.child_elapsed.unwrap_or(0);
        print!("  {}[{}]: {} ({:.2}%", record.label, record.hit_count, elapsed, elapsed as f64 * total_rcp);
        
        if record.child_elapsed.is_some() {
            print!(", {:.2}% w/children", record.elapsed as f64 * total_rcp);
        }
        
        println!(")");
    }
}

pub struct TimeBlock {
    start: u64,
    record: usize,
    parent: Option<usize>,
}

impl TimeBlock {
    pub fn new(label: &'static str, record: usize) -> Self {
        let time_records = get_time_records();
        time_records[record] .get_or_insert(TimeRecord::new(label));
        
        let parent = unsafe { PARENT_TIME_RECORD };
        unsafe { PARENT_TIME_RECORD = Some(record) }
        TimeBlock { start: read_cpu_timer(), record, parent }
    }
}

impl Drop for TimeBlock {
    fn drop(&mut self) {
        let elapsed = read_cpu_timer() - self.start;
        unsafe { PARENT_TIME_RECORD = self.parent }
        
        let time_records = get_time_records();
        if let Some(parent_record) = self.parent {
            let child_elapsed = time_records[parent_record].as_mut().unwrap().child_elapsed.get_or_insert(0);
            *child_elapsed += elapsed;
        }
        let time_record = time_records[self.record].as_mut().unwrap();
        time_record.elapsed += elapsed;
        time_record.hit_count += 1;
    }
}