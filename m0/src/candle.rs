use common::bar::Bar;
use common::time::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub struct Candle {
    // index的作用是为了计算Candle之间的距离，严格笔要求分型之间有5根K，通过index2 - index1就很容易检测是否满足条件，而无需保存整个Candle序列
    // 检测到分型的时候，分型的index就是分型中间Candle的index
    pub index: u64,
    pub time: Time,
    pub high:f64,
    pub low:f64
}

impl Candle {
    #[allow(dead_code)]
    pub(crate) fn new(index: u64, time: Time, high: f64, low: f64) -> Self {
        Self {
            index,time,high,low
            
        }
    }

    pub(crate) fn from_bar(index: u64, bar: &Bar) -> Self {
        Self {
            index,
            time:bar.time, 
            high:bar.high,
            low:bar.low
        }
    }

    // 检测包含方向
    pub fn check_direction(k1: &Candle, k2: &Candle) -> Direction {
        debug_assert!(k1.index != k2.index);
        if k1.high + k1.low > k2.high + k2.low {
            Direction::Down
        } else {
            Direction::Up
        }
    }

    // 检测并处理包含关系
    // 返回值: true:存在包含关系， false:没有包含关系
    // TODO: 特殊的一字板与前一根K高低点相同情况的处理
    pub fn merge(direction: Direction, current: &mut Candle, bar: &Bar) -> bool {
        // current,bar是否有包含关系
        if (current.high >= bar.high && current.low <= bar.low)
            || (current.high <= bar.high && current.low >= bar.low)
        {
            match direction {
                Direction::Down => {
                    // 下包含，取低低
                    if current.low > bar.low {
                        current.time = bar.time;
                    }
                    current.high = f64::min(bar.high, current.high);
                    current.low = f64::min(bar.low, current.low);
                }

                Direction::Up => {
                    // 上包含，取高高
                    if current.high < bar.high {
                        current.time = bar.time;
                    }
                    current.high = f64::max(bar.high, current.high);
                    current.low = f64::max(bar.low, current.low);
                }
            }
            true
        } else {
            false
        }
    }
}

