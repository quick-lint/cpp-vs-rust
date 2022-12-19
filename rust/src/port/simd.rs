#[allow(unused)]
use crate::qljs_assert;
#[allow(unused)]
use crate::util::array::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
#[cfg(target_arch = "arm")]
use std::arch::arm::*;
#[cfg(target_arch = "wasm32")]
use std::arch::wasm32::*;
#[cfg(target_arch = "wasm64")]
use std::arch::wasm64::*;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_feature = "simd128")]
#[derive(Clone, Copy)]
pub struct CharVector16WASMSIMD128(v128);

#[cfg(target_feature = "simd128")]
impl CharVector16WASMSIMD128 {
    // data must point to at least 16 elements.
    #[inline(always)]
    pub fn load(data: &[u8]) -> CharVector16WASMSIMD128 {
        qljs_assert!(data.len() >= 16);
        unsafe { CharVector16WASMSIMD128(v128_load(data.as_ptr() as *const v128)) }
    }

    // data must point to at least 16 elements.
    #[inline(always)]
    pub unsafe fn load_raw(data: *const u8) -> CharVector16WASMSIMD128 {
        Self::load(std::slice::from_raw_parts(data, 16))
    }

    // out_data must point to at least 16 elements.
    #[inline(always)]
    pub fn store(&self, out_data: &mut [u8]) {
        qljs_assert!(out_data.len() >= 16);
        unsafe {
            v128_store(out_data.as_mut_ptr() as *mut v128, self.0);
        }
    }

    #[inline(always)]
    pub fn repeated(x: u8) -> CharVector16WASMSIMD128 {
        CharVector16WASMSIMD128(u8x16_splat(x))
    }

    #[inline(always)]
    pub fn lane_eq(&self, rhs: CharVector16WASMSIMD128) -> BoolVector16WASMSIMD128 {
        BoolVector16WASMSIMD128(i8x16_eq(self.0, rhs.0))
    }

    #[inline(always)]
    pub fn lane_lt(&self, rhs: CharVector16WASMSIMD128) -> BoolVector16WASMSIMD128 {
        BoolVector16WASMSIMD128(u8x16_lt(self.0, rhs.0))
    }

    #[inline(always)]
    pub fn lane_gt(&self, rhs: CharVector16WASMSIMD128) -> BoolVector16WASMSIMD128 {
        BoolVector16WASMSIMD128(u8x16_gt(self.0, rhs.0))
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        16
    }
}

#[cfg(target_feature = "simd128")]
impl std::ops::BitOr<CharVector16WASMSIMD128> for CharVector16WASMSIMD128 {
    type Output = CharVector16WASMSIMD128;

    #[inline(always)]
    fn bitor(self, rhs: CharVector16WASMSIMD128) -> CharVector16WASMSIMD128 {
        CharVector16WASMSIMD128(v128_or(self.0, rhs.0))
    }
}

#[cfg(target_feature = "simd128")]
#[derive(Clone, Copy, Debug)]
pub struct BoolVector16WASMSIMD128(v128);

#[cfg(target_feature = "simd128")]
impl BoolVector16WASMSIMD128 {
    // data must point to at least 16 elements.
    #[inline(always)]
    pub fn load_slow(data: &[bool]) -> BoolVector16WASMSIMD128 {
        qljs_assert!(data.len() >= 16);
        let bytes: [u8; 16] = generate_array_n(|i: usize| {
            if unsafe { *data.get_unchecked(i) } {
                0xff
            } else {
                0x00
            }
        });
        unsafe { BoolVector16WASMSIMD128(v128_load(bytes.as_ptr() as *const v128)) }
    }

    #[inline(always)]
    pub fn find_first_false(&self) -> u32 {
        self.mask().trailing_ones()
    }

    #[inline(always)]
    pub fn mask(&self) -> u32 {
        i8x16_bitmask(self.0) as u32
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        16
    }
}

