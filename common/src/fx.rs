use crate::time::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxType {
    Top,
    Bottom,
}
// 分型
#[derive(Debug, Clone)]
pub struct Fx {
    pub fx_type: FxType,
    index: u64,
    pub time: Time,
    pub price: f64,
    aux: f64,
}

// 计算分型之间K线的数量,K线是经过包含处理过的
fn distance(lhs: &Fx, rhs: &Fx) -> u64 {
    if rhs.index > lhs.index {
        rhs.index - lhs.index
    } else {
        lhs.index - rhs.index
    }
}

impl Fx {
    pub fn new(fx_type: FxType, index: u64, time: Time, price: f64, aux: f64) -> Self {
        Self {
            fx_type,
            index,
            time,
            price,
            aux,
        }
    }

    pub(crate) fn distance(&self, other: &Fx) -> u64 {
        distance(self, other)
    }

    pub fn has_enough_distance(&self, other: &Fx) -> bool {
        self.distance(other) >= 4
    }

    pub fn is_same_type(&self, other: &Fx) -> bool {
        self.fx_type == other.fx_type
    }

    pub fn time(&self) -> Time {
        self.time
    }

    pub fn fx_type(&self) -> FxType {
        self.fx_type
    }

    // 返回分型的极值
    pub fn price(&self) -> f64 {
        self.price
    }

    pub fn range_high(&self) -> f64 {
        if self.fx_type == FxType::Top {
            self.price
        } else {
            self.aux
        }
    }

    pub fn range_low(&self) -> f64 {
        if self.fx_type == FxType::Top {
            self.aux
        } else {
            self.price
        }
    }
}
