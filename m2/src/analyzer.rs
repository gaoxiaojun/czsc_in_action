
use common::event::{PenEvent, SegmentEvent};
use crate::sd3::SegmentDetector;

#[derive(Debug)]
pub struct Analyzer {
    sd: SegmentDetector,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            sd: SegmentDetector::new(),
        }
    }

    pub fn on_new_pen_event(&mut self, pen_event: PenEvent) -> Option<SegmentEvent> {
        self.sd.on_pen_event(pen_event)
    }
}
