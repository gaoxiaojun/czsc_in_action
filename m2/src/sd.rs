//! 实现线段规则总结.md

use colored::Colorize;
// Debug 输出：
// Time, Sd情况， 第一分析情况，第二分型情况
// Sd情况: 线段方向, raw_points长度， need_ws
// 第一序列分型_时间, 缺口， 队列长度
// 第二序列分型_时间 缺口
use common::{direction::Direction, event::{PenEvent, SegmentEvent}, fx::FxType, point::Point, print_flush, time::Time};

use crate::fx_detector::{FxWindow, SeqFx};
use crate::seq::Seq;

#[derive(Debug)]
pub struct SlidingWindow {
    pub base: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct SegmentDetectorV2 {
    direction: Option<Direction>,
    raw_points: Vec<Point>,
    start: usize,
    w1: FxWindow,
    w2: FxWindow,
    need_w2: bool,
    post_task: bool,
    potencial_fx: Option<SeqFx>,
    last_segment_start_time: Option<Time>,
}

impl SegmentDetectorV2 {
    pub fn new() -> Self {
        Self {
            direction: None,
            raw_points: Vec::new(),
            start: 0,
            w1: FxWindow::new(false), //第一序列，因此只执行前包含后的规则
            w2: FxWindow::new(true),  //第二序列，需要执行全包含规则
            need_w2: false,
            post_task: false,
            potencial_fx: None,
            last_segment_start_time: None,
        }
    }

    fn is_broken(&self) -> bool {
        let last = &self.raw_points[self.raw_points.len() - 1];
        let extrem_price = &self.potencial_fx.as_ref().unwrap().k2;
        match self.direction.unwrap() {
            Direction::Up => last.price > extrem_price.high(),
            Direction::Down => last.price < extrem_price.low(),
        }
    }

    fn find_first_segment(&mut self) {
        let len = self.raw_points.len();

        if len >= 4 {
            let p1 = self.raw_points[len - 4];
            let p2 = self.raw_points[len - 3];
            let p3 = self.raw_points[len - 2];
            let p4 = self.raw_points[len - 1];

            let direction = SegmentDetectorV2::is_first_segment(&p1, &p2, &p3, &p4);
            if direction.is_some() {
                // 当检测到线段时，去除线段起始点之前的Point
                let _ = self.raw_points.drain(..len - 4);
                debug_assert!(self.raw_points.len() == 4);
                // 新的线段成立，w1放入第一个特征序列
                let seq = Seq::new(1, &p2, &p3);
                self.w1.push(seq);
            }

            self.direction = direction;
        }
    }

    fn new_or_update_event(&self, start: &Point, end: &Point) -> SegmentEvent {
        let event = if self.last_segment_start_time.is_some()
            && self.last_segment_start_time.unwrap() == start.time
        {
            SegmentEvent::UpdateTo(end.clone())
        } else {
            SegmentEvent::New(start.clone(), end.clone())
        };

        event
    }

