use crate::testing::ASMFunction;

extern "C" {
    pub fn Read_4x1(count: u64, data: *mut u8);
    pub fn Read_4x2(count: u64, data: *mut u8);
    pub fn Read_4x3(count: u64, data: *mut u8);
    pub fn Read_4x4(count: u64, data: *mut u8);
    pub fn Read_8x1(count: u64, data: *mut u8);
    pub fn Read_8x2(count: u64, data: *mut u8);
    pub fn Read_8x3(count: u64, data: *mut u8);
    pub fn Read_8x4(count: u64, data: *mut u8);
}

#[cfg(target_feature = "avx")]
extern "C" {
    pub fn Read_16x1(count: u64, data: *mut u8);
    pub fn Read_16x2(count: u64, data: *mut u8);
    pub fn Read_16x3(count: u64, data: *mut u8);
    pub fn Read_16x4(count: u64, data: *mut u8);
    pub fn Read_32x1(count: u64, data: *mut u8);
    pub fn Read_32x2(count: u64, data: *mut u8);
    pub fn Read_32x3(count: u64, data: *mut u8);
    pub fn Read_32x4(count: u64, data: *mut u8);
}

#[cfg(target_feature = "avx512f")]
extern "C" {
    pub fn Read_64x1(count: u64, data: *mut u8);
    pub fn Read_64x2(count: u64, data: *mut u8);
    pub fn Read_64x3(count: u64, data: *mut u8);
    pub fn Read_64x4(count: u64, data: *mut u8);
}

#[allow(dead_code)]
pub const READ_WIDTH_TESTS: &[ASMFunction] = &[
    ASMFunction { name: "Read_4x1", func: Read_4x1 },
    ASMFunction { name: "Read_4x2", func: Read_4x2 },
    ASMFunction { name: "Read_4x3", func: Read_4x3 },
    ASMFunction { name: "Read_4x4", func: Read_4x4 },
    ASMFunction { name: "Read_8x1", func: Read_8x1 },
    ASMFunction { name: "Read_8x2", func: Read_8x2 },
    ASMFunction { name: "Read_8x3", func: Read_8x3 },
    ASMFunction { name: "Read_8x4", func: Read_8x4 },
    #[cfg(target_feature = "avx")]
    ASMFunction { name: "Read_16x1", func: Read_16x1 },
    #[cfg(target_feature = "avx")]
    ASMFunction { name: "Read_16x2", func: Read_16x2 },
    #[cfg(target_feature = "avx")]
    ASMFunction { name: "Read_16x3", func: Read_16x3 },
    #[cfg(target_feature = "avx")]
    ASMFunction { name: "Read_16x4", func: Read_16x4 },
    #[cfg(target_feature = "avx")]
    ASMFunction { name: "Read_32x1", func: Read_32x1 },
    #[cfg(target_feature = "avx")]
    ASMFunction { name: "Read_32x2", func: Read_32x2 },
    #[cfg(target_feature = "avx")]
    ASMFunction { name: "Read_32x3", func: Read_32x3 },
    #[cfg(target_feature = "avx")]
    ASMFunction { name: "Read_32x4", func: Read_32x4 },
    #[cfg(target_feature = "avx512f")]
    ASMFunction { name: "Read_64x1", func: Read_64x1 },
    #[cfg(target_feature = "avx512f")]
    ASMFunction { name: "Read_64x2", func: Read_64x2 },
    #[cfg(target_feature = "avx512f")]
    ASMFunction { name: "Read_64x3", func: Read_64x3 },
    #[cfg(target_feature = "avx512f")]
    ASMFunction { name: "Read_64x4", func: Read_64x4 },
];

/* Aura results:

--- Read_4x1 ---
Min: 160231327 (64.194890ms) 16.329068 GB/s               
Max: 385825428 (154.576646ms) 6.781378 GB/s
Avg: 176575816 (70.743127ms) 14.817591 GB/s

--- Read_4x2 ---
Min: 80205919 (32.133605ms) 32.621385 GB/s                
Max: 385706796 (154.529117ms) 6.783464 GB/s
Avg: 88329925 (35.388397ms) 29.621085 GB/s

--- Read_4x3 ---
Min: 80104982 (32.093166ms) 32.662490 GB/s                
Max: 218689354 (87.615445ms) 11.964131 GB/s
Avg: 87767398 (35.163027ms) 29.810935 GB/s

--- Read_4x4 ---
Min: 79926308 (32.021582ms) 32.735507 GB/s                
Max: 187681545 (75.192513ms) 13.940786 GB/s
Avg: 86733251 (34.748708ms) 30.166380 GB/s

--- Read_8x1 ---
Min: 79963866 (32.036629ms) 32.720131 GB/s                
Max: 212522921 (85.144933ms) 12.311275 GB/s
Avg: 88891787 (35.613501ms) 29.433857 GB/s

--- Read_8x2 ---
Min: 39848084 (15.964689ms) 65.660075 GB/s                
Max: 142752344 (57.192131ms) 18.328443 GB/s
Avg: 44149813 (17.688129ms) 59.262498 GB/s

--- Read_8x3 ---
Min: 39836684 (15.960122ms) 65.678865 GB/s                
Max: 179133850 (71.767975ms) 14.605995 GB/s
Avg: 43631957 (17.480656ms) 59.965868 GB/s

--- Read_8x4 ---
Min: 39832686 (15.958520ms) 65.685457 GB/s                
Max: 144763070 (57.997706ms) 18.073865 GB/s
Avg: 43426183 (17.398215ms) 60.250015 GB/s

--- Read_16x1 ---
Min: 39853810 (15.966984ms) 65.650641 GB/s                
Max: 164606566 (65.947781ms) 15.895041 GB/s
Avg: 44210656 (17.712505ms) 59.180940 GB/s

--- Read_16x2 ---
Min: 19763056 (7.917848ms) 132.389859 GB/s                
Max: 174204741 (69.793182ms) 15.019271 GB/s
Avg: 22207102 (8.897027ms) 117.819434 GB/s

--- Read_16x3 ---
Min: 19754808 (7.914543ms) 132.445134 GB/s                
Max: 111250267 (44.571176ms) 23.518399 GB/s
Avg: 21858867 (8.757511ms) 119.696423 GB/s

--- Read_16x4 ---
Min: 19757584 (7.915655ms) 132.426525 GB/s                
Max: 1270031811455 (508824.050637ms) 0.002060 GB/s
Avg: 38492912 (15.421755ms) 67.971688 GB/s

--- Read_32x1 ---
Min: 19755544 (7.914838ms) 132.440199 GB/s                
Max: 150290865 (60.212355ms) 17.409097 GB/s
Avg: 22129024 (8.865746ms) 118.235137 GB/s

--- Read_32x2 ---
Min: 9836150 (3.940744ms) 266.001249 GB/s                 
Max: 128254547 (51.383751ms) 20.400276 GB/s
Avg: 11027756 (4.418147ms) 237.258440 GB/s

--- Read_32x3 ---
Min: 9844784 (3.944203ms) 265.767963 GB/s                 
Max: 115166092 (46.140008ms) 22.718737 GB/s
Avg: 10980332 (4.399147ms) 238.283158 GB/s

--- Read_32x4 ---
Min: 9840996 (3.942685ms) 265.870262 GB/s                 
Max: 77075144 (30.879295ms) 33.946459 GB/s
Avg: 10943145 (4.384249ms) 239.092892 GB/s

*/