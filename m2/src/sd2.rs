use crate::seq2::Seq;
use common::direction::Direction;
use common::event::{PenEvent, SegmentEvent};
use common::fx::FxType;
use common::point::Point;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct State {
    potential_index: usize,
    fx_type: FxType,
    has_gap: bool,
    k2: Seq,
    confirm: bool,
}

impl State {
    pub fn new(potential_index: usize, fx_type: FxType, has_gap: bool, k2: Seq) -> Self {
        Self {
            potential_index,
            fx_type,
            has_gap,
            k2,
            confirm: false,
        }
    }
}

#[derive(Debug)]
pub struct SegmentDetector {
    pub points: VecDeque<Point>,
    pub potential_state: Option<State>,
    pub state_for_case2: Option<State>,
    pub acc_count: usize,
}

impl SegmentDetector {
    pub fn new() -> Self {
        Self {
            points: VecDeque::new(),
            potential_state: None,
            state_for_case2: None,
            acc_count: 0,
        }
    }

    fn get_acc_len(&self) -> usize {
        self.points.len() + self.acc_count
    }

    fn post_case1_segement_comfired(&mut self) -> Option<SegmentEvent> {
        debug_assert!(self.state_for_case2.is_none());
        let start = self.points[0];
        let end_index = self.potential_state.as_ref().unwrap().potential_index;
        let end = self.points[end_index];
        let points = self.points.drain(0..end_index).collect();
        let event = SegmentEvent::New(start, end, points);
        // 清理工作
        self.potential_state = None;
        self.acc_count += end_index;

        println!("{}:case1 {:?}", self.get_acc_len(), event);
        Some(event)
    }

    fn post_case2_segement_comfired(&mut self) -> Option<SegmentEvent> {
        debug_assert!(self.potential_state.is_some());
        debug_assert!(self.state_for_case2.is_some());
        debug_assert!(
            self.potential_state.as_ref().unwrap().fx_type
                != self.state_for_case2.as_ref().unwrap().fx_type
        );

        let start = self.points[0];
        let end_index = self.potential_state.as_ref().unwrap().potential_index;
        let end = self.points[end_index];
        let end2_index = self.state_for_case2.as_ref().unwrap().potential_index;
        let end2 = self.points[end2_index];
        let points = self.points.drain(0..end_index).collect();

        let event = if !self.state_for_case2.as_ref().unwrap().has_gap {
            // 同时确认两个线段
            debug_assert!(end2.time == self.points[end2_index - end_index].time);
            let points2 = self.points.drain(0..(end2_index - end_index)).collect();
            let new2_event = SegmentEvent::New2(start, end, end2, points, points2);
            // 清理
            self.state_for_case2 = None;
            self.potential_state = None;

            self.acc_count += end2_index;
            println!("{}:case21 {:?}", self.get_acc_len(), new2_event);
            new2_event
        } else {
            // 只能确认前一个线段成立，当前线段有缺口，继续等待反向分型确认
            let new_event = SegmentEvent::New(start, end, points);
            // 清理工作
            self.potential_state = self.state_for_case2;
            self.potential_state.as_mut().unwrap().potential_index -= end_index;
            self.state_for_case2 = None;

            self.acc_count += end_index;
            println!("{}:case22 {:?}", self.get_acc_len(), new_event);
            new_event
        };

        Some(event)
    }

    pub fn when_odd(&mut self) -> Option<SegmentEvent> {
        // 奇数点：在有潜在点的情况下，找分型
        if self.potential_state.is_none() {
            return None;
        }

        let potential_state = self.potential_state.as_mut().unwrap();

        if !potential_state.confirm {
            let len = self.points.len();

            let direction = match potential_state.fx_type {
                FxType::Top => Direction::Up,
                FxType::Bottom => Direction::Down,
            };

            let merge = potential_state.k2.merge(
                self.points[len - 2].price,
                self.points[len - 1].price,
                direction,
            );

            if !merge {
                // 特征序列分型确认
                potential_state.confirm = true;
                println!(
                    "{}:潜在分型被确认 {}",
                    self.points.len() + self.acc_count,
                    self.points[potential_state.potential_index].time
                );
                if !potential_state.has_gap {
                    // 无缺口，线段成立
                    return self.post_case1_segement_comfired();
                }
            }
            return None;
        }

        debug_assert!(potential_state.confirm && potential_state.has_gap);

        // 有缺口情况，等反向分型成立
        if self.state_for_case2.is_none() {
            return None;
        }

        let case2_state = self.state_for_case2.as_mut().unwrap();
        if !case2_state.confirm {
            let len = self.points.len();
            let direction = match case2_state.fx_type {
                FxType::Top => Direction::Up,
                FxType::Bottom => Direction::Down,
            };

            let merge = case2_state.k2.merge(
                self.points[len - 2].price,
                self.points[len - 1].price,
                direction,
            );

            if !merge {
                // 特征序列分型确认
                case2_state.confirm = true;

                return self.post_case2_segement_comfired();
            }
        }

        None
    }

