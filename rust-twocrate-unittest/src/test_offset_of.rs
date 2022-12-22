use crate::qljs_offset_of;

use memoffset::offset_of;

#[test]
fn matches_memoffset_for_reference_fields() {
    struct Test<'a> {
        x: &'a i32,
        y: &'a (),
        z: &'a mut bool,
    }

    assert_eq!(qljs_offset_of!(Test, x), offset_of!(Test, x));
    assert_eq!(qljs_offset_of!(Test, y), offset_of!(Test, y));
    assert_eq!(qljs_offset_of!(Test, z), offset_of!(Test, z));
}

#[test]
fn matches_memoffset_for_primitive_fields() {
    struct Test {
        x: i32,
        y: (),
        z: bool,
    }

    assert_eq!(qljs_offset_of!(Test, x), offset_of!(Test, x));
    assert_eq!(qljs_offset_of!(Test, y), offset_of!(Test, y));
    assert_eq!(qljs_offset_of!(Test, z), offset_of!(Test, z));
}

#[test]
fn fields_have_different_offsets() {
    struct Test<'a> {
        x: i32,
        y: &'a [u8],
        z: bool,
    }

    assert_ne!(qljs_offset_of!(Test, x), qljs_offset_of!(Test, y));
    assert_ne!(qljs_offset_of!(Test, x), qljs_offset_of!(Test, z));
    assert_ne!(qljs_offset_of!(Test, y), qljs_offset_of!(Test, x));
}
