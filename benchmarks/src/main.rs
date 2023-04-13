use webgraph::codes::*;
use rand::Rng;

/// How many random codes we will write and read in the benchmark
const VALUES: usize = 10_000;
/// How many iterations to do before starting measuring, this is done to warmup
/// the caches and the branch predictor
const WARMUP_ITERS: usize = 100;
/// How many iterations of measurement we will execute
const BENCH_ITERS: usize = 1_000;
/// For how many times we will measure the measurement overhead
const CALIBRATION_ITERS: usize = 1_000_000;

#[cfg(feature = "rtdsc")]
mod x86_64 {
    pub struct Instant(u64);
    
    impl Instant {
        #[inline(always)]
        fn now() -> Self {
            Self(rdtsc())
        }
    
        fn elapsed(&self) -> Duration {
            Duration(rdtsc() - self.0)
        }
    }
    
    pub struct Duration(u64);

    impl Duration {
        fn as_nanos(&self) -> u128 {
            /// The TimeStampCounter frequency in Hertz. 
            /// find tsc freq with `dmesg | grep tsc` or `journalctl | grep tsc` 
            /// and convert it to hertz
            const TSC_FREQ: u128 = 3_609_600_000;
            const TO_NS: u128 = 1_000_000_000;
            self.0 as u128 * TO_NS / TSC_FREQ
        }
    }
    
    #[inline(always)]
    fn rdtsc() -> u64 {
        
        use core::arch::x86_64::{
            __rdtscp, __cpuid, 
            _mm_lfence, _mm_mfence, _mm_sfence
        };
        
        unsafe{
            let mut aux: u32 = 0;
            let _ = __cpuid(0);
            let _ = _mm_lfence();
            let _ = _mm_mfence();
            let _ = _mm_sfence();
            __rdtscp(&mut aux as *mut u32)
        }
    }
}

#[cfg(feature = "rtdsc")]
use x86_64::*;

#[cfg(not(feature = "rtdsc"))]
use std::time::Instant;

/// Structure to compute statistics from a stream
struct MetricsStream {
    min: f64,
    max: f64,
    avg: f64,
    m2: f64,
    count: usize,
}

#[derive(Debug)]
/// The result of [`MetricStream`]
struct Metrics {
    min: f64,
    max: f64,
    avg: f64,
    std: f64,
}

impl Default for MetricsStream {
    fn default() -> Self {
        MetricsStream {
            max: f64::NEG_INFINITY,
            min: f64::INFINITY,
            avg: 0.0,
            m2: 0.0,
            count: 0,
        }
    }
}

impl MetricsStream {
    /// Ingest a value from the stream
    fn update(&mut self, value: f64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);

        // Welford algorithm 
        // https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance
        self.count += 1;
        let delta = value - self.avg;
        self.avg += delta / self.count as f64;
        let delta2 = value - self.avg;
        self.m2 += delta * delta2;
    }

    /// Consume this builder to get the statistics
    fn finalize(self) -> Metrics {
        if self.count < 2 {
            panic!();
        }
        let var = self.m2 / (self.count - 1) as f64;
        Metrics {
            min: self.min ,
            max: self.max,
            avg: self.avg,
            std: var.sqrt(),
        }
    }
}

/// Routine for measuring the measurement overhead.
fn calibrate_overhead() -> u128 {
    let mut nanos = MetricsStream::default();
    // For many times, measure an empty block 
    for _ in 0..CALIBRATION_ITERS {
        let start = Instant::now();
        let delta = start.elapsed().as_nanos();
        nanos.update(delta as f64);
    }
    let measures = nanos.finalize();
    eprintln!("Timesource calibration is: {:#4?}", measures);
    measures.avg as u128
}

/// Pin the process to one core to avoid context switching and caches flushes
/// which would result in noise in the measurement.
#[cfg(target_os="linux")]
fn pin_to_core(core_id: usize) {
    unsafe{
        let mut cpu_set = core::mem::MaybeUninit::zeroed().assume_init();
        libc::CPU_ZERO(&mut cpu_set);
        libc::CPU_SET(core_id, &mut cpu_set);
        let res = libc::sched_setaffinity(
            libc::getpid(), 
            core::mem::size_of::<libc::cpu_set_t>(), 
            &cpu_set as *const libc::cpu_set_t,
        );
        assert_ne!(res, -1);
    }    
}