    fn check_potential_point_is_broken(&self) -> bool {
        let state = self.potential_state.as_ref().unwrap();

        let extreme_price = self.points[state.potential_index].price;
        let now_price = self.points[self.points.len() - 1].price;
        let fx_type = state.fx_type;

        let is_break = match fx_type {
            FxType::Top => now_price > extreme_price,
            FxType::Bottom => now_price < extreme_price,
        };

        is_break
    }

    fn check_fx1_is_broken(&mut self) {
        self.potential_state.as_ref().unwrap().potential_index;

        if self.check_potential_point_is_broken() {
            println!(
                "{}:分型被破坏 {}",
                self.points.len() + self.acc_count,
                self.points[self.potential_state.as_ref().unwrap().potential_index].time
            );
            self.potential_state = None;
        }
    }

    pub fn when_even(&mut self) -> Option<SegmentEvent> {
        // 偶数点：
        let mut start = 0;
        if self.potential_state.is_some() {
            start = self.potential_state.as_ref().unwrap().potential_index;

            if self.check_potential_point_is_broken() {
                println!(
                    "{}:分型被破坏 {}",
                    self.points.len() + self.acc_count,
                    self.points[self.potential_state.as_ref().unwrap().potential_index].time
                );
                self.potential_state = None;
                return None;
            }
        }

        let state = self.find_potential_fx(start);

        if state.is_some() {
            println!(
                "{}:找到潜在分型 {} \n{:?}",
                self.points.len() + self.acc_count,
                self.points[state.as_ref().unwrap().potential_index].time,
                state
            );
            if self.potential_state.is_none() {
                self.potential_state = state;
            } else {
                self.state_for_case2 = state;
            };
        }

        None
    }

    pub fn process(&mut self) -> Option<SegmentEvent> {
        /*let mut len = self.points.len();

        if len < 6 {
            return None;
        }

        if self.potential_state.is_some() && self.potential_state.as_ref().unwrap().confirm {
            len -= self.potential_state.as_ref().unwrap().potential_index;
        }

        if len % 2 == 0 {
                    self.when_even()
                } else {
                    self.when_odd()
                }
        */

        let has_potential1 = self.potential_state.is_some();

        let has_confirm_fx1 = if self.potential_state.is_some() {
            self.potential_state.as_ref().unwrap().confirm
        } else {
            false
        };

        let has_gap1 = if self.potential_state.is_some() {
            self.potential_state.as_ref().unwrap().has_gap
        } else {
            false
        };

        let has_potential2 = self.state_for_case2.is_some();

        let has_confirm_fx2 = if self.state_for_case2.is_some() {
            self.state_for_case2.as_ref().unwrap().confirm
        } else {
            false
        };

        let has_gap2 = if self.state_for_case2.is_some() {
            self.state_for_case2.as_ref().unwrap().has_gap
        } else {
            false
        };

        match (
            has_potential1,
            has_confirm_fx1,
            has_gap1,
            has_potential2,
            has_confirm_fx2,
            has_gap2,
        ) {
            (true, true, true, true, false, _) => {
                self.check_fx1_is_broken();
                self.search_fx2_confirm();
            }
            (true, true, true, true, true, _) => {
                //self.check_fx1_is_broken();
                return self.post_case2_segement_comfired();
            }
            (true, true, true, false, _, _) => {
                self.check_fx1_is_broken();
                self.search_fx2();
            }
            (true, true, false, _, _, _) => {
                // 无缺口，线段成立
                return self.post_case1_segement_comfired();
            }
            (true, false, _, _, _, _) => {
                self.check_fx1_is_broken();
                self.search_fx1_confirm();
            }

            (false, _, _, _, _, _) => {
                self.search_fx1();
            }
        }

        None
    }