    fn seq_process(&mut self) -> Option<SegmentEvent> {
        // TODO
        //  有post_task标志
        // 设置需要检测第二特征序列，因为设置post_task是第二序列有缺口才设置的。
        // 因此这个时候，有分型，有缺口
        //    设置需要检测第二特征序列，清除第二特征序列缓冲队列

        let direction = self.direction.unwrap();
        let from = &self.raw_points[self.raw_points.len() - 2];
        let to = &self.raw_points[self.raw_points.len() - 1];

        // 线段向上 && 笔向下 或者相反，true代表是特征序列
        let is_sequence = (direction == Direction::Up && from.price > to.price)
            || (direction == Direction::Down && from.price < to.price);

        if is_sequence {
            // 笔方向与线段方向相反，该笔为特征序列
            if !self.need_w2 {
                // 等第二种情况下，第一特征序列暂定计算，等第二特征序列成立或者破坏第一序列的高低点
                print_flush!(
                    "{} ({}->{})(一)--> 线段:{}(rp:{} w1:{} w2:{})",
                    to.time,
                    from,
                    to,
                    self.direction.as_ref().unwrap(),
                    self.raw_points.len(),
                    self.w1.len(),
                    self.w2.len()
                );

                let seq = Seq::new(self.raw_points.len() - 2, &from, &to);

                let has_fx = self.w1.push(seq);

                if has_fx.is_none() {
                    println!("\t无分型");
                    return None;
                }

                let fx = has_fx.unwrap();
                let has_gap = match fx.fx_type {
                    FxType::Top => fx.k2.low() > fx.k1.high(),
                    FxType::Bottom => fx.k2.high() < fx.k1.low(),
                };

                print!(
                    "\t有分型: {} {} {}",
                    self.raw_points[fx.k2.from_index].time,
                    if fx.fx_type == FxType::Top {
                        "顶"
                    } else {
                        "底"
                    },
                    if has_gap { "有缺口" } else { "无缺口" }
                );

                let is_current_fx_type = match self.direction.unwrap() {
                    Direction::Up => fx.fx_type == FxType::Top,
                    Direction::Down => fx.fx_type == FxType::Bottom,
                };

                //debug_assert!(is_current_fx_type);

                if !is_current_fx_type {
                    print!(" {}\n", "忽略".cyan());
                    return None;
                } else {
                    print!("\n");
                }

                if has_gap {
                    // 第一特征序列有缺口，等第二特征序列来确认
                    self.need_w2 = true;
                    self.w2.clear();
                    self.potencial_fx = Some(fx.clone());
                    self.recalc_seq2();
                    return None;
                } else {
                    // 第一特征序列无缺口，可以确认线段
                    let start = &self.raw_points[0];
                    let end = &self.raw_points[fx.k2.from_index];

                    let event = self.new_or_update_event(start, end);
                    self.last_segment_start_time = Some(start.time);

                    println!("{}", event);

                    // 清理
                    let _ = self.raw_points.drain(..fx.k2.from_index);
                    self.w1.clear();
                    self.direction = match self.direction.unwrap() {
                        Direction::Down => Some(Direction::Up),
                        Direction::Up => Some(Direction::Down),
                    };
                    self.recalc_seq1();

                    return Some(event);
                }
            }
        } else {
            // 笔方向与线段方向相同
            if self.need_w2 {
                print_flush!(
                    "{} ({}->{})(二)--> 线段:{}(rp:{} w1:{} w2:{})",
                    to.time,
                    from,
                    to,
                    self.direction.as_ref().unwrap(),
                    self.raw_points.len(),
                    self.w1.len(),
                    self.w2.len()
                );
            }
            if self.need_w2 {
                if self.is_broken() {
                    // 第二分型未成形就创了新高新低
                    // TODO:如何清理？？？？0823凌晨
                    println!("第二分型未成形就创了新高新低 {}", to);
                    self.w2.clear();
                    self.need_w2 = false;
                    self.recalc_seq1(); // 第一特征序列在分型形成后就停止了计算，当线段延续的时候，需要重新计算
                    return None;
                }

                let has_fx = self
                    .w2
                    .push(Seq::new(self.raw_points.len() - 2, &from, &to));

                if has_fx.is_none() {
                    println!("\t无分型");
                    return None;
                }

                let fx = has_fx.unwrap();
                let has_gap = match fx.fx_type {
                    FxType::Top => fx.k2.low() > fx.k1.high(),
                    FxType::Bottom => fx.k2.high() < fx.k1.low(),
                };

                let is_current_fx_type = match self.direction.unwrap() {
                    Direction::Up => fx.fx_type == FxType::Bottom,
                    Direction::Down => fx.fx_type == FxType::Top,
                };

                //debug_assert!(is_current_fx_type);
                if !is_current_fx_type {
                    return None;
                }

                println!(
                    "\t有分型: {} {} {}",
                    self.raw_points[fx.k2.from_index].time,
                    if fx.fx_type == FxType::Top {
                        "顶"
                    } else {
                        "底"
                    },
                    if has_gap { "有缺口" } else { "无缺口" }
                );

                if has_gap {
                    // TODO:
                    // 第二特征序列有缺口，只能确认第一个线段成立
                    let start = &self.raw_points[0];
                    let fx1_index = self.potencial_fx.as_ref().unwrap().k2.from_index;
                    let fx1_end = &self.raw_points[fx1_index];
                    let event = self.new_or_update_event(start, fx1_end);
                    self.last_segment_start_time = Some(start.time);
                    println!("{}", event);
                    // 把第二特征序列复制到第一特征序列中，注意需要调整其中的from_index(这个可以通过改变Seq结构来达到不需要调整from_index)
                    self.w1.clear();
                    for index in 0..self.w2.len() {
                        let mut seq = self.w2[index].clone();
                        seq.from_index -= fx1_index;
                        self.w1.push(seq);
                    }

                    let fx = self.w1.check_fx();
                    debug_assert!(fx.is_some());
                    let fx = fx.unwrap();
                    let has_gap = match fx.fx_type {
                        FxType::Top => fx.k2.low() > fx.k1.high(),
                        FxType::Bottom => fx.k2.high() < fx.k1.low(),
                    };
                    debug_assert!(has_gap);
                    // 清理第二特征序列
                    self.need_w2 = true;
                    self.w2.clear();
                    // TODO:需要计算第二特征序列
                    return Some(event);
                } else {
                    // 第二特征序列无缺口,同时确认两个线段
                    let start = &self.raw_points[0];
                    let fx1_end =
                        &self.raw_points[self.potencial_fx.as_ref().unwrap().k2.from_index];
                    let fx2_end = &self.raw_points[fx.k2.from_index];
                    let event = SegmentEvent::New2(start.clone(), fx1_end.clone(), fx2_end.clone());
                    println!("{}", event);
                    self.w1.clear();
                    self.w2.clear();
                    self.need_w2 = false;
                    let _ = self.raw_points.drain(..fx.k2.from_index);

                    self.recalc_seq1();
                    return Some(event);
                }
            }
        }

        None
    }

