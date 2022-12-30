use crate::fe::diagnostic_types::*;

pub trait DiagReporter {
    // Do not call directly. Call 'report' instead.
    fn report_impl(&self, type_: DiagType, diag: *const u8);
}

// TODO(strager): Make this a method on DiagReporter instead.
pub fn report<Diag: HasDiagType>(reporter: &dyn DiagReporter, diag: Diag) {
    reporter.report_impl(Diag::TYPE_, &diag as *const Diag as *const u8);
}

struct NullDiagReporter;
impl DiagReporter for NullDiagReporter {
    fn report_impl(&self, _type: DiagType, _diag: *const u8) {}
}

static NULL_DIAG_REPORTER_SINGLETON: NullDiagReporter = NullDiagReporter;

pub fn null_diag_reporter() -> &'static impl DiagReporter {
    &NULL_DIAG_REPORTER_SINGLETON
}
