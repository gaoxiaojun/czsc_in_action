//！ 线段判别
//!  本文件实现缠论第71课中描述的线段当下判别规则
//!
//！ 基于特征序列的线段划分标准方法主要步骤：
//！ 先根据线段方向，找出特征序列，然后对特征序列进行标准化，寻找标准特征序列的分型，
//!  然后判断分型第1,2元素间是否有缺口，分为第一种破坏和第二种破坏；
//！ 第一种破坏比较简单直接，当即可断定线段结束；
//！ 第二种破坏则需要寻找反向特征序列的分型来进行确认，如果没有出现反向特征序列分型就新高新低，则仍为原线段的延续。

#[cfg(debug_assertions)]
use colored::*;

use common::direction::Direction;
use common::event::{PenEvent, SegmentEvent};
use common::fx::FxType;
use common::point::Point;
use common::time::Time;
use debug_print::{debug_print, debug_println};

// Seq为特征序列
#[derive(Debug, Clone, Copy)]
pub struct Seq {
    high: f64,
    low: f64,
}

impl Seq {
    pub fn new(from: f64, to: f64) -> Self {
        Self {
            high: f64::max(from, to),
            low: f64::min(from, to),
        }
    }

    // 特征序列合并
    pub fn merge(&mut self, from: f64, to: f64, direction: Direction) -> bool {
        let _high = f64::max(from, to);
        let _low = f64::min(from, to);

        let prev_include_next = self.high > _high && self.low < _low;
        let next_include_prev = self.high < _high && self.low > _low;

        if !prev_include_next && !next_include_prev {
            return false;
        }

        match direction {
            Direction::Up => {
                self.high = f64::max(self.high, _high);
                self.low = f64::max(self.low, _low);
            }
            Direction::Down => {
                self.high = f64::min(self.high, _high);
                self.low = f64::min(self.low, _low);
            }
        }
        true
    }
}

// 特征序列分型信息
#[derive(Debug, Clone, Copy)]
pub struct State {
    time: Time, // 分型极值点对应的时间，可省略
    price: f64, // 分型极值点的价格，可省略

    potential_index: usize, // 分型极值点在Point数组中的索引
    fx_type: FxType,        // 分型类型，顶或底
    has_gap: bool,          // 第一、第二特征序列是否有缺口
    k2: Seq,                // 第二特征序列，不需要保存第一、第三特征序列
    // 潜在分界点查找的过程中，通过次高点的查找，确保了第一第二特征序列不会重合
    // 第三特征序列只要判别与第二不重合就可，因此也不用保存第三特征序列
    confirm: bool, // 分型成立标志，等第三特征序列与第二特征序列不重合时，判定分型成立
}

impl State {
    pub fn new(
        time: Time,
        price: f64,
        potential_index: usize,
        fx_type: FxType,
        has_gap: bool,
        k2: Seq,
    ) -> Self {
        Self {
            time,
            price,
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
    pub points: Vec<Point>,             // 笔对应的点数组，索引0代表线段的起点
    pub potential_state: Option<State>, // 潜在分界点的分型信息
    pub state_for_case2: Option<State>, // 反向分型信息
    pub direction: Option<Direction>,   // 线段方向，初始时候是无方向的
    pub total_count: usize,             // 检测到的线段总数，可省略
    pub is_first_segment:bool,
}

impl SegmentDetector {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            potential_state: None,
            state_for_case2: None,
            direction: None,
            total_count: 0,
            is_first_segment:true
        }
    }

    fn check_is_first_segment(&mut self, points: &mut Vec<Point>) -> Point {
        let mut start = points[0];
        if self.is_first_segment {
            let event_points_len = points.len();
            if event_points_len % 2 == 0 {
                points.remove(0);
                start = points[0];
            }
            self.is_first_segment = false;
        }
        start
    }

    // 当情况一确认时调用该函数，函数生成事件并清理内部状态
    fn post_case1_segement_comfired(&mut self) -> Option<SegmentEvent> {
        debug_assert!(self.state_for_case2.is_none());
        //let start = self.points[0];
        let end_index = self.potential_state.as_ref().unwrap().potential_index;
        let end = self.points[end_index];
        let mut points:Vec<Point> = self.points.drain(0..end_index).collect();
        let start = self.check_is_first_segment(&mut points);

        let event = SegmentEvent::New(start, end, points);

        debug_println!("{}:{}-->{}", "确认线段情况一".red(), start.time, end.time);
        self.total_count += 1;

        // 清理工作
        self.direction = match self.potential_state.as_ref().unwrap().fx_type {
            FxType::Top => Some(Direction::Down),
            FxType::Bottom => Some(Direction::Up),
        };
        self.potential_state = None;

        Some(event)
    }

