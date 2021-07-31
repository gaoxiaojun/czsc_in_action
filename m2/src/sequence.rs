
use common::point::Point;


// 向上的线段采用向上合并, 向下的特征序列，找顶分型
// 向下的线段采用向下合并, 向上的特征序列，找底分型

#[derive(Debug, Clone)]
pub struct Seq {
    pub from: Point,
    pub to: Point,
}

impl Seq {
    pub fn new(from: Point, to: Point) -> Self {
        Self { from, to }
    }

    pub fn start(&self) -> Point {
        self.from
    }

    pub fn end(&self) -> Point {
        self.to
    }

    pub fn high(&self) -> f64 {
        if self.from.price > self.to.price {
            self.from.price
        } else {
            self.to.price
        }
    }

    pub fn low(&self) -> f64 {
        if self.from.price < self.to.price {
            self.from.price
        } else {
            self.to.price
        }
    }

    // 向上线段(向下特征序列，找顶分型)需要向上合并
    // 向下线段(向上特征序列，找底分型)需要向下合并
    // 下列代码同时适用于向上合并和向下合并
    pub fn merge_up_down(&mut self, rhs: &Seq) {
        let lhs_length = self.to.price - self.from.price;
        let rhs_length = self.to.price - self.from.price;
        let is_same =
            (lhs_length < 0.0 && rhs_length < 0.0) || (lhs_length > 0.0 && rhs_length > 0.0);

        let is_larger = (lhs_length.abs() - rhs_length.abs()) > 0.0;

        match (is_larger, is_same) {
            (false, true) => {
                self.from = self.to;
                self.to = rhs.from;
            }
            (false, false) => {
                self.to = rhs.from;
            }
            (true, true) => {
                self.to = rhs.to;
            }
            (true, false) => {
                self.from = self.to;
                self.to = rhs.to;
            }
        }
    }

    pub fn merge(lhs: &mut Seq, rhs: &Seq) -> bool {
        let is_contain_1 = lhs.high() < rhs.high() && lhs.low() > rhs.low();
        let is_contain_2 = lhs.high() > rhs.high() && lhs.low() < rhs.low();
        let is_contain = is_contain_1 || is_contain_2;

        if !is_contain {
            return false;
        }

        lhs.merge_up_down(rhs);

        true
    }
}