macro_rules! bench {
    ($cal:expr, $ratio:expr, $code:literal, $read:ident, $write:ident, $data:expr, $bo:ident, $table:expr) => {{
// the memory where we will write values
let mut buffer = Vec::with_capacity(VALUES);
// counters for the total read time and total write time
let mut read_buff = MetricsStream::default();
let mut read_unbuff = MetricsStream::default();
let mut write = MetricsStream::default();

// measure
for iter in 0..(WARMUP_ITERS + BENCH_ITERS) {
    buffer.clear();
    // write the codes
    {   
        // init the writer
        let mut r = BufferedBitStreamWrite::<$bo, _>::new(
            MemWordWriteVec::new(&mut buffer)
        );
        // measure
        let w_start = Instant::now();
        for value in &$data {
            r.$write::<$table>(*value).unwrap();
        }
        let nanos = w_start.elapsed().as_nanos();
        // add the measurement if we are not in the warmup
        if iter >= WARMUP_ITERS {
            write.update((nanos - $cal) as f64);
        }
    }
    // read the codes
    {
        // init the reader
        let mut r = BufferedBitStreamRead::<$bo, _>::new(
            MemWordRead::new(&mut buffer)
        );
        // measure
        let r_start = Instant::now();
        for _ in &$data {
            r.$read::<$table>().unwrap();
        }
        let nanos =  r_start.elapsed().as_nanos();
        // add the measurement if we are not in the warmup
        if iter >= WARMUP_ITERS {
            read_buff.update((nanos - $cal) as f64);
        }
    }
    {
        // init the reader
        let mut r = UnbufferedBitStreamRead::<$bo, _>::new(
            MemWordRead::new(&mut buffer)
        );
        // measure
        let r_start = Instant::now();
        for _ in &$data {
            r.$read::<$table>().unwrap();
        }
        let nanos =  r_start.elapsed().as_nanos();
        // add the measurement if we are not in the warmup
        if iter >= WARMUP_ITERS {
            read_unbuff.update((nanos - $cal) as f64);
        }
    }
}
// convert from cycles to nano seconds
let read_buff = read_buff.finalize();
let read_unbuff = read_unbuff.finalize();
let write = write.finalize();

let bytes = buffer.len() * 8;
let table = if $table {
    "Table"
} else {
    "NoTable"
};
// print the results
println!("{}::{}::{},{},{},{},{},{},{},{}",
    $code, stringify!($bo), table, // the informations about what we are benchmarking
    "write",
    $ratio,
    bytes,
    write.avg / VALUES as f64, 
    write.std / VALUES as f64,
    write.max / VALUES as f64, 
    write.min / VALUES as f64,
);
println!("{}::{}::{},{},{},{},{},{},{},{}",
    $code, stringify!($bo), table, // the informations about what we are benchmarking
    "read_buff",
    $ratio,
    bytes,
    read_buff.avg / VALUES as f64, 
    read_buff.std / VALUES as f64,
    read_buff.max / VALUES as f64, 
    read_buff.min / VALUES as f64,
);
println!("{}::{}::{},{},{},{},{},{},{},{}",
    $code, stringify!($bo), table, // the informations about what we are benchmarking
    "read_unbuff",
    $ratio,
    bytes,
    read_unbuff.avg / VALUES as f64, 
    read_unbuff.std / VALUES as f64,
    read_unbuff.max / VALUES as f64, 
    read_unbuff.min / VALUES as f64,
);

}};
}

/// macro to implement all combinations of bit order and table use
macro_rules! impl_code {
    ($cal:expr, $ratio:expr, $code:literal, $read:ident, $write:ident, $data:expr) => {
        bench!(
            $cal, $ratio, $code, $read, $write, $data, M2L, false
        );
        bench!(
            $cal, $ratio, $code, $read, $write, $data, M2L, true
        );
        bench!(
            $cal, $ratio, $code, $read, $write, $data, L2M, false
        );
        bench!(
            $cal, $ratio, $code, $read, $write, $data, L2M, true
        );
    };
}

pub fn main() {
    // tricks to reduce the noise
    #[cfg(target_os="linux")]
    pin_to_core(5);
    //unsafe{assert_ne!(libc::nice(-20-libc::nice(0)), -1);}
    
    // figure out how much overhead we add by measuring
    let calibration = calibrate_overhead();
    // print the header of the csv
    println!("pat,type,ratio,bytes,ns_avg,ns_std,ns_max,ns_min");

    // benchmark the buffered impl
    let mut rng = rand::thread_rng();
    
    let unary_data = (0..VALUES)
        .map(|_| {
            let v: u64 = rng.gen();
            v.trailing_zeros() as u64
        })
        .collect::<Vec<_>>();

    let ratio = unary_data.iter().map(|value| {
        if len_unary::<false>(*value) <= unary_tables::READ_BITS as usize {
            1
        } else {
            0
        }
    }).sum::<usize>() as f64 / VALUES as f64;

    impl_code!(
        calibration, ratio, "unary", read_unary, write_unary, unary_data
    );

    let gamma_data = (0..VALUES)
        .map(|_| {
            rng.sample(rand_distr::Zeta::new(2.0).unwrap()) as u64 - 1
        })
        .collect::<Vec<_>>();

    let ratio = gamma_data.iter().map(|value| {
        if len_gamma::<false>(*value) <= gamma_tables::READ_BITS as usize {
            1
        } else {
            0
        }
    }).sum::<usize>() as f64 / VALUES as f64;

    impl_code!(
        calibration, ratio, "gamma", read_gamma, write_gamma, gamma_data
    );

    let delta_data = (0..VALUES)
        .map(|_| {
            rng.sample(rand_distr::Zeta::new(1.01).unwrap()) as u64 - 1
        })
        .collect::<Vec<_>>();

    let ratio = delta_data.iter().map(|value| {
        if len_delta::<false>(*value) <= delta_tables::READ_BITS as usize {
            1
        } else {
            0
        }
    }).sum::<usize>() as f64 / VALUES as f64;

    impl_code!(
        calibration, ratio, "delta", read_delta, write_delta, delta_data
    );

    let zeta3_data = (0..VALUES)
        .map(|_| {
            rng.sample(rand_distr::Zeta::new(1.2).unwrap()) as u64 - 1
        })
        .collect::<Vec<_>>();

    let ratio = zeta3_data.iter().map(|value| {
        if len_zeta::<false>(*value, 3) <= zeta_tables::READ_BITS as usize {
            1
        } else {
            0
        }
    }).sum::<usize>() as f64 / VALUES as f64;

    impl_code!(
        calibration, ratio, "zeta", read_zeta3, write_zeta3, zeta3_data
    );
}