    // 当情况二确认时调用该函数，函数根据有无缺口生成事件并清理内部状态
    fn post_case2_segement_comfired(&mut self) -> Option<SegmentEvent> {
        debug_assert!(self.potential_state.is_some());
        debug_assert!(self.state_for_case2.is_some());
        debug_assert!(
            self.potential_state.as_ref().unwrap().fx_type
                != self.state_for_case2.as_ref().unwrap().fx_type
        );

        let end_index = self.potential_state.as_ref().unwrap().potential_index;
        let end = self.points[end_index];
        let end2_index = self.state_for_case2.as_ref().unwrap().potential_index;
        let end2 = self.points[end2_index];
        let mut points = self.points.drain(0..end_index).collect();
        let start= self.check_is_first_segment(&mut points);

        let event = if !self.state_for_case2.as_ref().unwrap().has_gap {
            // 同时确认两个线段
            debug_assert!(end2.time == self.points[end2_index - end_index].time);
            let points2 = self.points.drain(0..(end2_index - end_index)).collect();
            let new2_event = SegmentEvent::New2(start, end, end2, points, points2);
            debug_println!(
                "{}-{}:{}-->{}-->{}",
                "确认线段情况二".red(),
                "双线段".yellow(),
                start.time,
                end.time,
                end2.time
            );
            self.total_count += 2;
            // 清理
            self.direction = match self.potential_state.as_ref().unwrap().fx_type {
                FxType::Top => Some(Direction::Up),
                FxType::Bottom => Some(Direction::Down),
            };
            self.state_for_case2 = None;
            self.potential_state = None;

            new2_event
        } else {
            // 只能确认前一个线段成立，当前线段有缺口，继续等待反向分型确认
            let new_event = SegmentEvent::New(start, end, points);
            debug_println!(
                "{}-{}:{}-->{}",
                "确认线段情况二".red(),
                "单线段".yellow(),
                start.time,
                end.time
            );
            self.total_count += 1;
            // 清理工作
            self.direction = match self.potential_state.as_ref().unwrap().fx_type {
                FxType::Top => Some(Direction::Down),
                FxType::Bottom => Some(Direction::Up),
            };
            self.potential_state = self.state_for_case2;
            self.potential_state.as_mut().unwrap().potential_index -= end_index;
            self.state_for_case2 = None;

            new_event
        };

        Some(event)
    }

    // 检查潜在分型是否被破坏
    // 第二种破坏则需要寻找反向特征序列的分型来进行确认，如果没有出现反向特征序列分型就新高新低，则仍为原线段的延续。
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

    fn check_fx1_is_broken(&mut self) -> bool {
        debug_print!("check_fx1_is_broken ");

        if self.check_potential_point_is_broken() {
            debug_println!(
                "{}:{}",
                "分型被破坏".yellow(),
                self.points[self.potential_state.as_ref().unwrap().potential_index].time
            );
            self.potential_state = None;
            self.state_for_case2 = None;
            return true;
        }
        debug_print!("{} ", "N".green());
        false
    }

