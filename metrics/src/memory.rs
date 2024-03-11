use std::sync::OnceLock;
use windows_sys::Win32::Foundation::{FALSE, HANDLE};
use windows_sys::Win32::System::Threading::{GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

struct OSMetrics {
    process_handle: HANDLE,
}

impl OSMetrics {
    pub fn get_global_metrics() -> &'static OSMetrics {
        static GLOBAL_METRICS: OnceLock<OSMetrics> = OnceLock::new();
        GLOBAL_METRICS.get_or_init(|| {
            OSMetrics {
                process_handle: unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, GetCurrentProcessId()) },
            }
        })
    }
}

pub fn read_os_page_fault_count() -> u64 {
    let mut memory_counters: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
    memory_counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    
    let global_metrics = OSMetrics::get_global_metrics();

    unsafe {
        GetProcessMemoryInfo(global_metrics.process_handle, &mut memory_counters, memory_counters.cb);
    }

    memory_counters.PageFaultCount as u64
}