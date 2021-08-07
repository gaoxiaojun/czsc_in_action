#[cfg(debug_assertions)]
use colored::*;

use common::direction::Direction;
use common::event::{PenEvent, SegmentEvent};
use common::fx::FxType;
use common::point::Point;
use common::time::Time;
use debug_print::{debug_print, debug_println};

#[cfg(debug_assertions)]
fn get_s(v: bool) -> &'static str {
    if v {
        return "T";
    } else {
        return "F";
    }
}

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

#[derive(Debug, Clone, Copy)]
pub struct State {
    time: Time,
    price: f64,
    potential_index: usize,
    fx_type: FxType,
    has_gap: bool,
    k2: Seq,
    confirm: bool,
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
    pub points: Vec<Point>,
    pub potential_state: Option<State>,
    pub state_for_case2: Option<State>,
    pub direction: Option<Direction>,
    pub total_count: usize,
}

impl SegmentDetector {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            potential_state: None,
            state_for_case2: None,
            direction: None,
            total_count: 0,
        }
    }

    fn post_case1_segement_comfired(&mut self) -> Option<SegmentEvent> {
        debug_assert!(self.state_for_case2.is_none());
        let start = self.points[0];
        let end_index = self.potential_state.as_ref().unwrap().potential_index;
        let end = self.points[end_index];
        let points = self.points.drain(0..end_index).collect();
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
            debug_println!("{}-{}:{}-->{}", "确认线段情况二".red(), "单线段".yellow(), start.time, end.time);
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
        self.potential_state.as_ref().unwrap().potential_index;

        if self.check_potential_point_is_broken() {
            debug_println!(
                "分型被破坏 {}",
                self.points[self.potential_state.as_ref().unwrap().potential_index].time
            );
            self.potential_state = None;
            self.state_for_case2 = None;
            return true;
        }
        debug_print!("{} ", "N".green());
        false
    }

    pub fn process(&mut self) -> Option<SegmentEvent> {
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
        debug_print!("{:?} ", self.direction);
        match (
            has_potential1,
            has_confirm_fx1,
            has_gap1,
            has_potential2,
            has_confirm_fx2,
            has_gap2,
        ) {
            (true, true, true, true, false, _) => {
                //规则1， 测试结果是2个线段
                if self.check_fx1_is_broken() {
                    return None;
                }
                let len = self.points.len();
                let fx2_start = self.state_for_case2.as_ref().unwrap().potential_index;
                if (len - fx2_start) % 2 == 0 {
                    return self.search_fx2_confirm();
                } 
                
                /* 
                // 规则2，测试结果是4个线段
                let len = self.points.len();
                let fx2_start = self.state_for_case2.as_ref().unwrap().potential_index;
                if (len - fx2_start) % 2 == 0 {
                    return self.search_fx2_confirm();
                } else {
                    if self.check_fx1_is_broken() {
                        return None;
                    }
                }   
                */             
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
                    if self.check_fx1_is_broken() {
                        return None;
                    }
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
        // 至少要6个点才能判断潜在点,且是偶数点
        // 奇数点是与线段方向不符的，不用考虑
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
                        // 忽略底分型
                    }
                    Direction::Down => {
                        if state.as_ref().unwrap().fx_type == FxType::Bottom {
                            self.set_fx1(state);
                        }
                    }
                }
            }
        }
        else {
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
                        // 忽略底分型
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
            Point::new(vec_pen_events[0].0, vec_pen_events[0].1),
            Point::new(vec_pen_events[1].0, vec_pen_events[1].1),
        ));

        for i in 2..vec_pen_events.len() {
            let p = Point::new(vec_pen_events[i].0, vec_pen_events[i].1);
            pen_events.push(PenEvent::New(p));
        }

        assert!(pen_events.len() == 22);

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
    fn test_80_detector() {
        // 构建复杂线段测试数据，图见线段分类实例
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
            Point::new(vec_pen_events[0].0, vec_pen_events[0].1),
            Point::new(vec_pen_events[1].0, vec_pen_events[1].1),
        ));

        for i in 2..vec_pen_events.len() {
            let p = Point::new(vec_pen_events[i].0, vec_pen_events[i].1);
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
        // 构建复杂线段测试数据，图见线段分类实例
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
            (9, 85.0),
            (10, 51.0),
            (11, 70.0),
            (12, 1.0),
            (13, 10.0),
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