#[cfg(target_feature = "simd128")]
impl std::ops::BitAnd<BoolVector16WASMSIMD128> for BoolVector16WASMSIMD128 {
    type Output = BoolVector16WASMSIMD128;

    #[inline(always)]
    fn bitand(self, rhs: BoolVector16WASMSIMD128) -> BoolVector16WASMSIMD128 {
        BoolVector16WASMSIMD128(v128_and(self.0, rhs.0))
    }
}

#[cfg(target_feature = "simd128")]
impl std::ops::BitOr<BoolVector16WASMSIMD128> for BoolVector16WASMSIMD128 {
    type Output = BoolVector16WASMSIMD128;

    #[inline(always)]
    fn bitor(self, rhs: BoolVector16WASMSIMD128) -> BoolVector16WASMSIMD128 {
        BoolVector16WASMSIMD128(v128_or(self.0, rhs.0))
    }
}

#[cfg(target_feature = "sse2")]
#[derive(Clone, Copy)]
pub struct CharVector16SSE2(__m128i);

#[cfg(target_feature = "sse2")]
impl CharVector16SSE2 {
    // data must point to at least 16 elements.
    #[inline(always)]
    pub fn load(data: &[u8]) -> CharVector16SSE2 {
        qljs_assert!(data.len() >= 16);
        unsafe { CharVector16SSE2(_mm_loadu_si128(data.as_ptr() as *const __m128i)) }
    }

    // data must point to at least 16 elements.
    #[inline(always)]
    pub unsafe fn load_raw(data: *const u8) -> CharVector16SSE2 {
        Self::load(std::slice::from_raw_parts(data, 16))
    }

    #[inline(always)]
    pub fn repeated(x: u8) -> CharVector16SSE2 {
        unsafe { CharVector16SSE2(_mm_set1_epi8(x as i8)) }
    }

    // out_data must point to at least 16 elements.
    #[inline(always)]
    pub fn store(&self, out_data: &mut [u8]) {
        qljs_assert!(out_data.len() >= 16);
        unsafe {
            _mm_storeu_si128(out_data.as_mut_ptr() as *mut __m128i, self.0);
        }
    }

    #[inline(always)]
    pub fn lane_eq(&self, rhs: CharVector16SSE2) -> BoolVector16SSE2 {
        unsafe { BoolVector16SSE2(_mm_cmpeq_epi8(self.0, rhs.0)) }
    }

    #[inline(always)]
    pub fn lane_lt(&self, rhs: CharVector16SSE2) -> BoolVector16SSE2 {
        unsafe { BoolVector16SSE2(_mm_cmplt_epi8(self.0, rhs.0)) }
    }

    #[inline(always)]
    pub fn lane_gt(&self, rhs: CharVector16SSE2) -> BoolVector16SSE2 {
        unsafe { BoolVector16SSE2(_mm_cmpgt_epi8(self.0, rhs.0)) }
    }

    #[inline(always)]
    pub fn m128i(&self) -> __m128i {
        self.0
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        16
    }
}

#[cfg(target_feature = "sse2")]
impl std::ops::BitOr<CharVector16SSE2> for CharVector16SSE2 {
    type Output = CharVector16SSE2;

    #[inline(always)]
    fn bitor(self, rhs: CharVector16SSE2) -> CharVector16SSE2 {
        unsafe { CharVector16SSE2(_mm_or_si128(self.0, rhs.0)) }
    }
}

#[cfg(target_feature = "sse2")]
#[derive(Clone, Copy, Debug)]
pub struct BoolVector16SSE2(__m128i);

#[cfg(target_feature = "sse2")]
impl BoolVector16SSE2 {
    // data must point to at least 16 elements.
    #[inline(always)]
    pub fn load_slow(data: &[bool]) -> BoolVector16SSE2 {
        qljs_assert!(data.len() >= 16);
        let bytes: [u8; 16] = generate_array_n(|i: usize| {
            if unsafe { *data.get_unchecked(i) } {
                0xff
            } else {
                0x00
            }
        });
        unsafe { BoolVector16SSE2(_mm_loadu_si128(bytes.as_ptr() as *const __m128i)) }
    }

