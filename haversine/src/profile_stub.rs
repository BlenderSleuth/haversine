#[macro_export]
macro_rules! time_block {
    ($name:expr, $idx:literal) => {};
}
pub use time_block;

#[macro_export]
macro_rules! time_function {
    ($idx:literal) => {};
}
pub use time_function;

pub fn print_time_records(_: u64) {}