// Copyright (C) 2020  Matthew "strager" Glazar
// Copyright (c) 2014-2020, Arm Limited.
// See end of file for extended copyright information.

// Some routines have a different copyright than the rest of quick-lint-js, thus
// are in this separate file.

#[cfg(target_feature = "neon")]
use crate::port::simd::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
#[cfg(target_arch = "arm")]
use std::arch::arm::*;

#[cfg(target_feature = "neon")]
impl BoolVector16NEON {
    #[cfg(target_arch = "aarch64")]
    pub fn find_first_false(&self) -> i32 {
        unsafe {
            // You might expect a magic pattern to look like the following:
            //
            //   { 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x08, [repeat] }
            //
            // However, the above magic pattern requires mixing cells 3 times
            // (16x8 -> 8x16 -> 4x32 -> 2x64). Our magic pattern requires mixing cells
            // only 2 times, but creates an unusual mask (see
            // NOTE[find_first_false NEON mask]).
            let magic: uint8x16_t = vld1q_u8(
                [
                    0x01, 0x04, 0x10, 0x40, 0x01, 0x04, 0x10, 0x40, //
                    0x01, 0x04, 0x10, 0x40, 0x01, 0x04, 0x10, 0x40, //
                ]
                .as_ptr(),
            );

            // It doesn't matter what 'garbage' is. Could be zeros or ones or anything. If
            // we ever extend this algorithm to uint8x32_t inputs, garbage would be the
            // upper 128 bits.
            let garbage: uint8x16_t = self.0;

            // We invert the input so that we can use countr_zero instead of countr_one.
            // countr_one can't be used because of the zero bits in our mask (see
            // NOTE[find_first_false NEON mask]).
            let mixed_0: uint8x16_t = vbicq_u8(magic, self.0);

            // Mix bits to create a mask. Note that arithmetic ADD is effectively
            // bitwise OR.
            //
            // mixed_0: { a b c d  e f g h  i j k l  m n o p }
            // mixed_1: { a+b c+d  e+f g+h  i+j k+l  m+n o+p  (64 bits unused...) }
            // mixed_2: { a+b+c+d  e+f+g+h  i+j+k+l  m+n+o+p  (96 bits unused...) }
            let mixed_1: uint8x16_t = vpaddq_u8(mixed_0, garbage);
            let mixed_2: uint8x16_t = vpaddq_u8(mixed_1, mixed_1);
            let mask: u32 = vgetq_lane_u32(vreinterpretq_u32_u8(mixed_2), 0);

            // NOTE[find_first_false NEON mask]: After mixing bits, an ideal mask looks
            // like this:
            //
            //   0b0000000000000000ABCDEFGHIJKLMNOP
            //
            // But our mask looks like this:
            //
            //   0b0A0B0C0D0E0F0G0H0I0J0K0L0M0N0O0P
            //
            // To deal with the extra zeros, we to divide our countr_zero result by 2.
            (mask.trailing_zeros() / 2) as i32
        }
    }

    #[cfg(target_arch = "arm")]
    pub fn find_first_false() -> i32 {
        self.mask().trailing_ones() as i32
    }

    #[rustfmt::skip]
    pub fn mask(&self) -> u32 {
        unsafe {
            // Algorithm derived from sse2neon's _mm_movemask_epi8 function:
            // https://github.com/DLTcollab/sse2neon/blob/814935c9ba06f68e9549272dbf5df0db8dab2a00/sse2neon.h#L4752-L4830
            let high_bits: uint16x8_t  = vreinterpretq_u16_u8 (vshrq_n_u8 (self.0,                8 - 1));
            let paired16:  uint32x4_t  = vreinterpretq_u32_u16(vsraq_n_u16(high_bits, high_bits,  8 - 1));
            let paired32:  uint64x2_t  = vreinterpretq_u64_u32(vsraq_n_u32(paired16,  paired16,  16 - 2));
            let paired64:  uint8x16_t  = vreinterpretq_u8_u64 (vsraq_n_u64(paired32,  paired32,  32 - 4));
            (vgetq_lane_u8(paired64, 0) as u32) | ((vgetq_lane_u8(paired64, 8) as u32) << 8)
        }
    }
}

// quick-lint-js finds bugs in JavaScript programs.
// Copyright (C) 2020  Matthew "strager" Glazar
//
// This file is part of quick-lint-js.
//
// quick-lint-js is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// quick-lint-js is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with quick-lint-js.  If not, see <https://www.gnu.org/licenses/>.
//
// ---
//
// Portions of this file are
// Copyright (c) 2014-2020, Arm Limited.
// Source:
// https://github.com/ARM-software/optimized-routines/blob/7a9fd1603e1179b044406fb9b6cc5770d736cde7/string/aarch64/memchr.S
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// --
//
// Portions of this file are from sse2neon.
// Source:
// https://github.com/DLTcollab/sse2neon/blob/814935c9ba06f68e9549272dbf5df0db8dab2a00/sse2neon.h
//
// sse2neon is freely redistributable under the MIT License.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