    #[inline(always)]
    pub fn find_first_false(&self) -> u32 {
        self.mask().trailing_ones()
    }

    #[inline(always)]
    pub fn mask(&self) -> u32 {
        unsafe { _mm_movemask_epi8(self.0) as u32 }
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        16
    }
}

#[cfg(target_feature = "sse2")]
impl std::ops::BitAnd<BoolVector16SSE2> for BoolVector16SSE2 {
    type Output = BoolVector16SSE2;

    #[inline(always)]
    fn bitand(self, rhs: BoolVector16SSE2) -> BoolVector16SSE2 {
        unsafe { BoolVector16SSE2(_mm_and_si128(self.0, rhs.0)) }
    }
}

#[cfg(target_feature = "sse2")]
impl std::ops::BitOr<BoolVector16SSE2> for BoolVector16SSE2 {
    type Output = BoolVector16SSE2;

    #[inline(always)]
    fn bitor(self, rhs: BoolVector16SSE2) -> BoolVector16SSE2 {
        unsafe { BoolVector16SSE2(_mm_or_si128(self.0, rhs.0)) }
    }
}

#[cfg(target_feature = "neon")]
#[derive(Clone, Copy)]
pub struct CharVector16NEON(uint8x16_t);

#[cfg(target_feature = "neon")]
impl CharVector16NEON {
    // data must point to at least 16 elements.
    #[inline(always)]
    pub fn load(data: &[u8]) -> CharVector16NEON {
        qljs_assert!(data.len() >= 16);
        unsafe { CharVector16NEON(vld1q_u8(data.as_ptr())) }
    }

    // data must point to at least 16 elements.
    #[inline(always)]
    pub unsafe fn load_raw(data: *const u8) -> CharVector16NEON {
        Self::load(std::slice::from_raw_parts(data, 16))
    }

    #[inline(always)]
    pub fn repeated(x: u8) -> CharVector16NEON {
        unsafe { CharVector16NEON(vdupq_n_u8(x)) }
    }

    // out_data must point to at least 16 elements.
    #[inline(always)]
    pub fn store(&self, out_data: &mut [u8]) {
        qljs_assert!(out_data.len() >= 16);
        unsafe {
            vst1q_u8(out_data.as_mut_ptr(), self.0);
        }
    }

    #[inline(always)]
    pub fn lane_eq(&self, rhs: CharVector16NEON) -> BoolVector16NEON {
        unsafe { BoolVector16NEON(vceqq_u8(self.0, rhs.0)) }
    }

    #[inline(always)]
    pub fn lane_lt(&self, rhs: CharVector16NEON) -> BoolVector16NEON {
        unsafe { BoolVector16NEON(vcltq_u8(self.0, rhs.0)) }
    }

    #[inline(always)]
    pub fn lane_gt(&self, rhs: CharVector16NEON) -> BoolVector16NEON {
        unsafe { BoolVector16NEON(vcgtq_u8(self.0, rhs.0)) }
    }

    #[inline(always)]
    pub fn uint8x16(&self) -> uint8x16_t {
        self.0
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        16
    }
}

#[cfg(target_feature = "neon")]
impl std::ops::BitOr<CharVector16NEON> for CharVector16NEON {
    type Output = CharVector16NEON;

    #[inline(always)]
    fn bitor(self, rhs: CharVector16NEON) -> CharVector16NEON {
        unsafe { CharVector16NEON(vorrq_u8(self.0, rhs.0)) }
    }
}

#[cfg(target_feature = "neon")]
#[derive(Clone, Copy, Debug)]
pub struct BoolVector16NEON(pub(crate) uint8x16_t);

