use common::direction::Direction;
use common::fx::FxType;
use common::point::Point;
use common::ringbuffer::RingBuffer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    high: f64,
    low: f64,
}

impl Line {
    pub fn new(from: &Point, to: &Point) -> Self {
        Self {
            high: f64::max(from.price, to.price),
            low: f64::min(from.price, to.price),
        }
    }

    pub fn merge(&mut self, rhs: &Line, direction: Direction) -> bool {
        let prev_include_next = self.high >= rhs.high && self.low <= rhs.low;
        let next_include_prev = self.high <= rhs.high && self.low >= rhs.low;

        let is_include = prev_include_next | next_include_prev;

        if !is_include {
            return false;
        }

        // direction 线段方向
        match direction {
            Direction::Up => {
                // 高高处理
                self.high = f64::max(self.high, rhs.high);
                self.low = f64::max(self.low, rhs.low);
            }
            Direction::Down => {
                self.high = f64::min(self.high, rhs.high);
                self.low = f64::min(self.low, rhs.low);
            }
        }

        false
    }
}

#[derive(Debug)]
pub struct FxDetector {
    window: RingBuffer<Line>,
}

impl FxDetector {
    pub fn new() -> Self {
        Self {
            window: RingBuffer::new(3),
        }
    }

    fn process_contain_relationship(&mut self, line: &Line, direction: Direction) -> bool {
        // 队列中有至少两个经过包含处理的Candle
        debug_assert!(self.window.len() >= 2);

        let current = self.window.get_mut(-1).unwrap();

        current.merge(line, direction)
    }

    pub fn add(&mut self, line: Line, direction: Direction) -> Option<FxType> {
        match self.window.len() {
            0 => {
                self.window.push(line);
                return None;
            }
            1 => {}
            _ => {}
        }

        None
    }

    pub fn is_top_fractal(s1: &Line, s2: &Line, s3: &Line) -> bool {
        if s1.high < s2.high && s2.high > s3.high {
            true
        } else {
            false
        }
    }

    pub fn is_bottom_fractal(s1: &Line, s2: &Line, s3: &Line) -> bool {
        if s1.low > s2.low && s2.low > s3.low {
            true
        } else {
            false
        }
    }
}
