use colored::Colorize;
use common::{direction::Direction, point::Point, print_flush};
// 序列的包含关系
#[derive(Debug, PartialEq, Eq)]
pub enum Relationship {
    NotInclude,       // 没有包含关系
    PrevIncludeNext, // 前包含后
    NextIncludePrev, // 后包含前
    FullEquivalent,   // 完全相等，这个是特殊情况
}
// 向上的线段采用向上合并, 向下的特征序列，找顶分型
// 向下的线段采用向下合并, 向上的特征序列，找底分型

#[derive(Debug, Clone, Copy)]
pub struct Seq {
    pub from_index: usize,
    pub from: Point,
    pub to: Point,
    pub direction: Direction,
    pub pin: bool, // prev include next
    pub nip: bool, // next include prev
}

impl Seq {
    pub fn new(from_index: usize, from: &Point, to: &Point) -> Self {
        Self {
            from_index,
            from: from.clone(),
            to: to.clone(),
            direction: if from.price - to.price > 0.0 {
                Direction::Down
            } else {
                Direction::Up
            },
            pin: false,
            nip: false,
        }
    }

    pub fn high(&self) -> f64 {
        f64::max(self.from.price, self.to.price)
    }

    pub fn low(&self) -> f64 {
        f64::min(self.from.price, self.to.price)
    }

    pub fn check_include_relationship(&self, rhs: &Seq) -> Relationship {
        let next_include_prev = self.high() <= rhs.high() && self.low() >= rhs.low();
        let prev_include_next = self.high() >= rhs.high() && self.low() <= rhs.low();
        match (prev_include_next, next_include_prev) {
            (true, true) => Relationship::FullEquivalent,
            (true, false) => Relationship::PrevIncludeNext,
            (false, true) => Relationship::NextIncludePrev,
            (false, false) => Relationship::NotInclude,
        }
    }

    pub fn merge(&mut self, rhs: &Seq, relationship:Relationship) {
        if self.direction != rhs.direction {
            print_flush!("self: {} {} {} rhs: {} {} {}\n",
            self.from, self.to, self.direction, rhs.from, rhs.to, rhs.direction);
        }
        debug_assert!(self.direction == rhs.direction); // 必须同方向
        debug_assert!(relationship != Relationship::NotInclude);

        match relationship {
            Relationship::PrevIncludeNext => {
                self.pin = true;
                self.to = rhs.to;
            }
            Relationship::NextIncludePrev => {
                self.nip = true;
                self.from = rhs.from;
            }

            _ => {}
        }
    }
}

impl std::fmt::Display for Seq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "index:{} from:{} to:{}",
            self.from_index, self.from, self.to
        )
    }
}
