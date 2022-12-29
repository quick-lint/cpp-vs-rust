use cpp_vs_rust_util::narrow_cast::*;
use cpp_vs_rust_util::padded_string::*;

pub trait LocatorLike<'code> {
    type RangeType;

    fn new(s: PaddedStringView<'code>) -> Self;
}

pub struct Document<Locator> {
    active_content_buffer: usize,
    content_buffers: std::cell::UnsafeCell<[PaddedString; 2]>,
    // locator contains a reference to the heap-allocated data of content_buffers[0] or
    // content_buffers[1].
    locator: Locator,
}

impl<'code, Locator: LocatorLike<'code>> Document<Locator> {
    pub fn new() -> Document<Locator> {
        let active_content_buffer: usize = 0;
        let content_buffers =
            std::cell::UnsafeCell::new([PaddedString::new(), PaddedString::new()]);
        let locator: Locator = Locator::new(unsafe {
            (*content_buffers.get().cast_const())[active_content_buffer].view()
        });
        Document {
            active_content_buffer: active_content_buffer,
            content_buffers: content_buffers,
            locator: locator,
        }
    }

    pub fn set_text(&mut self, new_text: &[u8]) {
        let content: &mut PaddedString =
            unsafe { &mut (*self.content_buffers.get())[self.active_content_buffer] };
        content.resize(narrow_cast::<i32, _>(new_text.len()));
        content.as_mut_slice().copy_from_slice(new_text);
        self.locator = Locator::new(content.view());
    }

    pub fn replace_text(_range: Locator::RangeType, _replacement_text: &[u8]) {
        unimplemented!();
    }

    pub fn string<'this>(&'this mut self) -> PaddedStringView<'this> {
        unsafe { (*self.content_buffers.get().cast_const())[self.active_content_buffer].view() }
    }

    pub fn locator<'this>(&'this mut self) -> &'this Locator {
        &self.locator
    }
}
