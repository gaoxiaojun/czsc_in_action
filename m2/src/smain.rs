// 总规则：
// 基于特征序列的线段划分标准方法主要步骤就是：
// 先根据线段方向，找出特征序列，然后对特征序列进行标准化，寻找标准特征序列的分型，然后判断分型第1,2元素间是否有缺口，分为第一种破坏和第二种破坏；
// 第一种破坏比较简单直接，当即可断定线段结束；
// 第二种破坏则需要寻找反向特征序列的分型来进行确认，如果没有出现反向特征序列分型就新高新低，则仍为原线段的延续。
// 可以解决90%的线段划分问题

// 特征序列合并方向
// 笔只有两个方向，向上或者向下
// 向下笔之间高高合并 向上笔之间低低合并

// 特征序列包含合并规则
// 前包含：前面的完全把后面的包含在内
// 后包含：后面的完全把前面的包含在内
// 合并的目的是找分型；找分型的目的是确定段的起始点

// 特征序列分型成立条件
// 特征序列分型分为第一元素、第二元素(定义为分型起点)和第三元素
// 顶分型：第二元素高点在相邻三个元素的高点中最高，低点比第三元素的低点高
// 底分型：第二元素低点在相邻三个元素的低点中最低，高点比第三元素的高点低
// 线段的端点必定是分型的起点
// 三个元素的不同合并规则
// 第一元素:只用前包含规则,处理完所有的前包含;如果出现后包含,前者为第一元素,后者为第二元素
// 第二元素:只用前包含规则,处理完所有的前包含;如果出现后包含,前者为第一元素,后者为第二元素
// 第三元素:不处理包含规则(这里是有争议的处理方案,按照本方案,79课的图一和图二的结果是相同)

// 第一特征序列的分型查找,这里要参见补充规则
// 顺着线段当前方向，合并特征序列，这里合并规则只考虑前包含，不考虑后包含
// 向上线段把向下笔作为第一特征序列，向下线段把向上笔作为第一特征序列
// 向上线段找顶分型，向下线段找底分型
// 找到分型后，把顶分型的高点、底分型的低点作为假设点

// 第二特征序列的分型查找
// 逆着线段当前方向，合并特征序列，这里合并规则前包含、后包含都要考虑
// 向上线段在假设点之后，把向上笔作为第二特征序列
// 向下线段在假设点之后，把向下笔作为第二特征序列
// 向上线段在假设点之后找底分型，向下线段在假设点之后找顶分型
// 顶分型的特征序列1，2，3，如果2的高点最高，2的低点也是最高，则顶分型成立
// 底分型的特征序列1，2，3，如果2的低点最低，2的高点也是最低，则底分型成立
// *第二特征序列只有在第一特征序列分型成立且有缺口的情况下才有意义*

// 线段终结
// 第一步:寻找第一特征序列分型
// 第二步:判断是否是线段终结的第一种情况
// 2.1 是第一种情况（无gap），前线段终结,转终结处理（case1）
// 2.2 是第二种情况，开始检测第二特征序列
// 第三步:寻找第二特征分型
// 3.1 找到对应分型,则线段结束
// 3.1.1 第二特征序列的分型无gap，同时结束两个线段，转终结处理（case2）
// 3.1.2 第二特征序列的分型有gap，A段结束，转终结处理（case3）
// 3.2 未找到，第一特征序列出现新分型

// 线段终结后的处理
// 三种情况
// case1: 前线段终结，新线段开始,方向反转, 新线段起点是分型顶点(也是旧线段的终结点),终点是??
// case2: 前两个线段终结，新线段开始，方向不变
// case3: 前线段终结，新线段开始，方向反转
// 三种情况都需要重新计算第一特征序列，转第一步

// 查找第一个线段
// 判断方式通过4个端点的滑动窗口来判断
// 向上线段，1-2-3-4，成立条件，1 < 2 > 3 < 4 同时 1 < 3 && 2 < 4，简化后 1 < 3 && 2 > 3 && 4 > 2
// 向下线段，1-2-3-4，成立条件，1 > 2 < 3 > 4 同时 1 > 3 && 2 > 4, 简化后 1 > 3 && 2 < 3 && 4 < 2
// 简单来说，就是形成N字

// 细节
// 出现第二种情况，如果分型未找到，但是出现了新高新低，放弃第二分型的查找，认为线段延续
// 反向特征序列分型第三笔直接破底前面分型的顶底的情况,看做一段延伸还是3段,都是有道理的,看自己选择哪个标准

// 特殊案例的汇总
// 首先复杂线段都是在中枢震荡中产生的,单边趋势中一般都很简单
// 其次不用太纠结很多特殊案例,本质上都是规则问题,用一种统一的规则就可,1分钟级别的线段有些许不同,不影响具体的操作
// 79课的图一图二,图一三段,图二两段, 个人观点,图二可以作为三段,否则规则就不统一了.
// 77课的80-83,个人观点,也是为了规则统一,80-81,81-82,82-83三段