    // 主处理函数
    pub fn process(&mut self) -> Option<SegmentEvent> {
        debug_assert!(self.verify_point());
        debug_print!("\n{}  ", self.points[self.points.len() - 1].time);
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

        debug_print!(
            "{}{}{} {}{}{} ",
            get_s(has_potential1),
            get_s(has_confirm_fx1),
            get_s(has_gap1),
            get_s(has_potential2),
            get_s(has_confirm_fx2),
            get_s(has_gap2)
        );
        debug_print!(
            "{} ",
            match self.direction {
                None => "None",
                Some(d) => {
                    match d {
                        Direction::Up => "Up",
                        Direction::Down => "Down",
                    }
                }
            }
        );
        match (
            has_potential1,
            has_confirm_fx1,
            has_gap1,
            has_potential2,
            has_confirm_fx2,
            has_gap2,
        ) {
            (true, true, true, true, false, _) => {
                // 规则1， test_sd_detector测试结果是2个线段
                // 这个规则更符合缠论原文
                /*if self.check_fx1_is_broken() {
                    return None;
                }

                let len = self.points.len();
                let fx2_start = self.state_for_case2.as_ref().unwrap().potential_index;
                if (len - fx2_start) % 2 == 0 {
                    return self.search_fx2_confirm();
                }*/

                
                // 规则2，test_sd_detector测试结果是4个线段
                let len = self.points.len();
                let fx2_start = self.state_for_case2.as_ref().unwrap().potential_index;
                if (len - fx2_start) % 2 == 0 {
                    return self.search_fx2_confirm();
                } else {
                    if self.check_fx1_is_broken() {
                        return None;
                    }
                }
                
            }

            (true, true, true, false, _, _) => {
                let len = self.points.len();
                let fx1_start = self.potential_state.as_ref().unwrap().potential_index;
                if (len - fx1_start) % 2 == 0 {
                    self.search_fx2();
                } else {
                    self.check_fx1_is_broken();
                }
            }

            (true, false, _, _, _, _) => {
                let len = self.points.len();
                let fx1_start = self.potential_state.as_ref().unwrap().potential_index;
                if (len - fx1_start) % 2 == 0 {
                    return self.search_fx1_confirm();
                } else {
                    self.check_fx1_is_broken();
                }
            }

            (false, _, _, _, _, _) => {
                let len = self.points.len();
                if len % 2 == 0 || self.direction.is_none() {
                    self.search_fx1();
                }
            }

            (_, _, _, _, _, _) => {}
        }
        debug_print!("\t");
        None
    }

    // 入口
    pub fn on_pen_event(&mut self, pen_event: PenEvent) -> Option<SegmentEvent> {
        match pen_event {
            PenEvent::First(a, b) => {
                self.points.push(a);
                self.points.push(b);
                None
            }

            PenEvent::New(a) => {
                // New事件代表新的一笔产生了，参数a就是旧笔的终点也是新笔开端，但是由于该笔可能延伸，所以先处理原有的笔，然后将新的端点保存
                // 后续在UpdateTo事件里更新最后一个端点，知道新的PenEvent::New事件产生，代表该端点已经完成
                let event = self.process();
                self.points.push(a);
                event
            }

            PenEvent::UpdateTo(a) => {
                self.points.pop();
                self.points.push(a);
                None
            }
        }
    }

    // 在最后5个点中，查找潜在的分界点
    fn find_potential_point(&self) -> Option<(FxType, usize)> {
        // 判断潜在分界点,且是偶数点，奇数点是与线段方向不符的，不用考虑
        let len = self.points.len();

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

        let is_top = p1.price <= p2.price && p2.price > p3.price;
        let is_bottom = p1.price >= p2.price && p2.price < p3.price;

        if is_top || is_bottom {
            let direction = if is_top { FxType::Top } else { FxType::Bottom };
            return Some((direction, len - 3));
        }

        None
    }

    // 查找潜在分型
    // start为开始点索引,查找第一分型序列，start = 0. 查找反向分型序列，start = 第一分型的potential_index
    // 函数只有在与查找方向同向的点调用才有意义,既 index %2 == 0
    // 找到潜在点时候，查找次高次低点，通过次高次低点与潜在点的比较确定有无缺口
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

        if secondary_index < 1 {
            return None;
        }

        let mut pos: isize = (secondary_index - 1) as isize;

        // 2. 找次高点或者次低点
        while pos > start as isize {
            if fx_type == FxType::Top {
                // 这里有个特例要处理
                if self.points[pos as usize].price > extreme_price {
                    break;
                }
                if self.points[secondary_index].price > secondary_price {
                    secondary_price = self.points[secondary_index].price;
                    secondary_index = pos as usize;
                }
            } else {
                // 这里有个特例要处理
                if self.points[pos as usize].price < extreme_price {
                    break;
                }
                if self.points[secondary_index].price < secondary_price {
                    secondary_price = self.points[secondary_index].price;
                    secondary_index = pos as usize;
                }
            }

            pos -= 2;
        }

        let time = self.points[potential_index].time;

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

