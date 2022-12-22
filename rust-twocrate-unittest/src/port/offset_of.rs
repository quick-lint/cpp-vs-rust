#[macro_export]
macro_rules! qljs_offset_of {
    ($type:ty, $field:tt $(,)?) => {
        unsafe {
            let temp: std::mem::MaybeUninit<$type> = std::mem::MaybeUninit::uninit();
            let base_ptr = temp.assume_init_ref() as *const _ as *const u8;
            let field_ptr = &temp.assume_init_ref().$field as *const _ as *const u8;
            (field_ptr.offset_from(base_ptr)) as usize
        }
    };
}
