use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;

use libc::{fopen, fclose, fread};

use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_READ, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, ReadFile};

use metrics::repetition_tester::{RepetitionTester, test_block};

pub struct ReadTestParameters<'a> {
    pub dest: &'a mut [u8],
    pub filename: &'a str,
}

type TestFunction = fn(&mut RepetitionTester, &mut ReadTestParameters);

pub struct ReadTestFunction {
    pub name: &'static str,
    pub func: TestFunction,
}

pub const TESTS: &[ReadTestFunction] = &[
    ReadTestFunction { name: "read", func: test_read },
    ReadTestFunction { name: "fread", func: test_fread },
    ReadTestFunction { name: "ReadFile", func: test_readfile },
];

fn test_read(tester: &mut RepetitionTester, params: &mut ReadTestParameters) {
    while tester.testing() {
        let mut file = File::open(params.filename);

        if let Ok(file) = &mut file {
            let result= {
                test_block!(tester);
                file.read(params.dest)
            };

            if let Ok(read_size) = result {
                tester.count_bytes(read_size as u64);
            } else {
                tester.error("File read error");
            }
        } else {
            tester.error("File open error");
        }
    }
}

fn test_fread(tester: &mut RepetitionTester, params: &mut ReadTestParameters) {
    while tester.testing() {
        let filename = CString::new(params.filename).unwrap();
        let file = unsafe {
            // Rust 1.77 (March 21 2024): use c literal syntax
            fopen(filename.as_ptr(), CStr::from_bytes_with_nul("rb\0".as_bytes()).unwrap().as_ptr())
        };
        
        if !file.is_null() {
            
            let result = unsafe {
                test_block!(tester);
                fread(params.dest.as_mut_ptr() as *mut libc::c_void, params.dest.len(), 1, file)
            };
            
            if result == 1 {
                tester.count_bytes(params.dest.len() as u64);
            } else {
                tester.error("fread failed");
            }
            
            unsafe {
                fclose(file);
            }
            
        } else {
            tester.error("fopen failed");
        }
    }
}

fn test_readfile(tester: &mut RepetitionTester, params: &mut ReadTestParameters) {
    while tester.testing() {
        let file = unsafe {
            let filename = CString::new(params.filename).unwrap();
            CreateFileA(filename.as_bytes_with_nul().as_ptr(), GENERIC_READ, FILE_SHARE_READ|FILE_SHARE_WRITE, std::ptr::null_mut(), OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, 0)
        };
        
        if file != INVALID_HANDLE_VALUE {
            let mut size_remaining = params.dest.len() as u64;
            let mut dest = params.dest.as_mut_ptr();
            while size_remaining > 0
            {
                let mut read_size = u32::MAX;
                if read_size as u64 > size_remaining
                {
                    read_size = size_remaining as u32;
                }

                let mut bytes_read = 0;
                let result = unsafe {
                    test_block!(tester);
                    ReadFile(file, dest, read_size, std::ptr::from_mut(&mut bytes_read), std::ptr::null_mut())
                };
                
                if result != 0 && (bytes_read == read_size)
                {
                    tester.count_bytes(read_size as u64);
                }
                else
                {
                    tester.error("ReadFile failed");
                }

                size_remaining -= read_size as u64;
                dest = unsafe { dest.add(read_size as usize) };
            }

            unsafe {
                CloseHandle(file);
            }
        } else {
            tester.error("CreateFileA failed");
        }
    }
}