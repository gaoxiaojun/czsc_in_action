
use crate::fx_dector::FxDetector;
use common::bar::Bar;
use common::fx::Fx;

#[derive(Debug)]
pub struct Analyzer {
    fd: FxDetector,
}

impl Analyzer {

    pub fn new() -> Self {
        Self {
            fd: FxDetector::new()
        }
    }

    pub fn on_new_bar(&mut self, bar: &Bar) -> Option<Fx> {
        self.fd.on_new_bar(bar)
    }

}