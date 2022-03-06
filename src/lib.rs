extern crate crossbeam;

extern crate raw_cpuid;
extern crate rand;
extern crate rand_pcg;
extern crate twox_hash;

use rand::prelude::*;
use std::sync::atomic::{fence, Ordering};
use std::mem::size_of;

use std::arch::x86_64::*;

mod aespcbc;
use aespcbc::Aes128Pcbc;
use crossbeam::channel::Sender;


#[repr(align(4096))]
pub struct TestBlock([__m128i; 8192 / size_of::<__m128i>()]);

pub enum TesterMessage {
    CoveredBytes(usize),
    FoundError,
}


#[inline(always)]
fn flush_cache<T: Sized>(vaddr: *mut T, len: usize) {
    const CLFLUSH_CACHE_LINE_SIZE: usize = 64;

    let vaddr_u8 = vaddr as *mut u8;
    let raw_len = len * size_of::<T>();

    fence(Ordering::SeqCst);
    for i in (0..raw_len-1).step_by(CLFLUSH_CACHE_LINE_SIZE) {
        unsafe { _mm_clflush(vaddr_u8.offset(i as isize)); }
    }
    fence(Ordering::SeqCst);
}

fn test_encrypt_round(blocks: &mut [TestBlock], key: &__m128i, nonce: &__m128i) -> __m128i {
    let mut cipher = Aes128Pcbc::new(key, nonce);

    for block in blocks {
        for x in block.0.iter_mut() {
            *x = cipher.encrypt(*x);
        }

        flush_cache(block as *mut TestBlock, 1);
    }

    cipher.nonce()
}

fn test_decrypt_round(blocks: &mut [TestBlock], key: &__m128i, nonce: &__m128i, check: bool) -> bool {
    let mut cipher = Aes128Pcbc::new(key, nonce);

    for block in blocks {
        for x in block.0.iter_mut() {
            *x = cipher.decrypt(*x);

            if check {
                let x_allzero = unsafe { _mm_testz_si128(*x, *x) };
                if x_allzero != 1 {
                    return false;
                }
            }
        }

        flush_cache(block as *mut TestBlock, 1);
    }

    true
}

pub fn test_thread(blocks: &mut [TestBlock], tx: Sender<TesterMessage>) {
    let mut rng = rand::thread_rng();
    let num_bytes = blocks.len() * size_of::<TestBlock>();

    const NUM_TEST_ITERS: usize = 4;

    loop {
        let keys: Vec<__m128i> = (&mut rng).sample_iter(rand::distributions::Standard).take(NUM_TEST_ITERS).collect();
        let mut nonces: Vec<__m128i> = vec![(&mut rng).sample(rand::distributions::Standard)];

        for key in &keys {
            let nonce = test_encrypt_round(blocks, key, nonces.last().unwrap());
            nonces.push(nonce);
            tx.send(TesterMessage::CoveredBytes(num_bytes / (NUM_TEST_ITERS * 2))).unwrap();
        }

        for (i, (key, nonce)) in keys.iter().zip(nonces.iter()).enumerate().rev() {
            if !test_decrypt_round(blocks, key, nonce, i == 0) {
                tx.send(TesterMessage::FoundError).unwrap();
            }
            tx.send(TesterMessage::CoveredBytes(num_bytes / (NUM_TEST_ITERS * 2))).unwrap();
        }

        flush_cache(&mut blocks[0] as *mut TestBlock, blocks.len());
    }
}
