use common::direction::Direction;

#[derive(Debug, Clone, Copy)]
pub struct Seq {
    high:f64,
    low:f64
}

impl Seq {
    pub fn new(from:f64, to:f64) -> Self { Self { 
        high:f64::max(from, to),
        low:f64::min(from, to)
     } }

    pub fn merge(&mut self, from:f64, to:f64, direction:Direction) -> bool {
        let _high = f64::max(from, to);
        let _low =f64::min(from, to);

        let prev_include_next = self.high > _high && self.low < _low;
        let next_include_prev = self.high < _high && self.low > _low;

        if !prev_include_next && !next_include_prev {
            return false;
        }

        match direction {
            Direction::Up => {
                self.high = f64::max(self.high, _high);
                self.low = f64::max(self.low, _low);
            },
            Direction::Down => {
                self.high = f64::min(self.high, _high);
                self.low = f64::min(self.low, _low);
            }
        }
        true
    }
}