use crate::sequence::Seq;
use common::point::Point;
use std::collections::VecDeque;

use common::{event::PenEvent, fx::Fx, ringbuffer::RingBuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeDirection {
    Up,
    Down,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationReson {
    // 对应线段破坏的第一种情况
    CASE1,
    // 对应线段破坏的第二种情况且同时确认第二线段也成立，即window2也构成了无gap的分型
    CASE21,
    // 对应线段破坏的第二种情况但是不确认第二线段也成立，即window2也构成了有gap的分型
    CASE22,
}

#[derive(Debug, Clone)]
pub enum SegmentEvent {
    First(Point, Point),
    New(Point, Point),
    New2(Point, Point, Point),
    Update(Point),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentDirection {
    Up,
    Down,
}

pub type FractalVecIndex = usize;

#[derive(Debug)]
pub struct SegmentDetector {
    points: VecDeque<Point>,
    direction: Option<SegmentDirection>,

    // 假设的线段终结点
    current: FractalVecIndex,
    // 对应假设终结点的前高(低)点，用于特征分型第一元素的计算
    prev: FractalVecIndex,

    // 对应线段终结第一种情况，保存3个分型判断即可
    window1: RingBuffer<Seq>,

    // 对应线段终结第二种情况，
    window2: RingBuffer<Seq>,
}

impl SegmentDetector {
    pub fn new() -> Self {
        Self {
            points: VecDeque::new(),
            direction: None,
            current: 0,
            prev: 0,
            window1: RingBuffer::new(3),
            window2: RingBuffer::new(3),
        }
    }

    // 判断第一个线段的时候，条件约束较严格
    fn is_first_segment(p1: &Point, p2: &Point, p3: &Point, p4: &Point) -> Option<SegmentDirection> {
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
                (true, false) => Some(SegmentDirection::Up),
                (false, true) => Some(SegmentDirection::Down),
                (_, _) => None,
            }
        };
        direction
    }

    // 特征序列进行标准化
    // [start, end) end不包含在里面
    fn merge_seq(&self, start: usize, end: usize, dir: MergeDirection) -> Seq {
        let mut from_index = start;
        let from = self.get(from_index as isize).unwrap();
        let to = self.get((from_index + 1) as isize).unwrap();
        let mut seq = Seq::new(
            Point::new(
                //            from_index,
                from.time,
                from.price,
            ),
            //          from_index + 1,
            Point::new(to.time, to.price),
        );
        while from_index + 2 < end {
            from_index += 2;
            let new_from = self.get(from_index as isize).unwrap();
            let new_to = self.get((from_index + 1) as isize).unwrap();
            let new_seq = Seq::new(
                //from_index,
                Point::new(new_from.time, new_from.price),
                //from_index + 1,
                Point::new(new_to.time, new_to.price),
            );
            let is_merged = Seq::merge(&mut seq, &new_seq);
            if !is_merged {
                break;
            }
        }
        seq
    }

    fn reset_state(&mut self, start_point: usize, prev: usize, current: usize) {
        debug_assert!(current - prev >= 2);
        debug_assert!(start_point < self.points.len());
        self.points.drain(..start_point);
        self.current = current;
        self.prev = prev;
        self.window1.clear();
        self.window2.clear();
        let seq = self.merge_seq(prev, current, self.merge_direction());
        self.window1.push(seq);
    }

    fn find_first_segment(&mut self) -> Option<SegmentEvent> {
        // 查找第一个线段
        // 判断方式通过4个分型的滑动窗口来判断
        // 这里没有包含全部的情况，例如1-2-3-4-5-6等多个笔组成线段，
        // TODO: 按照缠论前3笔重叠构成线段，因此如果前三笔没有构成4点高于2点是不是也算线段？
        // 如果算，这里的第一笔检测算法就要更新，
        debug_assert!(self.direction.is_none());
        let p1 = self.get(-4).unwrap();
        let p2 = self.get(-3).unwrap();
        let p3 = self.get(-2).unwrap();
        let p4 = self.get(-1).unwrap();

        self.direction = SegmentDetector::is_first_segment(p1, p2, p3, p4);

        if self.direction.is_some() {
            let len = self.points.len();
            self.reset_state(self.points.len() - 4, len - 3, len - 1);
            let start = self.get(-4).unwrap().clone();
            let end = self.get(-1).unwrap().clone();
            Some(SegmentEvent::New(start, end))
        } else {
            //self.points.pop_front();
            None
        }
    }

    fn add_seq_on_window1(&mut self, dir: MergeDirection) {
        debug_assert!(self.window1.len() > 0);
        let length = self.window1.len();
        let last = self.get(-1).unwrap();
        let prev = self.get(-2).unwrap();
        let seq = Seq::new(
            //length - 2,
            Point::new(prev.time, prev.price),
            //length - 1,
            Point::new(last.time, last.price),
        );
        let length = self.window1.len();
        if length > 1 {
            let s = self.window1.get_mut(-1).unwrap();
            let is_merged = Seq::merge(s, &seq);
            if !is_merged {
                self.window1.push(seq);
            }
        }
    }

    fn add_seg_on_window2(&mut self, dir: MergeDirection) {
        let length = self.window1.len();
        let last = self.get(-1).unwrap();
        let prev = self.get(-2).unwrap();
        let seq = Seq::new(
            //length - 2,
            Point::new(prev.time, prev.price),
            //length - 1,
            Point::new(last.time, last.price),
        );
        let length = self.window2.len();
        if length > 0 {
            let s = self.window2.get_mut(-1).unwrap();
            let is_merged = Seq::merge(s, &seq);
            if !is_merged {
                self.window2.push(seq);
            }
        } else {
            self.window2.push(seq);
        }
    }

    fn on_new_pen(&mut self) -> Option<SegmentEvent> {
        // 每当新的一笔确认，在假设点前后，填充情况一及情况二的序列(window1, window2)
        //debug_assert!(self.window1.len() == 1);
        debug_assert!(self.points.len() > self.current);
        debug_assert!(self.direction.is_some());

        // 具体过程如下：
        // 与线段当前方向相反的笔合并处理后放入window1
        // 与线段当前方向相同的笔合并处理后放入window2
        // 当window1的数量达到3，看是否是case1，如果是case1，形成顶分型，线段结束
        // 如果不是，。。。。
        // 当window2的数量达到3，如果是底分型，线段1结束，
        let segment_dir = self.direction.unwrap();
        let last = self.get(-1).unwrap();
        let prev = self.get(-2).unwrap();
        let is_same_direction_up = segment_dir == SegmentDirection::Up && last.price > prev.price;
        let is_same_direction_down =
            segment_dir == SegmentDirection::Down && last.price < prev.price;
        let is_same_direction = is_same_direction_up || is_same_direction_down;
        if is_same_direction {
            self.add_seg_on_window2(SegmentDetector::get_flip_merge_direction(segment_dir));
        } else {
            self.add_seq_on_window1(SegmentDetector::get_merge_direction(segment_dir));
        }

        // TODO:这里存在一个错误，就是何时需要判断是否终结
        // 当window1/window2的第4个分型要压入的时候
        // 参考fractal_dector的实现
        /*let reason = self.check_termination();
        self.flip(reason)
        */
        None
    }

    fn process_normal_segment(&mut self) -> Option<SegmentEvent> {
        // 开始常规线段处理
        debug_assert!(self.direction.is_some());
        let direction = self.direction.unwrap();
        let last_point = self.get(-1).unwrap();

        let new_higher =
            direction == SegmentDirection::Up && last_point.price > self.points[self.current].price;

        let new_lower = direction == SegmentDirection::Down
            && last_point.price < self.points[self.current].price;

        let new_higher_or_lower = new_higher || new_lower;

        if new_higher_or_lower {
            // 创新高或者新低，假设该点是线段终结点
            let new_assume_end_point = self.points.len() - 1;
            self.reset_state(0, self.current, new_assume_end_point);
            None
        } else {
            self.on_new_pen()
        }
    }

    pub fn process(&mut self) -> Option<SegmentEvent> {
        // 调用本方法，所以至少需要4个分型端点
        if self.points.len() < 5 {
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
                // PenEvent::New代表原有笔已经终结,但是该新笔后续还可能延伸
                // 线段检测算法只关注已经完成的笔
                // BUG: New(a) a 是终点也是起点
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

    // helper
    fn get(&self, index: isize) -> Option<&Point> {
        if index >= 0 {
            self.points.get(index as usize)
        } else {
            self.points
                .get((self.points.len() as isize + index) as usize)
        }
    }

    fn merge_direction(&self) -> MergeDirection {
        debug_assert!(self.direction.is_some());
        let direction = self.direction.unwrap();
        SegmentDetector::get_merge_direction(direction)
    }

    fn get_merge_direction(direction: SegmentDirection) -> MergeDirection {
        match direction {
            SegmentDirection::Down => MergeDirection::Down,
            SegmentDirection::Up => MergeDirection::Up,
        }
    }

    fn get_flip_merge_direction(direction: SegmentDirection) -> MergeDirection {
        match direction {
            SegmentDirection::Down => MergeDirection::Up,
            SegmentDirection::Up => MergeDirection::Down,
        }
    }
}
