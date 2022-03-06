#![feature(test)]

extern crate crc32c;
extern crate twox_hash;

extern crate rand;

extern crate rand_chacha;
extern crate rand_pcg;
extern crate rand_xoshiro;
extern crate rand_xorshift;

extern crate rayon;
use rayon::prelude::*;

extern crate test;

extern crate region;

extern crate rsmemtest;

use rsmemtest::*;

use test::Bencher;

use rand::prelude::*;
use std::hash::Hasher;



fn alloc_data() -> Vec<u8> {
    let len = 128 * 1024 * 1024;
    let data = unsafe {
        let raw = std::alloc::alloc(
            std::alloc::Layout::from_size_align(len, /*align*/4096).unwrap());
        Vec::from_raw_parts(raw, len, len)
    };
    region::lock(data.as_ptr(), data.len());

    data
}

#[bench]
fn bench_crc32c(b: &mut Bencher) {
    let data = alloc_data();
    b.iter(|| crc32c::crc32c(&data));
}


/*
data= 128 MiB
test bench_crc32c   ... bench:  14,863,724 ns/iter (+/- 1,038,262) = 8612 MiB/s
test bench_sha1     ... bench:  61,714,323 ns/iter (+/- 980,515)   = 2074 MiB/s
test bench_xxhash32       ... bench:  16,691,250 ns/iter (+/- 758,516) = 8041 MB/s
test bench_xxhash64       ... bench:  18,594,281 ns/iter (+/- 771,065) = 7218 MB/s
*/

#[bench]
fn bench_xxhash32(b: &mut Bencher) {
    let data = alloc_data();
    let mut hasher = twox_hash::XxHash32::with_seed(0);
    b.iter(|| { hasher.write(&data); hasher.finish(); });
    b.bytes = data.len() as u64;
}

#[bench]
fn bench_xxhash64(b: &mut Bencher) {
    let data = alloc_data();
    let mut hasher = twox_hash::XxHash64::with_seed(0);
    b.iter(|| { hasher.write(&data); hasher.finish(); });
    b.bytes = data.len() as u64;
}


/*
  size = 512 * 4096 = 2 MiB
test bench_chacharng      ... bench:     648,446 ns/iter (+/- 65,869) = 3084 MiB/s <<<>>>
test bench_osrng         ... bench:  26,423,819 ns/iter (+/- 725,452)
test bench_pcg32rng       ... bench:   1,039,964 ns/iter (+/- 23,381)
test bench_pcg64mcgrng    ... bench:     710,057 ns/iter (+/- 22,543) = 2817 MiB/s 
test bench_pcg64rng       ... bench:     773,072 ns/iter (+/- 15,486)
test bench_smallrng       ... bench:     720,100 ns/iter (+/- 11,437)
test bench_threadrng      ... bench:   1,424,262 ns/iter (+/- 40,097)
test bench_xorshiftprng   ... bench:     810,815 ns/iter (+/- 24,803)
test bench_xoshiro128prng ... bench:     921,302 ns/iter (+/- 11,729)
test bench_xoshiro128rng  ... bench:   1,033,120 ns/iter (+/- 28,188)
test bench_xoshiro256prng ... bench:     740,517 ns/iter (+/- 15,788)
test bench_xoshiro256rng  ... bench:     794,332 ns/iter (+/- 25,792)
test bench_xoshiroprng    ... bench:     760,506 ns/iter (+/- 20,453)
test bench_xoshirorng     ... bench:     785,893 ns/iter (+/- 23,302)
*/

/*

#[bench]
fn bench_threadrng(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut ys = vec![0u8; 512 * 4096];
    b.iter(|| { rng.fill_bytes(&mut ys); });
}


#[bench]
fn bench_smallrng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = SmallRng::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

*/

#[bench]
fn bench_chacharng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = rand_chacha::ChaCha8Rng::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

#[bench]
fn bench_xoshirorng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = rand_xoshiro::Xoshiro512StarStar::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

