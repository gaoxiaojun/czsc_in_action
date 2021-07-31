use crate::pen_detector::PenDetector;
use common::fx::Fx;
use common::event::PenEvent;

#[derive(Debug)]
pub struct Analyzer {
    pd:PenDetector,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            pd: PenDetector::new(),
        }
    }

    pub fn on_new_fractal(&mut self, f: Fx) -> Option<PenEvent> {
        self.pd.on_new_fractal(f)
    }
}

