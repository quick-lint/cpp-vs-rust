use crate::fe::diagnostic_types::*;

pub trait DiagReporter {
    // Do not call directly. Call 'report' instead.
    fn report_impl(&self, type_: DiagType, diag: *const u8);
}

// TODO(port): Make this a method on DiagReporter instead.
pub fn report<Diag: HasDiagType>(reporter: &dyn DiagReporter, diag: Diag) {
    reporter.report_impl(Diag::TYPE_, &diag as *const Diag as *const u8);
}