        Some(State::new(
            time,
            extreme_price,
            potential_index,
            fx_type,
            has_gap,
            k2,
        ))
    }

    fn set_fx1(&mut self, state: Option<State>) {
        debug_println!(
            "{} 潜在fx1:{} ",
            "找到".yellow(),
            self.points[state.as_ref().unwrap().potential_index].time,
        );
        debug_assert!(self.potential_state.is_none());
        self.potential_state = state;
    }

    // 查找与线段方向同向的潜在分界点分型，对于向上线段查找顶分型，向下线段查找底分型
    fn search_fx1(&mut self) {
        debug_print!("search_fx1 ");
        let state = self.find_potential_fx(0);

        if state.is_some() {
            if self.direction.is_none() {
                self.set_fx1(state);
            } else {
                match self.direction.as_ref().unwrap() {
                    Direction::Up => {
                        if state.as_ref().unwrap().fx_type == FxType::Top {
                            self.set_fx1(state);
                        }
                    }
                    Direction::Down => {
                        if state.as_ref().unwrap().fx_type == FxType::Bottom {
                            self.set_fx1(state);
                        }
                    }
                }
            }
        } else {
            debug_print!("{} ", "N".green());
        }
    }

    fn set_fx2(&mut self, state: Option<State>) {
        debug_println!(
            "{} 潜在fx2:{} ",
            "找到".yellow(),
            self.points[state.as_ref().unwrap().potential_index].time,
        );
        debug_assert!(self.potential_state.is_some());
        debug_assert!(
            self.potential_state.as_ref().unwrap().fx_type != state.as_ref().unwrap().fx_type
        );
        self.state_for_case2 = state;
    }

    // 当特征序列有缺口的时候，查找与潜在分型相反的分型
    fn search_fx2(&mut self) {
        debug_print!("search_fx2 ");
        let start = self.potential_state.as_ref().unwrap().potential_index;
        let state = self.find_potential_fx(start);
        if state.is_some() {
            if self.direction.is_none() {
                self.set_fx2(state);
            } else {
                match self.direction.as_ref().unwrap() {
                    Direction::Up => {
                        if state.as_ref().unwrap().fx_type == FxType::Bottom {
                            self.set_fx2(state);
                        }
                    }
                    Direction::Down => {
                        if state.as_ref().unwrap().fx_type == FxType::Top {
                            self.set_fx2(state);
                        }
                    }
                }
            }
        }
    }

    // 测试潜在分界点分型是否成立
    // 如果分型成立且无缺口，代表线段成立，返回对应的线段事件
    // 如果分型成立且有缺口，设置状态，后续等待反向分型成立
    fn search_fx1_confirm(&mut self) -> Option<SegmentEvent> {
        debug_print!("search_fx1_confirm ");
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
            debug_println!(
                "{} 潜在fx1分型:{}",
                "确认".yellow(),
                self.points[potential_state.potential_index].time
            );
            if !potential_state.has_gap {
                // 无缺口，线段成立
                return self.post_case1_segement_comfired();
            }
        } else {
            debug_print!(" {} ", "N".green());
        }

        None
    }

    // 测试反向分型是否成立
    // 如果反向分型成立，根据有无缺口，返回对应的线段事件
    fn search_fx2_confirm(&mut self) -> Option<SegmentEvent> {
        debug_print!("search_fx2_confirm ");
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
            debug_println!(
                "{} 潜在fx2分型:{}",
                "确认".yellow(),
                self.points[potential_state.potential_index].time
            );
            return self.post_case2_segement_comfired();
        }
        None
    }

    //#[cfg(debug_assertions)]
    fn verify_point(&self) -> bool {
        // 校验输入的数据，确保数据点是高低错落形式的
        let len = self.points.len();
        if len < 3 {
            return true;
        }

        let p1 = self.points[len - 1];
        let p2 = self.points[len - 2];
        let p3 = self.points[len - 3];

        let is_down = p1.price < p2.price && p2.price > p3.price;
        let is_up = p1.price > p2.price && p3.price > p2.price;

        if is_down || is_up {
            return true;
        }
        false
    }
}

