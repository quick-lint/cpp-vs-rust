use cpp_vs_rust::scoped_trace;
use cpp_vs_rust::util::simd::*;

// TODO(port-later): Run tests with each supported CharVector16 implementation, rather than run
// with just one.
#[cfg(target_feature = "simd128")]
type CharVector16 = CharVector16WASMSIMD128;
#[cfg(any(target_feature = "sse2", target_arch = "x86_64"))]
type CharVector16 = CharVector16SSE2;
#[cfg(target_feature = "neon")]
type CharVector16 = CharVector16NEON;

#[test]
fn char16_repeated() {
    let mut actual: [u8; 16] = [0; 16];
    CharVector16::repeated(b'x').store(&mut actual);
    assert_eq!(actual, [b'x'; 16]);
}

#[test]
fn char16_bitwise_or() {
    let lhs: [u8; 16] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, //
        0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, //
    ];
    let rhs: [u8; 16] = [
        0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, //
        0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, //
    ];
    let mut actual: [u8; 16] = [0; 16];
    (CharVector16::load(&lhs) | CharVector16::load(&rhs)).store(&mut actual);
    assert_eq!(
        actual,
        [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, //
            0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, //
        ]
    );
}

// TODO(port-later): Run tests with each supported BoolVector16 implementation, rather than run
// with just one.
#[cfg(target_feature = "simd128")]
type BoolVector16 = BoolVector16WASMSIMD128;
#[cfg(any(target_feature = "sse2", target_arch = "x86_64"))]
type BoolVector16 = BoolVector16SSE2;
#[cfg(target_feature = "neon")]
type BoolVector16 = BoolVector16NEON;

#[test]
fn bool16_first_false_of_all_false() {
    let bools_data: [bool; 16] = [false; 16];
    let bools = BoolVector16::load_slow(&bools_data);
    assert_eq!(bools.find_first_false(), 0);
}

#[test]
fn bool16_first_false_of_all_true() {
    let bools_data: [bool; 16] = [true; 16];
    let bools = BoolVector16::load_slow(&bools_data);
    assert_eq!(bools.find_first_false(), 16);
}

#[test]
fn bool16_find_first_false_exhaustive_slow() {
    for i in 0..=0xffff {
        scoped_trace!(i);
        let mut bools_data: [bool; 16] = [false; 16];
        let mut first_false = 16;
        for bit in 0..16 {
            let bit_on = ((i >> bit) & 1) != 0;
            bools_data[bit as usize] = bit_on;
            if !bit_on {
                first_false = std::cmp::min(first_false, bit);
            }
        }

        let bools = BoolVector16::load_slow(&bools_data);
        assert_eq!(bools.find_first_false(), first_false);
    }
}

#[test]
fn bool16_mask_all_false() {
    let bools_data: [bool; 16] = [false; 16];
    let bools = BoolVector16::load_slow(&bools_data);
    assert_eq!(bools.mask(), 0x0000);
}

#[test]
fn bool16_mask_all_true() {
    let bools_data: [bool; 16] = [true; 16];
    let bools = BoolVector16::load_slow(&bools_data);
    assert_eq!(bools.mask(), 0xffff);
}
