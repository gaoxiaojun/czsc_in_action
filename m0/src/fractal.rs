use crate::candle::Candle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractalType {
    Top,
    Bottom,
}
// 分型
#[derive(Debug, Clone)]
pub struct Fractal {
    fx_type:FractalType,
    index: u64,
    time: Time,
    price:f64,
    aux:f64,
}

// 计算分型之间K线的数量,K线是经过包含处理过的
fn distance(lhs: &Fractal, rhs: &Fractal) -> u64 {
    if rhs.index > lhs.index {
        rhs.index - lhs.index
    } else {
        lhs.index - rhs.index
    }
}

impl Fractal {
    pub fn new(k1: &Candle, k2: &Candle, k3: &Candle) -> Self {
        debug_assert!(
            // 合并之后，分型的最高/最低是唯一的，所以没有等号
            ((k1.high < k2.high) && (k2.high > k3.high)) // Top
                || ((k1.low > k2.low) && (k2.low < k3.low)) // Bottom
        );

        let is_top = (k1.high < k2.high) && (k2.high > k3.high);
        let (ftype, price, aux) = if is_top {
            (FractalType::Top, k2.high, k2.low)
        } else {
            (FractalType::Bottom, k2.low, k2.high)
        };

        Self { 
            index: k2.index,
            fx_type: ftype,
            time: k2.time,
            price: price,
            aux: aux
        }

    }

    //  ------k2---------
    //  ------|----------
    //  -k1-|---|-k3-----
    //  ------|----------
    //  -----k2----------

    // 检查分型
    pub fn check_fractal(k1: &Candle, k2: &Candle, k3: &Candle) -> Option<Fractal> {
        debug_assert!(k1.index != k2.index && k1.index != k3.index && k2.index != k3.index);
        if ((k1.high < k2.high) && (k2.high > k3.high))
            || ((k1.low > k2.low) && (k2.low < k3.low))
        {
            return Some(Fractal::new(k1, k2, k3));
        }
        None
    }

    pub(crate) fn distance(&self, other: &Fractal) -> u64 {
        distance(self, other)
    }

    pub fn has_enough_distance(&self, other: &Fractal) -> bool {
        self.distance(other) >= 4
    }

    pub fn is_same_type(&self, other: &Fractal) -> bool {
        self.fx_type == other.fx_type
    }

    pub fn time(&self) -> Time {
        self.time
    }

    pub fn fractal_type(&self) -> FractalType {
        self.fx_type
    }

    // 返回分型的极值
    pub fn price(&self) -> f64 {
        self.price
    }

    pub fn high(&self) -> f64 {
        if self.fx_type == FractalType::Top {
            self.price
        }else {
            self.aux
        }
    }

    pub fn low(&self) -> f64 {
        if self.fx_type == FractalType::Top {
            self.aux
        }else {
            self.price
        }
    }

    // 分型包含规则1，第二根Candle的最高最低作为分型区间
    fn is_contain_rule1(&self, other: &Fractal) -> bool {
        if self.high() >= other.high() && self.low() <= other.low() {
            true
        } else {
            false
        }
    }

    pub fn is_contain(&self, other: &Fractal) -> bool {
        self.is_contain_rule1(other)
    }
}
