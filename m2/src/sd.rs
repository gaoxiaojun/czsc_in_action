use crate::fx::Line;
use crate::sequence::Seq;
use common::direction::{self, Direction};
use common::event::{PenEvent, SegmentEvent};
use common::fx::FxType;
use common::point::Point;
use common::ringbuffer::RingBuffer;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct State {
    potential_point: usize,
    fx_type: FxType,
    window: RingBuffer<Seq>,
}

impl State {
    pub fn new(potential_point: usize, fx_type: FxType) -> Self {
        Self {
            potential_point,
            fx_type,
            window: RingBuffer::new(3),
        }
    }
}

// 合并规则
// 前包含后，合并
// 后包含前，不合并
// 然后找分型

#[derive(Debug)]
pub struct SegmentDetector {
    points: VecDeque<Point>,
    direction: Option<Direction>,
    sequence_1: RingBuffer<Line>,
    sequence_2: RingBuffer<Line>,
    potential_point: Option<(FxType, usize)>,
}

impl SegmentDetector {
    pub fn new() -> Self {
        Self {
            points: VecDeque::new(),
            direction: None,
            sequence_1: RingBuffer::new(3),
            sequence_2: RingBuffer::new(3),
            potential_point: None,
        }
    }

    fn check_potential_point(&self) -> Option<(FxType, usize)> {
        // 至少要6个点才能判断潜在点,且是偶数点
        // 奇数点是与线段方向不符的，不用考虑
        let len = self.points.len();
        if len >= 6 && len % 2 == 0 {
            let p3 = &self.points[len - 1]; // 最后一个点
            let p2 = &self.points[len - 3];
            let p1 = &self.points[len - 5];

            let is_top = p1.price < p2.price && p2.price > p3.price;
            let is_bottom = p1.price > p2.price && p2.price < p3.price;
            if is_top || is_bottom {
                let direction = if is_top { FxType::Top } else { FxType::Bottom };
                return Some((direction, len - 3));
            }
        }

        None
    }

    fn process_normal_segment(&mut self) -> Option<SegmentEvent> {
        // **要先找到潜在点（potential point），原因是潜在点前后的特征序列是不用处理包含关系的**

        let potential_point = self.check_potential_point();

        None
    }

    fn find_first_segment(&mut self) -> Option<SegmentEvent> {
        let p1 = self.get(-4).unwrap();
        let p2 = self.get(-3).unwrap();
        let p3 = self.get(-2).unwrap();
        let p4 = self.get(-1).unwrap();

        self.direction = SegmentDetector::is_first_segment(p1, p2, p3, p4);

        None
    }

    // 线段检测主入口
    pub fn process(&mut self) -> Option<SegmentEvent> {
        // 调用本方法，所以至少需要4个分型端点
        if self.points.len() < 4 {
            return None;
        }

        if self.direction.is_none() {
            self.find_first_segment()
        } else {
            self.process_normal_segment()
        }
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
                self.points.push_back(a);
                let event = self.process();
                event
            }

            PenEvent::UpdateTo(a) => {
                self.points.pop_back();
                self.points.push_back(a);
                None
            }
        }
    }

    // helper
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

    // helper
    fn get(&self, index: isize) -> Option<&Point> {
        if index >= 0 {
            self.points.get(index as usize)
        } else {
            self.points
                .get((self.points.len() as isize + index) as usize)
        }
    }

    /*fn add_point(&mut self, p: Point) -> Option<SegmentEvent> {
        let len = self.points.len();
        match len {
            0|1|2|3 => {
                self.points.push_back(p);
            }
            4 => {
                self.points.push_back(p);
                let p4 = &self.points[len-1];
                let p3 = &self.points[len-2];
                let p2 = &self.points[len-3];
                let p1 = &self.points[len-4];
                self.direction = SegmentDetector::is_first_segment(p1, p2, p3, p4);
                if self.direction.is_none() {
                    self.points.pop_front();
                }
            }
            _ => {}
        }

        None
    }*/
}
