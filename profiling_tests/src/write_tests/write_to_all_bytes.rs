use metrics::repetition_tester::{RepetitionTester, test_block};
use crate::write_tests::{WriteTestAllocBlock, WriteTestParameters};


pub fn write_to_all_bytes(tester: &mut RepetitionTester, params: &mut WriteTestParameters) {
    while tester.testing() {
        let alloc = WriteTestAllocBlock::new(params);

        let dest = alloc.get_dest();
        let bytes = dest.len();
        {
            test_block!(tester);
            for i in 0..bytes {
                dest[i] = i as u8;
            }
        }
        tester.count_bytes(bytes as u64);
    }
}