#[bench]
fn bench_xoshiro256rng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = rand_xoshiro::Xoshiro256StarStar::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

#[bench]
fn bench_xoshiro128rng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = rand_xoshiro::Xoshiro128StarStar::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

#[bench]
fn bench_xoshiroprng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = rand_xoshiro::Xoshiro512Plus::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

#[bench]
fn bench_xoshiro256prng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = rand_xoshiro::Xoshiro256Plus::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

#[bench]
fn bench_xoshiro128prng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = rand_xoshiro::Xoshiro128Plus::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

#[bench]
fn bench_xorshiftprng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = rand_xorshift::XorShiftRng::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

#[bench]
fn bench_pcg32rng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = rand_pcg::Pcg32::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

#[bench]
fn bench_pcg64rng(b: &mut Bencher) {
    let mut ys = vec![0u8; 512 * 4096];
    let mut small_rng = rand_pcg::Pcg64::from_entropy();
    b.iter(|| { small_rng.fill_bytes(&mut ys); });
}

/*
Parallel size = 516 MiB

>> Select Pcg64Mcg <<

test bench_pcg64mcgrayon  ... bench:  29,195,637 ns/iter (+/- 1,897,304) = 18532 MB/s
test bench_pcg64mcgrng    ... bench:  86,884,543 ns/iter (+/- 1,643,448) = 6227 MB/s

test bench_xoshiro256pgrayon  ... bench:  29,406,300 ns/iter (+/- 1,694,286) = 18399 MB/s
test bench_xoshiro256prng    ... bench:  86,546,453 ns/iter (+/- 2,361,028) = 6251 MB/s

test bench_chacha8rayon  ... bench:  36,456,352 ns/iter (+/- 2,252,792) = 14841 MB/s
test bench_chacha8rng    ... bench: 171,459,742 ns/iter (+/- 1,815,879) = 3155 MB/s

test bench_aesrng_memfill ... bench:  28,996,198 ns/iter (+/- 1,215,452) = 18659 MB/s

*/

fn pcg64mcg_fill_bytes(xs: &mut [u8]) {
    let mut rng = rand_pcg::Pcg64Mcg::from_rng(rand::thread_rng()).unwrap();
    //let mut rng = rand_xoshiro::Xoshiro512Plus::from_rng(rand::thread_rng()).unwrap();
    //let mut rng = rand_chacha::ChaCha8Rng::from_rng(rand::thread_rng()).unwrap();
    rng.fill_bytes(xs);
}

#[bench]
fn bench_pcg64mcgrng(b: &mut Bencher) {
    let len = 1024 * 1024 * 516;
    let mut ys = vec![0u8; len];
    println!("{:?}", &ys[0] as *const u8);
    b.iter(|| { pcg64mcg_fill_bytes(&mut ys); });
    b.bytes = len as u64;
}

#[bench]
fn bench_pcg64mcgrayon(b: &mut Bencher) {
    //let mut ys = vec![[0u8; 1024 * 1024]; 512 * (1024 / 1024)];
    let len = 1024 * 1024 * 516;
    //let mut ys = vec![0u8; len];
    let mut ys = unsafe {
        let raw = std::alloc::alloc(
            std::alloc::Layout::from_size_align(len, /*align*/4096).unwrap());
        Vec::from_raw_parts(raw, len, len)
    };

    println!("{:?}", &ys[0] as *const u8);

    b.iter(|| { ys.par_chunks_mut(len/12).for_each(|x| pcg64mcg_fill_bytes(x) ); });
    b.bytes = len as u64;
}

#[bench]
fn bench_testop_rayon(b: &mut Bencher) {
    let len = 516 * 1024 * 1024;
    let num_blocks = len / std::mem::size_of::<TestBlock>();

    let mut v: Vec<TestBlock> = Vec::with_capacity(num_blocks);
    unsafe { v.set_len(num_blocks); }

    b.iter(|| { v.par_chunks_mut(num_blocks/12).for_each(testop) });
    b.bytes = len as u64;
}