    // 入口
    pub fn on_pen_event(&mut self, pen_event: PenEvent) -> Option<SegmentEvent> {
        match pen_event {
            PenEvent::First(a, b) => {
                self.points.push_back(a);
                self.points.push_back(b);
                None
            }

            PenEvent::New(a) => {
                // New事件代表新的一笔产生了，参数a是新的笔端点，但是由于该笔可能延伸，所以先处理原有的笔，然后将新的端点保存
                // 后续在UpdateTo事件里更新最后一个端点，知道新的PenEvent::New事件产生，代表该端点已经完成
                let event = self.process();
                self.points.push_back(a);
                event
            }

            PenEvent::UpdateTo(a) => {
                self.points.pop_back();
                self.points.push_back(a);
                None
            }
        }
    }

    // 在最后5个点中，查找潜在的分界点
    fn find_potential_point(&self) -> Option<(FxType, usize)> {
        // 至少要6个点才能判断潜在点,且是偶数点
        // 奇数点是与线段方向不符的，不用考虑
        let mut len = self.points.len();

        let current_len = if self.potential_state.is_some() {
            len - self.potential_state.as_ref().unwrap().potential_index
        } else {
            len
        };

        if current_len < 5 {
            return None;
        }

        let p3 = &self.points[len - 1]; // 最后一个点
        let p2 = &self.points[len - 3];
        let p1 = &self.points[len - 5];

        let is_top = p1.price < p2.price && p2.price > p3.price;
        let is_bottom = p1.price > p2.price && p2.price < p3.price;

        if is_top || is_bottom {
            let direction = if is_top { FxType::Top } else { FxType::Bottom };
            return Some((direction, len - 3));
        }

        None
    }

    // 从start开始，查找分型
    fn find_potential_fx(&self, start: usize) -> Option<State> {
        // 1. 找潜在点
        let result = self.find_potential_point();

        if result.is_none() {
            return None;
        }

        let (fx_type, potential_index) = result.unwrap();
        let extreme_price = self.points[potential_index].price; // 分型的极值点
        let mut secondary_index = potential_index - 2;
        let mut secondary_price = self.points[secondary_index].price;

        let mut pos = secondary_index - 1;
        //debug_assert!(pos % 2 == 0);

        // 2. 找次高点或者次低点
        while pos > ((start + 1) / 2) * 2 {
            if fx_type == FxType::Top {
                // 这里有个特例要处理
                if self.points[pos].price > extreme_price {
                    break;
                }
                if self.points[secondary_index].price > secondary_price {
                    secondary_price = self.points[secondary_index].price;
                    secondary_index = pos;
                }
            } else {
                // 这里有个特例要处理
                if self.points[pos].price < extreme_price {
                    break;
                }
                if self.points[secondary_index].price < secondary_price {
                    secondary_price = self.points[secondary_index].price;
                    secondary_index = pos;
                }
            }

            pos -= 2;
        }

        // update has_gap flag
        let to_price = self.points[potential_index + 1].price;

        let has_gap = if fx_type == FxType::Top {
            if secondary_price < to_price {
                true
            } else {
                false
            }
        } else {
            if secondary_price > to_price {
                true
            } else {
                false
            }
        };

        let k2 = Seq::new(extreme_price, to_price);

        Some(State::new(potential_index, fx_type, has_gap, k2))
    }

    fn search_fx1(&mut self) {
        let state = self.find_potential_fx(0);
        if state.is_some() {
            println!(
                "{}:找到潜在分型 {} \t {:?}",
                self.points.len() + self.acc_count,
                self.points[state.as_ref().unwrap().potential_index].time,
                state
            );
            debug_assert!(self.potential_state.is_none());
            self.potential_state = state;
        }
    }

