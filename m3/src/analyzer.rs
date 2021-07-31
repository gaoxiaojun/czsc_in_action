
use m2::seq_fx::SeqFx;
use common::segment::Segment;

#[derive(Debug)]
pub struct Analyzer {

}

impl Analyzer {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn on_new_seq_fx(&mut self, seq_fx:SeqFx) -> Option<Segment> {
        None
    }
}