    pub fn process(&mut self) -> Option<SegmentEvent> {
        if self.direction.is_none() {
            self.find_first_segment();
        } else {
            return self.seq_process();
        }
        None
    }

    // 入口
    pub fn on_pen_event(&mut self, pen_event: PenEvent) -> Option<SegmentEvent> {
        match pen_event {
            PenEvent::First(a, b) => {
                self.raw_points.push(a);
                self.raw_points.push(b);
                None
            }

            PenEvent::New(a) => {
                // New事件代表新的一笔产生了，参数a就是旧笔的终点也是新笔开端，但是由于该笔可能延伸，所以先处理原有的笔，然后将新的端点保存
                // 后续在UpdateTo事件里更新最后一个端点，知道新的PenEvent::New事件产生，代表该端点已经完成
                let event = self.process();
                self.raw_points.push(a);
                event
            }

            PenEvent::UpdateTo(a) => {
                self.raw_points.pop();
                self.raw_points.push(a);
                None
            }
        }
    }

    // utils

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

    fn recalc_seq1(&mut self) {
        self.w1.clear();
        for index in 1..self.raw_points.len() - 1 {
            if index % 2 == 0 {
                let from = &self.raw_points[index - 1];
                let to = &self.raw_points[index];
                self.w1.push(Seq::new(index - 1, from, to));
            }
        }
    }

    fn recalc_seq2(&mut self) {
        debug_assert!(self.potencial_fx.is_some());
        self.w2.clear();
        let start_index = self.potencial_fx.as_ref().unwrap().k2.from_index + 1;
        for index in start_index..self.raw_points.len() - 1 {
            let from = &self.raw_points[index - 1];
            let to = &self.raw_points[index];
            let direction = if from.price > to.price {
                Direction::Down
            } else {
                Direction::Up
            };
            if direction == self.direction.unwrap() {
                self.w2.push(Seq::new(index - 1, from, to));
            }
        }
    }
}

impl std::fmt::Display for SegmentDetectorV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dir = match self.direction {
            None => "无",
            Some(Direction::Up) => "上",
            Some(Direction::Down) => "下",
        };
        write!(f, "{} {} {}", dir, self.raw_points.len(), self.need_w2)
    }
}
