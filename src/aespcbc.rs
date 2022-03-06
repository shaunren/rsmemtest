use std::arch::x86_64::*;
use std::mem::MaybeUninit;

#[derive(Copy, Clone)]
pub struct Aes128Pcbc {
    encrypt_keys: [__m128i; 6],
    decrypt_keys: [__m128i; 6],
    nonce: __m128i,
}

macro_rules! expand_round {
    ($enc_keys:expr, $dec_keys:expr, $pos:expr, $round:expr) => {
        let mut t1 = _mm_load_si128($enc_keys.as_ptr().offset($pos - 1));
        let mut t2;
        let mut t3;

        t2 = _mm_aeskeygenassist_si128(t1, $round);
        t2 = _mm_shuffle_epi32(t2, 0xff);
        t3 = _mm_slli_si128(t1, 0x4);
        t1 = _mm_xor_si128(t1, t3);
        t3 = _mm_slli_si128(t3, 0x4);
        t1 = _mm_xor_si128(t1, t3);
        t3 = _mm_slli_si128(t3, 0x4);
        t1 = _mm_xor_si128(t1, t3);
        t1 = _mm_xor_si128(t1, t2);

        _mm_store_si128($enc_keys.as_mut_ptr().offset($pos), t1);
        let t1 = if $pos != 5 { _mm_aesimc_si128(t1) } else { t1 };
        _mm_store_si128($dec_keys.as_mut_ptr().offset($pos), t1);
    }
}

#[inline(always)]
fn expand(key: &__m128i) -> ([__m128i; 6], [__m128i; 6]) {
    unsafe {
        let mut enc_keys: [__m128i; 6] = MaybeUninit::uninit().assume_init();
        let mut dec_keys: [__m128i; 6] = MaybeUninit::uninit().assume_init();

        enc_keys[0] = *key;
        dec_keys[0] = *key;

        expand_round!(enc_keys, dec_keys, 1, 1);
        expand_round!(enc_keys, dec_keys, 2, 1<<1);
        expand_round!(enc_keys, dec_keys, 3, 1<<2);
        expand_round!(enc_keys, dec_keys, 4, 1<<3);
        expand_round!(enc_keys, dec_keys, 5, 1<<4);

        (enc_keys, dec_keys)
    }
}

impl Aes128Pcbc {
    #[inline]
    pub fn new(key: &__m128i, nonce: &__m128i) -> Self {
        let (encrypt_keys, decrypt_keys) = expand(key);

        Self {
            encrypt_keys: encrypt_keys,
            decrypt_keys: decrypt_keys,
            nonce: *nonce,
        }
    }

    #[inline(always)]
    pub fn encrypt(&mut self,  plaintext: __m128i) -> __m128i {
        let keys = self.encrypt_keys;
        unsafe {
            let mut b = plaintext;

            b = _mm_xor_si128(b, self.nonce);

            b = _mm_xor_si128(b, keys[0]);
            b = _mm_aesenc_si128(b, keys[1]);
            b = _mm_aesenc_si128(b, keys[2]);
            b = _mm_aesenc_si128(b, keys[3]);
            b = _mm_aesenc_si128(b, keys[4]);
            b = _mm_aesenclast_si128(b, keys[5]);

            self.nonce = _mm_xor_si128(b, plaintext);

            b
        }
    }

    #[inline(always)]
    pub fn decrypt(&mut self, ciphertext: __m128i) -> __m128i {
        let keys = self.decrypt_keys;
        unsafe {
            let mut b = ciphertext;

            b = _mm_xor_si128(b, keys[5]);
            b = _mm_aesdec_si128(b, keys[4]);
            b = _mm_aesdec_si128(b, keys[3]);
            b = _mm_aesdec_si128(b, keys[2]);
            b = _mm_aesdec_si128(b, keys[1]);
            b = _mm_aesdeclast_si128(b, keys[0]);

            b = _mm_xor_si128(b, self.nonce);
            self.nonce = _mm_xor_si128(b, ciphertext);

            b
        }
    }

    #[inline]
    pub fn nonce(&self) -> __m128i {
        self.nonce
    }
}
