use crate::fe::diag_reporter::*;
use crate::fe::diagnostic_types::*;

pub struct DiagCollector<'code> {
    pub errors: std::cell::RefCell<Vec<AnyDiag<'code>>>,
}

impl<'code> DiagCollector<'code> {
    pub fn new() -> DiagCollector<'code> {
        DiagCollector {
            errors: std::cell::RefCell::new(vec![]),
        }
    }

    pub fn len(&self) -> usize {
        self.errors.borrow().len()
    }

    pub fn index<'a>(&'a self, index: usize) -> AnyDiag<'code> {
        self.errors.borrow_mut()[index].clone()
    }
}

impl<'code> DiagReporter for DiagCollector<'code> {
    fn report_impl(&self, type_: DiagType, diag: *const u8) {
        self.errors
            .borrow_mut()
            .push(unsafe { AnyDiag::from_raw_parts(type_, diag) });
    }
}