    fn search_fx2(&mut self) {
        let start = self.potential_state.as_ref().unwrap().potential_index;
        let state = self.find_potential_fx(start);
        if state.is_some() {
            println!(
                "{}:找到潜在分型 {} \t {:?}",
                self.points.len() + self.acc_count,
                self.points[state.as_ref().unwrap().potential_index].time,
                state
            );
            debug_assert!(self.potential_state.is_some());
            debug_assert!(
                self.potential_state.as_ref().unwrap().fx_type != state.as_ref().unwrap().fx_type
            );
            self.state_for_case2 = state;
        }
    }

    fn search_fx1_confirm(&mut self) {
        let potential_state = self.potential_state.as_mut().unwrap();

        let len = self.points.len();

        let direction = match potential_state.fx_type {
            FxType::Top => Direction::Up,
            FxType::Bottom => Direction::Down,
        };

        let merge = potential_state.k2.merge(
            self.points[len - 2].price,
            self.points[len - 1].price,
            direction,
        );

        if !merge {
            // 特征序列分型确认
            potential_state.confirm = true;
            println!(
                "{}:潜在分型被确认 {}",
                self.points.len() + self.acc_count,
                self.points[potential_state.potential_index].time
            );
        }
    }

    fn search_fx2_confirm(&mut self) {
        let potential_state = self.state_for_case2.as_mut().unwrap();

        let len = self.points.len();

        let direction = match potential_state.fx_type {
            FxType::Top => Direction::Up,
            FxType::Bottom => Direction::Down,
        };

        let merge = potential_state.k2.merge(
            self.points[len - 2].price,
            self.points[len - 1].price,
            direction,
        );

        if !merge {
            // 特征序列分型确认
            potential_state.confirm = true;
            println!(
                "{}:潜在分型被确认 {}",
                self.points.len() + self.acc_count,
                self.points[potential_state.potential_index].time
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::test_util::*;

    #[test]
    fn test_sd_detector() {
        // 构建复杂线段测试数据，图见线段分类实例
        // 最终结果是线段1-4-13-16-19-22
        let vec_pen_events = vec![
            (1, 100.0),
            (2, 200.0),
            (3, 150.0),
            (4, 1000.0),
            (5, 900.0),
            (6, 950.0),
            (7, 700.0),
            (8, 800.0),
            (9, 750.0),
            (10, 850.0),
            (11, 600.0),
            (12, 650.0),
            (13, 400.0),
            (14, 500.0),
            (15, 450.0),
            (16, 610.0),
            (17, 480.0),
            (18, 550.0),
            (19, 50.0),
            (20, 625.0),
            (21, 500.0),
            (22, 800.0),
        ];
        let mut pen_events: Vec<PenEvent> = Vec::new();
        pen_events.push(PenEvent::First(
            Point::new(vec_pen_events[0].0, vec_pen_events[0].1),
            Point::new(vec_pen_events[1].0, vec_pen_events[1].1),
        ));

        for i in 2..vec_pen_events.len() {
            let p = Point::new(vec_pen_events[i].0, vec_pen_events[i].1);
            pen_events.push(PenEvent::New(p));
        }

        assert!(pen_events.len() == 21);

        // 开始处理事件
        let mut segment_events: Vec<SegmentEvent> = Vec::new();
        let mut sd = SegmentDetector::new();

        for pen_event in pen_events {
            /*match pen_event {
                PenEvent::New(p) => print!("({})", p.time),
                PenEvent::First(p1, p2) => print!("({}-{})", p1.time, p2.time),
                _ => {}
            };*/

            let seg_event = sd.on_pen_event(pen_event);
            if seg_event.is_some() {
                segment_events.push(seg_event.unwrap());
            }
        }
        println!("Segment Event Count {}", segment_events.len());
        //assert!(segment_events.len() == 4);
    }

    #[test]
    fn test_eurusd2021_detector() {
        let bars = load_eurusd_2021();
        let mut fd = m0::analyzer::Analyzer::new();
        let mut pd = m1::analyzer::Analyzer::new();
        let mut sd = SegmentDetector::new();
        let mut seg_count = 0;
        for bar in &bars {
            let f = fd.on_new_bar(bar);
            if let Some(fx) = f {
                let pen_event = pd.on_new_fractal(fx);
                if pen_event.is_some() {
                    let seg_event = sd.on_pen_event(pen_event.unwrap());
                    if seg_event.is_some() {
                        seg_count += 1;
                    }
                }
            }
        }
        println!("Segment Event Count {}", seg_count);
    }
}
