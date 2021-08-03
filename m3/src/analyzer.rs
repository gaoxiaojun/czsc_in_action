
use m2::seq_fx::SeqFx;
use common::segment::Segment;


use crate::segment_detector::SegmentDetector;
use common::event::SegmentEvent;
#[derive(Debug)]
pub struct Analyzer {
    sd: SegmentDetector
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            sd: SegmentDetector::new()
        }
    }

    pub fn on_new_seq_fx(&mut self, seq_fx:SeqFx) -> Option<SegmentEvent> {
        self.sd.on_new_seq_fx(seq_fx)
    }
}