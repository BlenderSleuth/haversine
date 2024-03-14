use crate::testing::ASMFunction;

extern "C" {
    pub fn Read_4x2(count: u64, data: *mut u8);
    pub fn Read_8x2(count: u64, data: *mut u8);
    #[cfg(target_feature = "avx2")]
    pub fn Read_16x2(count: u64, data: *mut u8);
    #[cfg(target_feature = "avx2")]
    pub fn Read_32x2(count: u64, data: *mut u8);
    #[cfg(target_feature = "avx512")]
    pub fn Read_64x2(count: u64, data: *mut u8);
}

pub const READ_WIDTH_TESTS: &[ASMFunction] = &[
    ASMFunction { name: "Read_4x2", func: Read_4x2 },
    ASMFunction { name: "Read_8x2", func: Read_8x2 },
    #[cfg(target_feature = "avx2")]
    ASMFunction { name: "Read_16x2", func: Read_16x2 },
    #[cfg(target_feature = "avx2")]
    ASMFunction { name: "Read_32x2", func: Read_32x2 },
    #[cfg(target_feature = "avx512")]
    ASMFunction { name: "Read_64x2", func: Read_64x2 },
];