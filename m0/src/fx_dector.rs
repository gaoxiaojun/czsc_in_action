use crate::candle::Candle;
use common::bar::Bar;
use common::fx::*;
use common::ringbuffer::RingBuffer;

#[derive(Debug)]
pub struct FxDetector {
    window: RingBuffer<Candle>,
    next_index: u64,
}

impl FxDetector {
    pub fn new() -> Self {
        Self {
            window: RingBuffer::new(3),
            next_index: 0,
        }
    }

    // 当确定当前Bar与前Candle不存在合并关系的时候，该方法被调用
    fn add_candle(&mut self, bar: &Bar) {
        let c = Candle::from_bar(self.next_index, bar);
        self.next_index += 1;
        self.window.push(c);
    }

    fn build_fx(k1: &Candle, k2: &Candle, k3: &Candle) -> Fx {
        let is_top = (k1.high < k2.high) && (k2.high > k3.high);
        let (ftype, price, aux) = if is_top {
            (FxType::Top, k2.high, k2.low)
        } else {
            (FxType::Bottom, k2.low, k2.high)
        };

        Fx::new(ftype, k2.index, k2.time, price, aux)
    }
    // 检查是否为顶底分型
    fn check_fx(&self) -> Option<Fx> {
        let k1 = self.window.get(-3).unwrap();
        let k2 = self.window.get(-2).unwrap();
        let k3 = self.window.get(-1).unwrap();

        debug_assert!(k1.index != k2.index && k1.index != k3.index && k2.index != k3.index);
        if ((k1.high < k2.high) && (k2.high > k3.high)) || ((k1.low > k2.low) && (k2.low < k3.low))
        {
            return Some(FxDetector::build_fx(k1, k2, k3));
        }
        None
    }

    // 处理与当前bar的包含关系
    fn process_contain_relationship(&mut self, bar: &Bar) -> bool {
        // 队列中有至少两个经过包含处理的Candle
        debug_assert!(self.window.len() >= 2);
        let direction = {
            let k1 = self.window.get(-2).unwrap();
            let k2 = self.window.get(-1).unwrap();
            Candle::check_direction(k1, k2)
        };

        let current = self.window.get_mut(-1).unwrap();

        Candle::merge(direction, current, bar)
    }

    // 处理K线包含关系，更新内部缓冲区，检测分型
    pub fn on_new_bar(&mut self, bar: &Bar) -> Option<Fx> {
        let len = self.window.len();
        debug_assert!(len <= 3);

        // 初始边界条件验证，前两个candle必须是非包含的
        match len {
            0 => {
                // 队列中没有K线
                self.add_candle(bar);
            }

            1 => {
                // 仅有一根K线
                // 起始开始的两K就存在包含关系，合理的处理方式是：
                // 1. 如果第一根K包含第二根K，直接忽略与第一根K存在包含的K线，直到遇到不包含的
                // 2. 如果第一根K包含在第二根K，忽略第一根K，从第二根K开始
                let last = self.window.get(-1).unwrap();
                let k1_include_k2 = last.high >= bar.high && last.low <= bar.low;
                let k2_include_k1 = last.high <= bar.high && last.low >= bar.low;
                if k1_include_k2 {
                    // 情况1，忽略当前Bar，直到遇到不包含的
                    return None;
                };

                if k2_include_k1 {
                    // 情况2，忽略K1,清空队列
                    self.window.clear();
                }
                // 当前Bar作为Candle放入队列
                self.add_candle(bar);
            }

            2 => {
                let merged = self.process_contain_relationship(bar);
                if !merged {
                    self.add_candle(bar);
                }
            }

            _ => {
                let merged = self.process_contain_relationship(bar);
                if !merged {
                    let result = self.check_fx();
                    self.add_candle(bar);
                    return result;
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fx::tests::*;
    use common::test_util::*;

    #[test]
    fn test_candle_merge_fx_detector() {
        let bars = load_eurusd_2021();
        let mut fvec: Vec<Fx> = Vec::new();
        let mut fd = FxDetector::new();
        for bar in &bars {
            let f = fd.on_new_bar(bar);
            if let Some(fx) = f {
                fvec.push(fx);
            }
        }

        let fxs = load_fx();

        assert!(fvec.len() == fxs.len());

        for i in 0..fvec.len() {
            let f1 = &fvec[i];
            let f2 = &fxs[i];
            assert!(
                f1.time() == f2.time
                    && f1.price() == f2.price
                    && f1.fx_type() == f2.ftype
                    && f1.high() == f2.high
                    && f1.low() == f2.low
            );
        }
    }
}
