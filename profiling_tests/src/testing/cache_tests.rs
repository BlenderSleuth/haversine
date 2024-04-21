use std::mem::MaybeUninit;
use metrics::repetition_tester::RepetitionTester;
use metrics::test_block;
use crate::testing::{AllocType, TestParameters, TRY_FOR_SECONDS};

extern "C" {
    #[cfg(target_feature = "avx")]
    pub fn CacheTest(count: u64, data: *mut u8, mask: u64);
    #[cfg(target_feature = "avx")]
    pub fn CacheTest2(data: *mut u8, inner_count: u64, outer_count: u64);
}

const MIN_CACHE_IDX: usize = 10; // 1GB
const MAX_CACHE_IDX: usize = 30; // 1GB

fn print_csv(testers: &[RepetitionTester], cpu_freq: u64, size: u64) {
    println!();
    println!("Region Size,GB/s");
    for (cache_size_idx, tester) in (MIN_CACHE_IDX..=MAX_CACHE_IDX).zip(testers.iter()) {
        let cache_size = 1u64 << cache_size_idx;
        let seconds = tester.get_result().min.time as f64 / cpu_freq as f64;
        const GB: f64 = 1024.0 * 1024.0 * 1024.0;
        let bandwidth = size as f64 / GB / seconds;
        println!("{cache_size},{bandwidth}");
    }
}

#[allow(dead_code)]
#[cfg(target_feature = "avx")]
pub fn cache_test_loop(size: u64, cpu_freq: u64) {
    let mut params = TestParameters::new(AllocType::None, size as usize, "");
    let dest = params.handle_allocation();
    for (i, byte) in dest.iter_mut().enumerate() {
        *byte = MaybeUninit::new(i as u8);
    }
    
    let mut testers = [RepetitionTester::new(size, cpu_freq); MAX_CACHE_IDX-MIN_CACHE_IDX+1];
    
    for (cache_size_idx, tester) in (MIN_CACHE_IDX..=MAX_CACHE_IDX).zip(testers.iter_mut()) {
        let len = dest.len() as u64;

        let cache_size = 1u64 << cache_size_idx;
        print!("\n--- Cache Test: {}KB ---\n", cache_size/1024);

        tester.new_test_wave(len, cpu_freq, TRY_FOR_SECONDS);
        
        let mask = cache_size.next_power_of_two() - 1;

        while tester.testing() {
            unsafe {
                test_block!(tester);
                CacheTest(len, dest.as_mut_ptr() as *mut u8, mask);
            }
            tester.count_bytes(len);
        }

        if tester.has_error() {
            break;
        }
    }

    print_csv(&testers, cpu_freq, size);
}

pub struct CacheSizeTest{
    // Number of 256byte blocks to move
    pub inner_count: u64,
    pub outer_count: u64,
    pub real_size: u64
}

impl CacheSizeTest {
    pub fn new(full_size: u64, mut cache_size: u64) -> Self {
        
        let inner_count = cache_size / 256;
        cache_size = inner_count * 256;
        
        let outer_count = full_size / cache_size;
        let real_size = cache_size * outer_count;
        
        Self {
            inner_count,
            outer_count,
            real_size
        }
    }
    
    pub fn cache_size(&self) -> u64 {
        self.inner_count * 256
    }
}

// Allows testing of cache sizes of arbitrary multiples of 256 bytes
#[allow(dead_code)]
#[cfg(target_feature = "avx")]
pub fn cache_test_loop2(size: u64, cpu_freq: u64, tests: &[CacheSizeTest]) {
    let mut params = TestParameters::new(AllocType::None, size as usize, "");
    let dest = params.handle_allocation();
    for (i, byte) in dest.iter_mut().enumerate() {
        *byte = MaybeUninit::new(i as u8);
    }
    
    let mut testers: Vec<_> = tests.iter().map(|test| RepetitionTester::new(test.real_size, cpu_freq)).collect();

    for (size_test, tester) in tests.iter().zip(testers.iter_mut()) {
        print!("\n--- Cache Test: {}KB ---\n", size_test.cache_size() / 1024);

        tester.new_test_wave(size_test.real_size, cpu_freq, TRY_FOR_SECONDS);
        
        while tester.testing() {
            unsafe {
                test_block!(tester);
                CacheTest2(dest.as_mut_ptr() as *mut u8, size_test.inner_count, size_test.outer_count);
            }
            tester.count_bytes(size_test.real_size);
        }

        if tester.has_error() {
            break;
        }
    }

    print_csv(&testers, cpu_freq, size);
}

