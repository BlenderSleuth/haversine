#[allow(dead_code)]
extern "C" {
    pub fn NOPAllBytesASM(count: u64);
    pub fn MOVAllBytesASM(count: u64, data: *mut u8);
    pub fn CMPAllBytesASM(count: u64);
    pub fn DECAllBytesASM(count: u64);
}