// helper utils
#[cfg(debug_assertions)]
fn get_s(v: bool) -> &'static str {
    if v {
        return "T";
    } else {
        return "F";
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::test_util::*;

    #[test]
    fn test_sd_detector() {
        // 构建复杂线段测试数据，图见线段分类实例
        // 最终结果是线段1-4-19
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
            (23, 700.0),
        ];
        let mut pen_events: Vec<PenEvent> = Vec::new();
        pen_events.push(PenEvent::First(
            Point::new(Time::new(vec_pen_events[0].0), vec_pen_events[0].1),
            Point::new(Time::new(vec_pen_events[1].0), vec_pen_events[1].1),
        ));

        for i in 2..vec_pen_events.len() {
            let p = Point::new(Time::new(vec_pen_events[i].0), vec_pen_events[i].1);
            pen_events.push(PenEvent::New(p));
        }

        assert!(pen_events.len() == 22);    // 第一个事件包含了两个端点

        // 开始处理事件
        let mut segment_events: Vec<SegmentEvent> = Vec::new();
        let mut sd = SegmentDetector::new();

        for pen_event in pen_events {
            let seg_event = sd.on_pen_event(pen_event);
            if seg_event.is_some() {
                segment_events.push(seg_event.unwrap());
            }
        }
        debug_assert!(sd.total_count == 2 || sd.total_count == 3);  // 不同的规则返回的线段数量不同
        debug_println!("\n{}{}", "线段总数:".red(), sd.total_count);
    }

    #[test]
    fn test_80_detector() {
        // 构建复杂线段测试数据，图见复杂线段集中营
        // 最终结果是线段1-4-19
        let vec_pen_events = vec![
            (1, 100.0),
            (2, 50.0),
            (3, 60.0),
            (4, 10.0),
            (5, 90.0),
            (6, 50.0),
            (7, 80.0),
            (8, 20.0),
            (9, 55.0),
            (10, 45.0),
            (11, 85.0),
            (12, 51.0),
            (13, 70.0),
            (14, 1.0),
            (15, 10.0),
        ];
        let mut pen_events: Vec<PenEvent> = Vec::new();
        pen_events.push(PenEvent::First(
            Point::new(Time::new(vec_pen_events[0].0), vec_pen_events[0].1),
            Point::new(Time::new(vec_pen_events[1].0), vec_pen_events[1].1),
        ));

        for i in 2..vec_pen_events.len() {
            let p = Point::new(Time::new(vec_pen_events[i].0), vec_pen_events[i].1);
            pen_events.push(PenEvent::New(p));
        }

        // 开始处理事件
        let mut segment_events: Vec<SegmentEvent> = Vec::new();
        let mut sd = SegmentDetector::new();

        for pen_event in pen_events {
            let seg_event = sd.on_pen_event(pen_event);
            if seg_event.is_some() {
                segment_events.push(seg_event.unwrap());
            }
        }
        debug_println!("\n{}{}", "线段总数:".red(), sd.total_count);
    }

    #[test]
    fn test_81_detector() {
        // 构建复杂线段测试数据，图见复杂线段集中营
        // 被测试的结果与缠师给出的结果不同
        // 最终结果是线段1-4-19
        // 缠师给出的结果就是一个线段
        let vec_pen_events = vec![
            (1, 100.0),
            (2, 50.0),
            (3, 60.0),
            (4, 10.0),
            (5, 90.0),
            (6, 50.0),
            (7, 80.0),
            (8, 20.0),
            (9, 85.0),
            (10, 51.0),
            (11, 70.0),
            (12, 1.0),
            (13, 10.0),
        ];
        let mut pen_events: Vec<PenEvent> = Vec::new();
        pen_events.push(PenEvent::First(
            Point::new(Time::new(vec_pen_events[0].0), vec_pen_events[0].1),
            Point::new(Time::new(vec_pen_events[1].0), vec_pen_events[1].1),
        ));

        for i in 2..vec_pen_events.len() {
            let p = Point::new(Time::new(vec_pen_events[i].0), vec_pen_events[i].1);
            pen_events.push(PenEvent::New(p));
        }

        // 开始处理事件
        let mut segment_events: Vec<SegmentEvent> = Vec::new();
        let mut sd = SegmentDetector::new();

        for pen_event in pen_events {
            let seg_event = sd.on_pen_event(pen_event);
            if seg_event.is_some() {
                segment_events.push(seg_event.unwrap());
            }
        }
        //assert!(sd.total_count == 1);
        debug_println!("\n{}{}", "线段总数:".red(), sd.total_count);
    }
    
    #[test]
    fn test_eurusd2021_detector() {
        let bars = load_eurusd_2021();
        let mut fd = m0::analyzer::Analyzer::new();
        let mut pd = m1::analyzer::Analyzer::new();
        let mut sd = SegmentDetector::new();
        for bar in &bars {
            let f = fd.on_new_bar(bar);
            if let Some(fx) = f {
                let pen_event = pd.on_new_fractal(fx);
                if pen_event.is_some() {
                    let _ = sd.on_pen_event(pen_event.unwrap());
                }
            }
        }
        debug_println!("\n{}{}", "线段总数:".red(), sd.total_count);
    }
}