#[cfg(target_feature = "neon")]
impl BoolVector16NEON {
    // data must point to at least 16 elements.
    #[inline(always)]
    pub fn load_slow(data: &[bool]) -> BoolVector16NEON {
        qljs_assert!(data.len() >= 16);
        let bytes: [u8; 16] = generate_array_n(|i: usize| {
            if unsafe { *data.get_unchecked(i) } {
                0xff
            } else {
                0x00
            }
        });
        unsafe { BoolVector16NEON(vld1q_u8(bytes.as_ptr())) }
    }

    // find_first_falls and mask are implemented in simd_neon_arm.rs.

    #[inline(always)]
    pub const fn len(&self) -> usize {
        16
    }
}

#[cfg(target_feature = "neon")]
impl std::ops::BitAnd<BoolVector16NEON> for BoolVector16NEON {
    type Output = BoolVector16NEON;

    #[inline(always)]
    fn bitand(self, rhs: BoolVector16NEON) -> BoolVector16NEON {
        unsafe { BoolVector16NEON(vandq_u8(self.0, rhs.0)) }
    }
}

#[cfg(target_feature = "neon")]
impl std::ops::BitOr<BoolVector16NEON> for BoolVector16NEON {
    type Output = BoolVector16NEON;

    #[inline(always)]
    fn bitor(self, rhs: BoolVector16NEON) -> BoolVector16NEON {
        unsafe { BoolVector16NEON(vorrq_u8(self.0, rhs.0)) }
    }
}

#[cfg(target_feature = "neon")]
pub use crate::port::simd_neon_arm::*;

#[derive(Clone, Copy, Debug)]
pub struct CharVector1(u8);

impl CharVector1 {
    #[inline(always)]
    pub fn new(data: u8) -> CharVector1 {
        CharVector1(data)
    }

    #[inline(always)]
    pub fn load(data: &[u8]) -> CharVector1 {
        CharVector1(data[0])
    }

    #[inline(always)]
    pub unsafe fn load_raw(data: *const u8) -> CharVector1 {
        Self::load(std::slice::from_raw_parts(data, 1))
    }

    #[inline(always)]
    pub fn repeated(c: u8) -> CharVector1 {
        CharVector1(c)
    }

    #[inline(always)]
    pub fn lane_eq(&self, rhs: CharVector1) -> BoolVector1 {
        BoolVector1(self.0 == rhs.0)
    }

    #[inline(always)]
    pub fn lane_lt(&self, rhs: CharVector1) -> BoolVector1 {
        BoolVector1(self.0 < rhs.0)
    }

    #[inline(always)]
    pub fn lane_gt(&self, rhs: CharVector1) -> BoolVector1 {
        BoolVector1(self.0 > rhs.0)
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        1
    }
}

impl std::ops::BitOr<CharVector1> for CharVector1 {
    type Output = CharVector1;

    #[inline(always)]
    fn bitor(self, rhs: CharVector1) -> CharVector1 {
        CharVector1(self.0 | rhs.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BoolVector1(bool);

impl BoolVector1 {
    #[inline(always)]
    pub fn new(data: bool) -> BoolVector1 {
        BoolVector1(data)
    }

    #[inline(always)]
    pub fn find_first_false(&self) -> u32 {
        if self.0 {
            1
        } else {
            0
        }
    }

    #[inline(always)]
    pub fn mask(&self) -> u32 {
        if self.0 {
            1
        } else {
            0
        }
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        1
    }
}

impl std::ops::BitAnd<BoolVector1> for BoolVector1 {
    type Output = BoolVector1;

    #[inline(always)]
    fn bitand(self, rhs: BoolVector1) -> BoolVector1 {
        BoolVector1(self.0 && rhs.0)
    }
}

impl std::ops::BitOr<BoolVector1> for BoolVector1 {
    type Output = BoolVector1;

    #[inline(always)]
    fn bitor(self, rhs: BoolVector1) -> BoolVector1 {
        BoolVector1(self.0 || rhs.0)
    }
}
