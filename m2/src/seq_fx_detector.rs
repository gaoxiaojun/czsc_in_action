use std::collections::VecDeque;

use crate::seq_fx::SeqFx;
use crate::sequence::Seq;
use common::direction::Direction;
use common::event::PenEvent;
use common::point::Point;
use common::ringbuffer::RingBuffer;

#[derive(Debug)]
pub struct SeqFxDetector {
    pens: VecDeque<Point>,
    window: RingBuffer<Seq>,
    direction: Option<Direction>,
}

impl SeqFxDetector {
    pub fn new() -> Self {
        Self {
            pens: VecDeque::new(),
            window: RingBuffer::new(3),
            direction: None,
        }
    }

    // 当确定当前Bar与前Candle不存在合并关系的时候，该方法被调用
    fn add_seq(&mut self, seq: Seq) {
        //let c = Candle::from_bar(self.next_index, bar);
        self.window.push(seq);
    }

    fn build_seq_fx(s1: &Seq, s2: &Seq, s3: &Seq) -> SeqFx {
        SeqFx::build_seq_fx(s1, s2, s3)
    }

    // 检查是否为顶底分型
    fn check_fx(&self) -> Option<SeqFx> {
        let k1 = self.window.get(-3).unwrap();
        let k2 = self.window.get(-2).unwrap();
        let k3 = self.window.get(-1).unwrap();

        if ((k1.high() < k2.high()) && (k2.high() > k3.high()))
            || ((k1.low() > k2.low()) && (k2.low() < k3.low()))
        {
            return Some(SeqFxDetector::build_seq_fx(k1, k2, k3));
        }
        None
    }

    // 处理包含关系
    fn process_contain_relationship(&mut self, seq: &Seq) -> bool {
        debug_assert!(self.window.len() >= 2);

        let current = self.window.get_mut(-1).unwrap();

        Seq::merge(current, &seq)
    }

    // 处理包含关系，更新内部缓冲区，检测分型
    fn on_new_seq(&mut self, seq: Seq) -> Option<SeqFx> {
        let len = self.window.len();
        debug_assert!(len <= 3);
        match len {
            0 | 1 => {
                self.add_seq(seq);
            }

            2 => {
                let merged = self.process_contain_relationship(&seq);
                if !merged {
                    self.add_seq(seq)
                }
            }

            _ => {
                let merged = self.process_contain_relationship(&seq);
                if !merged {
                    let result = self.check_fx();
                    self.add_seq(seq);
                    return result;
                }
            }
        }
        None
    }

    pub fn update_seq(&mut self) -> Option<SeqFx> {
        debug_assert!(self.pens.len() > 3);
        let _2 = self.pens.len() - 2;
        let _3 = self.pens.len() - 3;

        let from = self.pens[_3];
        let to = self.pens[_2];
        let last_seq = Seq::new(from, to);

        self.on_new_seq(last_seq)
    }

    // 判断第一个线段的时候，条件约束较严格
    fn is_first_segment(p1: &Point, p2: &Point, p3: &Point, p4: &Point) -> Option<Direction> {
        let direction_up = p1.price < p2.price
            && p2.price > p3.price
            && p3.price > p1.price
            && p4.price > p3.price
            && p4.price > p2.price;
        let direction_down = p1.price > p2.price
            && p2.price < p3.price
            && p3.price < p1.price
            && p4.price < p3.price
            && p4.price < p2.price;

        let direction = {
            match (direction_up, direction_down) {
                (true, false) => Some(Direction::Up),
                (false, true) => Some(Direction::Down),
                (_, _) => None,
            }
        };
        direction
    }

    fn find_first_segment(&mut self) -> bool {
        // 查找第一个线段
        // 判断方式通过4个分型的滑动窗口来判断
        // 这里没有包含全部的情况，例如1-2-3-4-5-6等多个笔组成线段，
        // TODO: 按照缠论前3笔重叠构成线段，因此如果前三笔没有构成4点高于2点是不是也算线段？
        // 如果算，这里的第一笔检测算法就要更新，
        debug_assert!(self.direction.is_none());
        debug_assert!(self.pens.len() >= 4);
        let _1 = self.pens.len() - 1;
        let _2 = self.pens.len() - 2;
        let _3 = self.pens.len() - 3;
        let _4 = self.pens.len() - 4;
        let p1 = self.pens.get(_4).unwrap();
        let p2 = self.pens.get(_3).unwrap();
        let p3 = self.pens.get(_2).unwrap();
        let p4 = self.pens.get(_1).unwrap();

        self.direction = SeqFxDetector::is_first_segment(p1, p2, p3, p4);

        self.direction.is_some()
    }

    pub fn process_when_new_pen(&mut self) -> Option<SeqFx>{
        // 注意：当前笔是不处理的，因为笔可能会继续延伸

        if self.pens.len() < 4 {
            // 笔的端点数量太少，未构成线段
            return None;
        }

        // 如果当前没有方向，说明没有找到第一个开始线段
        if self.direction.is_none() {
            let found = self.find_first_segment();
            if !found {
                // 如果没有找到线段，说明第一个起始点是不对的，弹出
                self.pens.pop_front();
                return None;
            }
        }

        // 最后一笔不要处理,长度为偶数的时候更新特征序列
        if self.pens.len() % 2 == 0 {
            // it's time to update seq
            return self.update_seq();
        }

        None
    }

    pub fn on_new_pen_event(&mut self, pen_event: &PenEvent) -> Option<SeqFx> {
        match pen_event {
            PenEvent::First(start, end) => {
                let from = Point::new(start.time, start.price);
                let to = Point::new(end.time, end.price);
                self.pens.push_back(from);
                self.pens.push_back(to);
                self.process_when_new_pen();
            }

            PenEvent::New(end) => {
                let to = Point::new(end.time, end.price);
                self.pens.push_back(to);
                return self.process_when_new_pen()
            }

            PenEvent::UpdateTo(end) => {
                let to = Point::new(end.time, end.price);
                self.pens.pop_back();
                self.pens.push_back(to);
            }
        };
        None
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use common::test_util::*;

    #[test]
    fn test_seq_fx_detector() {
        let bars = load_eurusd_2021();
        let mut fd = m0::analyzer::Analyzer::new();
        let mut pd = m1::analyzer::Analyzer::new();
        let mut sfxd = SeqFxDetector::new();
        let mut count = 0;
        for bar in &bars {
            let f = fd.on_new_bar(bar);
            if let Some(fx) = f {
                let event = pd.on_new_fractal(fx);
                if let Some(pe) = event {
                    let sfx = sfxd.on_new_pen_event(&pe);
                    if sfx.is_some() {
                        count += 1;
                    }
                }
            }
        }
        println!("Sfx Count {} ", count);
    }
}
