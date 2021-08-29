use common::{
    direction::Direction,
    event::{PenEvent, SegmentEvent},
    point::Point,
};

use crate::{fx_detector::FxWindow, seq::Seq};

// 本实现有两个不同，
// 1.使用先标准化线段，然后标准化特征序列
// 2.有缺口的请下，迟迟不能确认第二分型，但是已经封闭了缺口

#[derive(Debug)]
pub enum State {
    Case0,
    Case1,
    Case21,
    Case22,
}

#[derive(Debug)]
pub struct SegmentDetectorV3 {
    pub raw_points: Vec<Point>,
    pub direction: Option<Direction>,
    pub w1: FxWindow,
    pub state: State,
}

impl SegmentDetectorV3 {
    pub fn new() -> Self {
        Self {
            raw_points: Vec::new(),
            direction: None,
            w1: FxWindow::new(false),
            state: State::Case0,
        }
    }

    fn process_case1(&mut self) {}

    fn process_case21(&mut self) {}

    fn process_case22(&mut self) {}

    fn find_first_segment(&mut self) {
        let len = self.raw_points.len();

        if len >= 4 {
            let p1 = self.raw_points[len - 4];
            let p2 = self.raw_points[len - 3];
            let p3 = self.raw_points[len - 2];
            let p4 = self.raw_points[len - 1];

            let direction = Self::is_first_segment(&p1, &p2, &p3, &p4);
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

    fn process(&mut self) -> Option<SegmentEvent> {
        match self.state {
            State::Case0 => self.find_first_segment(),
            State::Case1 => self.process_case1(),
            State::Case21 => self.process_case21(),
            State::Case22 => self.process_case22(),
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
}
