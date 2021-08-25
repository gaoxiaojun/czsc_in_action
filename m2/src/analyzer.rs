
use common::event::{PenEvent, SegmentEvent};
use crate::sd::SegmentDetectorV2;

#[derive(Debug)]
pub struct Analyzer {
    sd: SegmentDetectorV2,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            sd: SegmentDetectorV2::new(),
        }
    }

    pub fn on_new_pen_event(&mut self, pen_event: PenEvent) -> Option<SegmentEvent> {
        self.sd.on_pen_event(pen_event)
    }
}
