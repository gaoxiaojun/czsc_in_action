use crate::{seq_fx::SeqFx, seq_fx_detector::SeqFxDetector};
use common::event::PenEvent;
use common::point::Point;

#[derive(Debug)]
pub struct Analyzer {
    sfd: SeqFxDetector,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            sfd: SeqFxDetector::new(),
        }
    }

    pub fn on_new_pen_event(&mut self, pen_event: &PenEvent) -> Option<SeqFx> {
        self.sfd.on_new_pen_event(pen_event)
    }

    pub fn get_pens(&self)  -> Vec<Point> {
        let mut pens:Vec<Point> = Vec::new();
        for p in &self.sfd.pens {
            pens.push(p.clone());
        }
        pens
    }
}