#[allow(dead_code)]
#[cfg(target_feature = "avx")]
pub fn cache_test_loop_unaligned(size: u64, cpu_freq: u64) {
    let mut params = TestParameters::new(AllocType::None, size as usize, "");
    let dest = params.handle_allocation();
    for (i, byte) in dest.iter_mut().enumerate() {
        *byte = MaybeUninit::new(i as u8);
    }

    const MAX_UNALIGNMENT_IDX: usize = 6;
    const MAX_UNALIGNMENT: usize = 1 << MAX_UNALIGNMENT_IDX;
    // Reduce size by 1 so we don't read past the end of the buffer.
    let size = size-(MAX_UNALIGNMENT as u64);

    let mut testers = [[RepetitionTester::new(size, cpu_freq); MAX_UNALIGNMENT_IDX]; MAX_CACHE_IDX-MIN_CACHE_IDX+1];

    for (cache_size_idx, tester) in (MIN_CACHE_IDX..=MAX_CACHE_IDX).zip(testers.iter_mut()) {
        for (unalignment_idx, tester) in tester.iter_mut().enumerate() {
            let cache_size = 1u64 << cache_size_idx;
            let unalignment = (1 << unalignment_idx) - 1;
            print!("\n--- Cache Test: {}KB, {unalignment}B unaligned ---\n", cache_size / 1024);

            tester.new_test_wave(size, cpu_freq, TRY_FOR_SECONDS);

            let mask = cache_size.next_power_of_two() - 1;

            while tester.testing() {
                unsafe {
                    test_block!(tester);
                    CacheTest(size, dest.as_mut_ptr().add(unalignment) as *mut u8, mask);
                }
                tester.count_bytes(size);
            }

            if tester.has_error() {
                break;
            }
        }
    }

    
    //print_csv(, cpu_freq, size);
}
//*
//     Finished `release` profile [optimized] target(s) in 0.03s
//      Running `target\release\profiling_tests.exe`
// 
// --- Cache Test: 1KB, 0B unaligned ---
// Min: 9875538 (3.956526ms) 252.746970 GB/s                                        
// Max: 17654004 (7.072883ms) 141.385054 GB/s
// Avg: 10378499 (4.158032ms) 240.498391 GB/s
// 
// --- Cache Test: 1KB, 1B unaligned ---
// Min: 19745718 (7.910905ms) 126.407777 GB/s                                       
// Max: 26863242 (10.762463ms) 92.915528 GB/s
// Avg: 20454982 (8.195064ms) 122.024664 GB/s
// 
// --- Cache Test: 1KB, 3B unaligned ---
// Min: 19751630 (7.913274ms) 126.369941 GB/s                                       
// Max: 25745305 (10.314574ms) 96.950194 GB/s
// Avg: 20321026 (8.141396ms) 122.829050 GB/s
// 
// --- Cache Test: 1KB, 7B unaligned ---
// Min: 19758754 (7.916128ms) 126.324378 GB/s                                       
// Max: 45539088 (18.244736ms) 54.810327 GB/s
// Avg: 20884756 (8.367248ms) 119.513597 GB/s
// 
// --- Cache Test: 1KB, 15B unaligned ---
// Min: 19763770 (7.918138ms) 126.292317 GB/s                                       
// Max: 71256364 (28.548080ms) 35.028623 GB/s
// Avg: 21107519 (8.456496ms) 118.252283 GB/s
// 
// --- Cache Test: 1KB, 31B unaligned ---
// Min: 20190880 (8.089254ms) 123.620779 GB/s                                       
// Max: 29525708 (11.829151ms) 84.536916 GB/s
// Avg: 20874401 (8.363100ms) 119.572883 GB/s
// 
// --- Cache Test: 2KB, 0B unaligned ---
// Min: 9840460 (3.942472ms) 253.647930 GB/s                                        
// Max: 18283566 (7.325110ms) 136.516712 GB/s
// Avg: 10556865 (4.229492ms) 236.434994 GB/s
// 
// --- Cache Test: 2KB, 1B unaligned ---
// Min: 19774376 (7.922387ms) 126.224580 GB/s                                       
// Max: 44039269 (17.643850ms) 56.676970 GB/s
// Avg: 20988189 (8.408688ms) 118.924616 GB/s
// 
// --- Cache Test: 2KB, 3B unaligned ---
// Min: 19767090 (7.919468ms) 126.271106 GB/s                                       
// Max: 31853524 (12.761765ms) 78.359064 GB/s
// Avg: 20891501 (8.369951ms) 119.475011 GB/s
// 
// --- Cache Test: 2KB, 7B unaligned ---
// Min: 19786758 (7.927347ms) 126.145592 GB/s                                       
// Max: 26684535 (10.690866ms) 93.537786 GB/s
// Avg: 20868985 (8.360930ms) 119.603915 GB/s
// 
// --- Cache Test: 2KB, 15B unaligned ---
// Min: 19836016 (7.947082ms) 125.832340 GB/s                                       
// Max: 27889071 (11.173450ms) 89.497865 GB/s
// Avg: 20827263 (8.344214ms) 119.843510 GB/s
// 
// --- Cache Test: 2KB, 31B unaligned ---
// Min: 19771952 (7.921416ms) 126.240055 GB/s                                       
// Max: 29249499 (11.718491ms) 85.335216 GB/s
// Avg: 20908671 (8.376830ms) 119.376899 GB/s
// 
// --- Cache Test: 4KB, 0B unaligned ---
// Min: 9879550 (3.958133ms) 252.644332 GB/s                                        
// Max: 16633711 (6.664114ms) 150.057453 GB/s
// Avg: 10514050 (4.212339ms) 237.397797 GB/s
// 
// --- Cache Test: 4KB, 1B unaligned ---
// Min: 19877042 (7.963519ms) 125.572623 GB/s                                       
// Max: 26289929 (10.532772ms) 94.941767 GB/s
// Avg: 20752020 (8.314069ms) 120.278041 GB/s
// 
// --- Cache Test: 4KB, 3B unaligned ---
// Min: 19960808 (7.997079ms) 125.045655 GB/s                                       
// Max: 28096250 (11.256454ms) 88.837916 GB/s
// Avg: 20786623 (8.327932ms) 120.077817 GB/s
// 
// --- Cache Test: 4KB, 7B unaligned ---
// Min: 19985450 (8.006951ms) 124.891474 GB/s                                       
// Max: 25999144 (10.416272ms) 96.003634 GB/s
// Avg: 20815800 (8.339622ms) 119.909507 GB/s
// 
// --- Cache Test: 4KB, 15B unaligned ---
// Min: 19762820 (7.917757ms) 126.298388 GB/s                                       
// Max: 27892919 (11.174992ms) 89.485518 GB/s
// Avg: 20896908 (8.372117ms) 119.444097 GB/s
// 
// --- Cache Test: 4KB, 31B unaligned ---
// Min: 19784966 (7.926630ms) 126.157018 GB/s                                       
// Max: 26403712 (10.578357ms) 94.532629 GB/s
// Avg: 20984121 (8.407058ms) 118.947671 GB/s
// 
// --- Cache Test: 8KB, 0B unaligned ---
// Min: 9870126 (3.954358ms) 252.885557 GB/s                                        
// Max: 35303124 (14.143809ms) 70.702307 GB/s
// Avg: 10780114 (4.318934ms) 231.538582 GB/s
// 
// --- Cache Test: 8KB, 1B unaligned ---
// Min: 19317846 (7.739483ms) 129.207589 GB/s                                       
// Max: 66813523 (26.768105ms) 37.357891 GB/s
// Avg: 20750800 (8.313580ms) 120.285112 GB/s
// 
// --- Cache Test: 8KB, 3B unaligned ---
// Min: 19760116 (7.916674ms) 126.315671 GB/s                                       
// Max: 32209363 (12.904328ms) 77.493377 GB/s
// Avg: 20619557 (8.260999ms) 121.050724 GB/s
// 
// --- Cache Test: 8KB, 7B unaligned ---
// Min: 19765228 (7.918722ms) 126.283001 GB/s                                       
// Max: 32528867 (13.032334ms) 76.732224 GB/s
// Avg: 20871950 (8.362118ms) 119.586925 GB/s
// 
// --- Cache Test: 8KB, 15B unaligned ---
// Min: 19877250 (7.963602ms) 125.571309 GB/s                                       
// Max: 39771610 (15.934059ms) 62.758644 GB/s
// Avg: 21033694 (8.426919ms) 118.667330 GB/s
// 
// --- Cache Test: 8KB, 31B unaligned ---
// Min: 20052260 (8.033718ms) 124.475361 GB/s                                       
// Max: 27255114 (10.919462ms) 91.579595 GB/s
// Avg: 20914919 (8.379333ms) 119.341237 GB/s
// 
// --- Cache Test: 16KB, 0B unaligned ---
// Min: 9883984 (3.959910ms) 252.530995 GB/s                                        
// Max: 17319044 (6.938685ms) 144.119520 GB/s
// Avg: 10496807 (4.205431ms) 237.787768 GB/s
// 
// --- Cache Test: 16KB, 1B unaligned ---
// Min: 19775024 (7.922646ms) 126.220444 GB/s                                       
// Max: 27481594 (11.010199ms) 90.824874 GB/s
// Avg: 20821642 (8.341962ms) 119.875863 GB/s
// 
// --- Cache Test: 16KB, 3B unaligned ---
// Min: 20008496 (8.016184ms) 124.747623 GB/s                                       
// Max: 28708138 (11.501600ms) 86.944417 GB/s
// Avg: 20911109 (8.377806ms) 119.362981 GB/s
// 
// --- Cache Test: 16KB, 7B unaligned ---
// Min: 19958218 (7.996041ms) 125.061882 GB/s                                       
// Max: 26179188 (10.488404ms) 95.343382 GB/s
// Avg: 20755074 (8.315293ms) 120.260343 GB/s
// 
// --- Cache Test: 16KB, 15B unaligned ---
// Min: 19921866 (7.981477ms) 125.290086 GB/s                                       
// Max: 26529565 (10.628779ms) 94.084178 GB/s
// Avg: 20786747 (8.327982ms) 120.077101 GB/s
// 
// --- Cache Test: 16KB, 31B unaligned ---
// Min: 19992328 (8.009707ms) 124.848507 GB/s                                       
// Max: 26500029 (10.616946ms) 94.189041 GB/s
// Avg: 20846854 (8.352063ms) 119.730887 GB/s
// 
// --- Cache Test: 32KB, 0B unaligned ---
// Min: 9895226 (3.964414ms) 252.244093 GB/s                                        
// Max: 23014245 (9.220405ms) 108.455103 GB/s
// Avg: 10726588 (4.297490ms) 232.693967 GB/s
// 
// --- Cache Test: 32KB, 1B unaligned ---
// Min: 20062088 (8.037655ms) 124.414384 GB/s                                       
// Max: 31338958 (12.555610ms) 79.645670 GB/s
// Avg: 21129536 (8.465317ms) 118.129064 GB/s
// 
// --- Cache Test: 32KB, 3B unaligned ---
// Min: 19871640 (7.961354ms) 125.606760 GB/s                                       
// Max: 29778367 (11.930376ms) 83.819650 GB/s
// Avg: 21281234 (8.526093ms) 117.287010 GB/s
// 
// --- Cache Test: 32KB, 7B unaligned ---
// Min: 19864874 (7.958644ms) 125.649542 GB/s                                       
// Max: 32008498 (12.823853ms) 77.979676 GB/s
// Avg: 21236828 (8.508302ms) 117.532256 GB/s
// 
// --- Cache Test: 32KB, 15B unaligned ---
// Min: 20073986 (8.042422ms) 124.340642 GB/s                                       
// Max: 35953652 (14.404436ms) 69.423054 GB/s
// Avg: 21319393 (8.541381ms) 117.077081 GB/s
// 
// --- Cache Test: 32KB, 31B unaligned ---
// Min: 19866040 (7.959111ms) 125.642167 GB/s                                       
// Max: 32334060 (12.954286ms) 77.194522 GB/s
// Avg: 21148140 (8.472770ms) 118.025146 GB/s
// 
// --- Cache Test: 64KB, 0B unaligned ---
// Min: 22301124 (8.934701ms) 111.923162 GB/s                                       
// Max: 31809142 (12.743984ms) 78.468395 GB/s
// Avg: 24452908 (9.796789ms) 102.074253 GB/s
// 
// --- Cache Test: 64KB, 1B unaligned ---
// Min: 22064560 (8.839924ms) 113.123140 GB/s                                       
// Max: 36224676 (14.513019ms) 68.903648 GB/s
// Avg: 24292808 (9.732647ms) 102.746966 GB/s
// 
// --- Cache Test: 64KB, 3B unaligned ---
// Min: 22359928 (8.958260ms) 111.628817 GB/s                                       
// Max: 34753490 (13.923604ms) 71.820479 GB/s
// Avg: 24250995 (9.715895ms) 102.924120 GB/s
// 
// --- Cache Test: 64KB, 7B unaligned ---
// Min: 21991464 (8.810639ms) 113.499143 GB/s                                       
// Max: 36318905 (14.550771ms) 68.724878 GB/s
// Avg: 24233206 (9.708768ms) 102.999674 GB/s
// 
// --- Cache Test: 64KB, 15B unaligned ---
// Min: 22552126 (9.035262ms) 110.677473 GB/s                                       
// Max: 34324447 (13.751713ms) 72.718209 GB/s
// Avg: 24274010 (9.725116ms) 102.826534 GB/s
// 
// --- Cache Test: 64KB, 31B unaligned ---
// Min: 22624488 (9.064253ms) 110.323483 GB/s                                       
// Max: 36595765 (14.661692ms) 68.204950 GB/s
// Avg: 24446372 (9.794171ms) 102.101543 GB/s
// 
// --- Cache Test: 128KB, 0B unaligned ---
// Min: 22591232 (9.050929ms) 110.485887 GB/s                                       
// Max: 35743964 (14.320427ms) 69.830316 GB/s
// Avg: 24605560 (9.857948ms) 101.440988 GB/s
// 
// --- Cache Test: 128KB, 1B unaligned ---
// Min: 22386992 (8.969103ms) 111.493867 GB/s                                       
// Max: 34055396 (13.643921ms) 73.292711 GB/s
// Avg: 24344543 (9.753374ms) 102.528616 GB/s
// 
// --- Cache Test: 128KB, 3B unaligned ---
// Min: 22522918 (9.023560ms) 110.821001 GB/s                                       
// Max: 43039584 (17.243337ms) 57.993412 GB/s
// Avg: 24760808 (9.920146ms) 100.804962 GB/s
// 
// --- Cache Test: 128KB, 7B unaligned ---
// Min: 22990068 (9.210718ms) 108.569157 GB/s                                       
// Max: 35810138 (14.346939ms) 69.701276 GB/s
// Avg: 24487213 (9.810533ms) 101.931253 GB/s
// 
// --- Cache Test: 128KB, 15B unaligned ---
// Min: 22830064 (9.146615ms) 109.330062 GB/s                                       
// Max: 42031418 (16.839426ms) 59.384442 GB/s
// Avg: 24263213 (9.720790ms) 102.872291 GB/s
// 
// --- Cache Test: 128KB, 31B unaligned ---
// Min: 23066020 (9.241148ms) 108.211660 GB/s                                       
// Max: 32476734 (13.011447ms) 76.855398 GB/s
// Avg: 24128086 (9.666653ms) 103.448417 GB/s
// 
// --- Cache Test: 256KB, 0B unaligned ---
// Min: 28374716 (11.368019ms) 87.966072 GB/s                                       
// Max: 43072913 (17.256690ms) 57.948537 GB/s
// Avg: 29812235 (11.943945ms) 83.724428 GB/s
// 
// --- Cache Test: 256KB, 1B unaligned ---
// Min: 28022496 (11.226905ms) 89.071734 GB/s                                       
// Max: 90220143 (36.145710ms) 27.665799 GB/s
// Avg: 30428086 (12.190679ms) 82.029882 GB/s
// 
// --- Cache Test: 256KB, 3B unaligned ---
// Min: 27980778 (11.210192ms) 89.204536 GB/s                                       
// Max: 39092682 (15.662054ms) 63.848582 GB/s
// Avg: 29509011 (11.822461ms) 84.584750 GB/s
// 
// --- Cache Test: 256KB, 7B unaligned ---
// Min: 28049878 (11.237876ms) 88.984783 GB/s                                       
// Max: 41755985 (16.729077ms) 59.776157 GB/s
// Avg: 29508848 (11.822396ms) 84.585217 GB/s
// 
// --- Cache Test: 256KB, 15B unaligned ---
// Min: 27515214 (11.023669ms) 90.713898 GB/s                                       
// Max: 45670234 (18.297278ms) 54.652935 GB/s
// Avg: 29363035 (11.763978ms) 85.005256 GB/s
// 
// --- Cache Test: 256KB, 31B unaligned ---
// Min: 27988834 (11.213419ms) 89.178860 GB/s                                       
// Max: 45856350 (18.371843ms) 54.431116 GB/s
// Avg: 29354772 (11.760667ms) 85.029184 GB/s
// 
// --- Cache Test: 512KB, 0B unaligned ---
// Min: 38794798 (15.542710ms) 64.338840 GB/s                                       
// Max: 52757922 (21.136882ms) 47.310664 GB/s
// Avg: 40878601 (16.377563ms) 61.059142 GB/s
// 
// --- Cache Test: 512KB, 1B unaligned ---
// Min: 38384662 (15.378394ms) 65.026294 GB/s                                       
// Max: 50851314 (20.373021ms) 49.084519 GB/s
// Avg: 40867807 (16.373238ms) 61.075269 GB/s
// 
// --- Cache Test: 512KB, 3B unaligned ---
// Min: 38477346 (15.415526ms) 64.869659 GB/s                                       
// Max: 52308349 (20.956766ms) 47.717283 GB/s
// Avg: 40909162 (16.389807ms) 61.013528 GB/s
// 
// --- Cache Test: 512KB, 7B unaligned ---
// Min: 38328070 (15.355721ms) 65.122306 GB/s                                       
// Max: 50399314 (20.191932ms) 49.524728 GB/s
// Avg: 40908726 (16.389632ms) 61.014179 GB/s
// 
// --- Cache Test: 512KB, 15B unaligned ---
// Min: 37074534 (14.853505ms) 67.324172 GB/s                                       
// Max: 80279596 (32.163139ms) 31.091491 GB/s
// Avg: 40451038 (16.206264ms) 61.704531 GB/s
// 
// --- Cache Test: 512KB, 31B unaligned ---
// Min: 37595224 (15.062114ms) 66.391739 GB/s                                       
// Max: 51287619 (20.547822ms) 48.666956 GB/s
// Avg: 40064076 (16.051232ms) 62.300509 GB/s
// 
// --- Cache Test: 1024KB, 0B unaligned ---
// Min: 38134862 (15.278314ms) 65.452244 GB/s                                       
// Max: 51133830 (20.486208ms) 48.813326 GB/s
// Avg: 40293111 (16.142993ms) 61.946379 GB/s
// 
// --- Cache Test: 1024KB, 1B unaligned ---
// Min: 37732450 (15.117092ms) 66.150285 GB/s                                       
// Max: 1256689846 (503.478995ms) 1.986180 GB/s
// Avg: 50807046 (20.355285ms) 49.127287 GB/s
// 
// --- Cache Test: 1024KB, 3B unaligned ---
// Min: 43127282 (17.278472ms) 57.875484 GB/s                                       
// Max: 1672121872 (669.917278ms) 1.492722 GB/s
// Avg: 216371631 (86.686919ms) 11.535765 GB/s
// 
// --- Cache Test: 1024KB, 7B unaligned ---
// Min: 46995342 (18.828168ms) 53.111909 GB/s                                       
// Max: 1316761972 (527.546233ms) 1.895568 GB/s
// Avg: 99273139 (39.772694ms) 25.142877 GB/s
// 
// --- Cache Test: 1024KB, 15B unaligned ---
// Min: 50769436 (20.340217ms) 49.163680 GB/s                                       
// Max: 155042319 (62.116004ms) 16.098910 GB/s
// Avg: 57989336 (23.232791ms) 43.042609 GB/s
// 
// --- Cache Test: 1024KB, 31B unaligned ---
// Min: 43566176 (17.454310ms) 57.292435 GB/s                                       
// Max: 94632694 (37.913550ms) 26.375793 GB/s
// Avg: 57561630 (23.061435ms) 43.362433 GB/s
// 
// --- Cache Test: 2048KB, 0B unaligned ---
// Min: 44030636 (17.640391ms) 56.688082 GB/s                                       
// Max: 97740302 (39.158579ms) 25.537186 GB/s
// Avg: 57253372 (22.937935ms) 43.595901 GB/s
// 
// --- Cache Test: 2048KB, 1B unaligned ---
// Min: 46233614 (18.522990ms) 53.986961 GB/s                                       
// Max: 86106318 (34.497551ms) 28.987563 GB/s
// Avg: 58711734 (23.522212ms) 42.513006 GB/s
// 
// --- Cache Test: 2048KB, 3B unaligned ---
// Min: 51455906 (20.615244ms) 48.507791 GB/s                                       
// Max: 135440592 (54.262787ms) 18.428835 GB/s
// Avg: 58528489 (23.448797ms) 42.646109 GB/s
// 
// --- Cache Test: 2048KB, 7B unaligned ---
// Min: 46497268 (18.628620ms) 53.680838 GB/s                                       
// Max: 126041466 (50.497130ms) 19.803104 GB/s
// Avg: 59264962 (23.743857ms) 42.116155 GB/s
// 
// --- Cache Test: 2048KB, 15B unaligned ---
// Min: 50875466 (20.382697ms) 49.061218 GB/s                                       
// Max: 85647284 (34.313644ms) 29.142924 GB/s
// Avg: 58844742 (23.575500ms) 42.416913 GB/s
// 
// --- Cache Test: 2048KB, 31B unaligned ---
// Min: 47911462 (19.195201ms) 52.096350 GB/s                                       
// Max: 89518614 (35.864650ms) 27.882607 GB/s
// Avg: 58220216 (23.325291ms) 42.871918 GB/s
// 
// --- Cache Test: 4096KB, 0B unaligned ---
// Min: 56891630 (22.793007ms) 43.873102 GB/s                                       
// Max: 114665895 (45.939632ms) 21.767696 GB/s
// Avg: 68388683 (27.399175ms) 36.497447 GB/s
// 
// --- Cache Test: 4096KB, 1B unaligned ---
// Min: 51868486 (20.780540ms) 48.121943 GB/s                                       
// Max: 107226052 (42.958941ms) 23.278040 GB/s
// Avg: 69548987 (27.864038ms) 35.888550 GB/s
// 
// --- Cache Test: 4096KB, 3B unaligned ---
// Min: 54860172 (21.979126ms) 45.497712 GB/s                                       
// Max: 120859867 (48.421179ms) 20.652119 GB/s
// Avg: 71240483 (28.541718ms) 35.036432 GB/s
// 
// --- Cache Test: 4096KB, 7B unaligned ---
// Min: 54815697 (21.961307ms) 45.534627 GB/s                                       
// Max: 116075818 (46.504503ms) 21.503293 GB/s
// Avg: 69950702 (28.024981ms) 35.682448 GB/s
// 
// --- Cache Test: 4096KB, 15B unaligned ---
// Min: 52197312 (20.912280ms) 47.818790 GB/s                                       
// Max: 116998877 (46.874316ms) 21.333643 GB/s
// Avg: 70101845 (28.085535ms) 35.605515 GB/s
// 
// --- Cache Test: 4096KB, 31B unaligned ---
// Min: 51656780 (20.695722ms) 48.319162 GB/s                                       
// Max: 135289121 (54.202102ms) 18.449468 GB/s
// Avg: 69910210 (28.008758ms) 35.703116 GB/s
// 
// --- Cache Test: 8192KB, 0B unaligned ---
// Min: 108794452 (43.587303ms) 22.942460 GB/s                                       
// Max: 186544348 (74.736946ms) 13.380262 GB/s
// Avg: 124103485 (49.720699ms) 20.112347 GB/s
// 
// --- Cache Test: 8192KB, 1B unaligned ---
// Min: 119134870 (47.730078ms) 20.951148 GB/s                                       
// Max: 176430020 (70.684751ms) 14.147322 GB/s
// Avg: 129733198 (51.976182ms) 19.239581 GB/s
// 
// --- Cache Test: 8192KB, 3B unaligned ---
// Min: 115649914 (46.333869ms) 21.582483 GB/s                                       
// Max: 178482522 (71.507064ms) 13.984632 GB/s
// Avg: 130221617 (52.171862ms) 19.167419 GB/s
// 
// --- Cache Test: 8192KB, 7B unaligned ---
// Min: 117677670 (47.146267ms) 21.210586 GB/s                                       
// Max: 183129464 (73.368810ms) 13.629769 GB/s
// Avg: 129210060 (51.766593ms) 19.317477 GB/s
// 
// --- Cache Test: 8192KB, 15B unaligned ---
// Min: 115453108 (46.255021ms) 21.619273 GB/s                                       
// Max: 184361571 (73.862440ms) 13.538680 GB/s
// Avg: 129868378 (52.030340ms) 19.219554 GB/s
// 
// --- Cache Test: 8192KB, 31B unaligned ---
// Min: 113328384 (45.403773ms) 22.024600 GB/s                                       
// Max: 177302460 (71.034285ms) 14.077708 GB/s
// Avg: 128124064 (51.331500ms) 19.481214 GB/s
// 
// --- Cache Test: 16384KB, 0B unaligned ---
// Min: 143304086 (57.413209ms) 17.417593 GB/s                                       
// Max: 213625906 (85.586875ms) 11.684034 GB/s
// Avg: 159800306 (64.022239ms) 15.619572 GB/s
// 
// --- Cache Test: 16384KB, 1B unaligned ---
// Min: 144776671 (58.003184ms) 17.240432 GB/s                                       
// Max: 234260894 (93.854056ms) 10.654840 GB/s
// Avg: 158972815 (63.690714ms) 15.700875 GB/s
// 
// --- Cache Test: 16384KB, 3B unaligned ---
// Min: 124594090 (49.917255ms) 20.033152 GB/s                                       
// Max: 175638465 (70.367623ms) 14.211080 GB/s
// Avg: 151521310 (60.705350ms) 16.473012 GB/s
// 
// --- Cache Test: 16384KB, 7B unaligned ---
// Min: 138381440 (55.441005ms) 18.037190 GB/s                                       
// Max: 168795841 (67.626201ms) 14.787167 GB/s
// Avg: 147066875 (58.920730ms) 16.971955 GB/s
// 
// --- Cache Test: 16384KB, 15B unaligned ---
// Min: 136575963 (54.717661ms) 18.275634 GB/s                                       
// Max: 174543120 (69.928786ms) 14.300262 GB/s
// Avg: 147078435 (58.925361ms) 16.970621 GB/s
// 
// --- Cache Test: 16384KB, 31B unaligned ---
// Min: 137353277 (55.029083ms) 18.172208 GB/s                                       
// Max: 175510837 (70.316491ms) 14.221414 GB/s
// Avg: 147374895 (59.044134ms) 16.936482 GB/s
// 
// --- Cache Test: 32768KB, 0B unaligned ---
// Min: 144587328 (57.927326ms) 17.263009 GB/s                                       
// Max: 177625740 (71.163803ms) 14.052087 GB/s
// Avg: 162546170 (65.122339ms) 15.355713 GB/s
// 
// --- Cache Test: 32768KB, 1B unaligned ---
// Min: 144225288 (57.782279ms) 17.306343 GB/s                                       
// Max: 183381828 (73.469917ms) 13.611012 GB/s
// Avg: 152657670 (61.160620ms) 16.350389 GB/s
// 
// --- Cache Test: 32768KB, 3B unaligned ---
// Min: 142286836 (57.005659ms) 17.542117 GB/s                                       
// Max: 179855623 (72.057182ms) 13.877866 GB/s
// Avg: 153280281 (61.410062ms) 16.283975 GB/s
// 
// --- Cache Test: 32768KB, 7B unaligned ---
// Min: 143790462 (57.608071ms) 17.358678 GB/s                                       
// Max: 181743225 (72.813429ms) 13.733730 GB/s
// Avg: 152955647 (61.280001ms) 16.318537 GB/s
// 
// --- Cache Test: 32768KB, 15B unaligned ---
// Min: 144527935 (57.903531ms) 17.270103 GB/s                                       
// Max: 174017004 (69.718003ms) 14.343497 GB/s
// Avg: 152635851 (61.151879ms) 16.352726 GB/s
// 
// --- Cache Test: 32768KB, 31B unaligned ---
// Min: 145155715 (58.155044ms) 17.195412 GB/s                                       
// Max: 191824610 (76.852425ms) 13.011950 GB/s
// Avg: 155028889 (62.110623ms) 16.100304 GB/s
// 
// --- Cache Test: 65536KB, 0B unaligned ---
// Min: 154690251 (61.974951ms) 16.135550 GB/s                                       
// Max: 195965116 (78.511273ms) 12.737024 GB/s
// Avg: 164636093 (65.959644ms) 15.160784 GB/s
// 
// --- Cache Test: 65536KB, 1B unaligned ---
// Min: 134385816 (53.840202ms) 18.573480 GB/s                                       
// Max: 194601694 (77.965033ms) 12.826262 GB/s
// Avg: 147243321 (58.991421ms) 16.951617 GB/s
// 
// --- Cache Test: 65536KB, 3B unaligned ---
// Min: 131507924 (52.687207ms) 18.979939 GB/s                                       
// Max: 151668614 (60.764366ms) 16.457013 GB/s
// Avg: 138305800 (55.410701ms) 18.047055 GB/s
// 
// --- Cache Test: 65536KB, 7B unaligned ---
// Min: 129910924 (52.047386ms) 19.213260 GB/s                                       
// Max: 153428910 (61.469609ms) 16.268201 GB/s
// Avg: 138006388 (55.290745ms) 18.086209 GB/s
// 
// --- Cache Test: 65536KB, 15B unaligned ---
// Min: 126709187 (50.764645ms) 19.698748 GB/s                                       
// Max: 160103580 (64.143742ms) 15.589984 GB/s
// Avg: 138514584 (55.494348ms) 18.019852 GB/s
// 
// --- Cache Test: 65536KB, 31B unaligned ---
// Min: 131911032 (52.848707ms) 18.921938 GB/s                                       
// Max: 173581028 (69.543334ms) 14.379523 GB/s
// Avg: 138615299 (55.534698ms) 18.006759 GB/s
// 
// --- Cache Test: 131072KB, 0B unaligned ---
// Min: 138007482 (55.291183ms) 18.086065 GB/s                                       
// Max: 179325380 (71.844746ms) 13.918902 GB/s
// Avg: 148543326 (59.512253ms) 16.803261 GB/s
// 
// --- Cache Test: 131072KB, 1B unaligned ---
// Min: 130756348 (52.386096ms) 19.089034 GB/s                                       
// Max: 157570692 (63.128969ms) 15.840587 GB/s
// Avg: 139567239 (55.916083ms) 17.883941 GB/s
// 
// --- Cache Test: 131072KB, 3B unaligned ---
// Min: 132075368 (52.914547ms) 18.898394 GB/s                                       
// Max: 154510831 (61.903069ms) 16.154287 GB/s
// Avg: 139546163 (55.907639ms) 17.886642 GB/s
// 
// --- Cache Test: 131072KB, 7B unaligned ---
// Min: 130811515 (52.408198ms) 19.080983 GB/s                                       
// Max: 173303148 (69.432004ms) 14.402579 GB/s
// Avg: 140373185 (56.238976ms) 17.781262 GB/s
// 
// --- Cache Test: 131072KB, 15B unaligned ---
// Min: 128635002 (51.536202ms) 19.403835 GB/s                                       
// Max: 153692784 (61.575327ms) 16.240270 GB/s
// Avg: 139367692 (55.836136ms) 17.909548 GB/s
// 
// --- Cache Test: 131072KB, 31B unaligned ---
// Min: 131113423 (52.529154ms) 19.037046 GB/s                                       
// Max: 158141536 (63.357671ms) 15.783408 GB/s
// Avg: 139315591 (55.815263ms) 17.916245 GB/s
// 
// --- Cache Test: 262144KB, 0B unaligned ---
// Min: 139325426 (55.819203ms) 17.914981 GB/s                                       
// Max: 168440794 (67.483956ms) 14.818336 GB/s
// Avg: 148442294 (59.471776ms) 16.814698 GB/s
// 
// --- Cache Test: 262144KB, 1B unaligned ---
// Min: 131074612 (52.513605ms) 19.042683 GB/s                                       
// Max: 159331296 (63.834335ms) 15.665550 GB/s
// Avg: 140269107 (56.197278ms) 17.794455 GB/s
// 
// --- Cache Test: 262144KB, 3B unaligned ---
// Min: 122099379 (48.917776ms) 20.442465 GB/s                                       
// Max: 187182384 (74.992568ms) 13.334654 GB/s
// Avg: 141581756 (56.723177ms) 17.629477 GB/s
// 
// --- Cache Test: 262144KB, 7B unaligned ---
// Min: 130215770 (52.169519ms) 19.168280 GB/s                                       
// Max: 150337587 (60.231104ms) 16.602716 GB/s
// Avg: 140641806 (56.346596ms) 17.747300 GB/s
// 
// --- Cache Test: 262144KB, 15B unaligned ---
// Min: 136348307 (54.626453ms) 18.306148 GB/s                                       
// Max: 153459982 (61.482058ms) 16.264907 GB/s
// Avg: 140289836 (56.205583ms) 17.791826 GB/s
// 
// --- Cache Test: 262144KB, 31B unaligned ---
// Min: 132482288 (53.077575ms) 18.840347 GB/s                                       
// Max: 178544322 (71.531823ms) 13.979791 GB/s
// Avg: 141088307 (56.525482ms) 17.691135 GB/s
// 
// --- Cache Test: 524288KB, 0B unaligned ---
// Min: 137574682 (55.117787ms) 18.142963 GB/s                                       
// Max: 208376060 (83.483582ms) 11.978402 GB/s
// Avg: 151875142 (60.847109ms) 16.434634 GB/s
// 
// --- Cache Test: 524288KB, 1B unaligned ---
// Min: 129528258 (51.894075ms) 19.270021 GB/s                                       
// Max: 157484626 (63.094487ms) 15.849244 GB/s
// Avg: 140619602 (56.337700ms) 17.750102 GB/s
// 
// --- Cache Test: 524288KB, 3B unaligned ---
// Min: 130688294 (52.358831ms) 19.098974 GB/s                                       
// Max: 160194685 (64.180243ms) 15.581118 GB/s
// Avg: 140108303 (56.132854ms) 17.814878 GB/s
// 
// --- Cache Test: 524288KB, 7B unaligned ---
// Min: 131815569 (52.810461ms) 18.935641 GB/s                                       
// Max: 163253930 (65.405895ms) 15.289141 GB/s
// Avg: 140727255 (56.380830ms) 17.736524 GB/s
// 
// --- Cache Test: 524288KB, 15B unaligned ---
// Min: 131269240 (52.591580ms) 19.014449 GB/s                                       
// Max: 158013489 (63.306370ms) 15.796198 GB/s
// Avg: 140589563 (56.325666ms) 17.753895 GB/s
// 
// --- Cache Test: 524288KB, 31B unaligned ---
// Min: 132111109 (52.928866ms) 18.893281 GB/s                                       
// Max: 152669494 (61.165357ms) 16.349123 GB/s
// Avg: 140238641 (56.185072ms) 17.798321 GB/s
// 
// --- Cache Test: 1048576KB, 0B unaligned ---
// Min: 136197087 (54.565868ms) 18.326474 GB/s                                       
// Max: 226932874 (90.918166ms) 10.998901 GB/s
// Avg: 150022427 (60.104839ms) 16.637595 GB/s
// 
// --- Cache Test: 1048576KB, 1B unaligned ---
// Min: 120474900 (48.266947ms) 20.718111 GB/s                                       
// Max: 159047684 (63.720709ms) 15.693484 GB/s
// Avg: 140378128 (56.240956ms) 17.780635 GB/s
// 
// --- Cache Test: 1048576KB, 3B unaligned ---
// Min: 134302490 (53.806819ms) 18.585004 GB/s                                       
// Max: 170289668 (68.224687ms) 14.657450 GB/s
// Avg: 142168548 (56.958269ms) 17.556712 GB/s
// 
// --- Cache Test: 1048576KB, 7B unaligned ---
// Min: 133788012 (53.600699ms) 18.656472 GB/s                                       
// Max: 156712534 (62.785157ms) 15.927330 GB/s
// Avg: 141457757 (56.673498ms) 17.644931 GB/s
// 
// --- Cache Test: 1048576KB, 15B unaligned ---
// Min: 134237013 (53.780586ms) 18.594069 GB/s                                       
// Max: 162498453 (65.103222ms) 15.360222 GB/s
// Avg: 141380622 (56.642595ms) 17.654557 GB/s
// 
// --- Cache Test: 1048576KB, 31B unaligned ---
// Min: 132571191 (53.113193ms) 18.827713 GB/s                                       
// Max: 153476109 (61.488519ms) 16.263198 GB/s
// Avg: 141322797 (56.619428ms) 17.661781 GB/s
// */