use std::num::NonZeroU64;
use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

pub fn get_os_timer_freq() -> u64 {
    let mut freq = 0;
    unsafe {
        QueryPerformanceFrequency(&mut freq);
    }
    freq as u64
}

pub fn read_os_timer() -> u64 {
    let mut count = 0;
    unsafe {
        QueryPerformanceCounter(&mut count);
    }
    count as u64
}

pub fn read_cpu_timer() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

pub fn estimate_cpu_frequency(millis_to_wait: u64) -> u64 {
    let os_freq = get_os_timer_freq();

    let mut os_elapsed: u64 = 0;
    let os_wait_time = millis_to_wait * os_freq / 1000;

    let cpu_start = read_cpu_timer();
    let os_start = read_os_timer();
    while os_elapsed < os_wait_time {
        let os_end = read_os_timer();
        os_elapsed = os_end - os_start;
    }
    let cpu_end = read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;

    if os_elapsed > 0 {
        (cpu_elapsed * os_freq) / os_elapsed
    } else {
        0
    }
}

pub fn cpu_time_to_seconds(cpu_time: u64, cpu_freq: NonZeroU64) -> f64 {
    cpu_time as f64 / cpu_freq.get() as f64
}
