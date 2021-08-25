use std::fmt::Display;
use chrono::{Utc,TimeZone};
// 时间为EPOCH(1970/1/1)开始的毫秒数
//pub type Time = i64;

pub const SECOND_UNIT: i64 = 1000; // 每秒 = 1000毫秒
pub const MINUTE_UNIT: i64 = SECOND_UNIT * 60;

#[derive(Debug,Clone,Copy, PartialEq,Eq)]
pub struct Time {
    time:i64
}

impl Time {
    pub fn new(time: i64) -> Self { Self { time } }

    pub fn as_i64(&self) -> i64 { self.time }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let datetime = Utc.timestamp_millis(self.time);
        datetime.format("%Y-%m-%d %H:%M").fmt(f)
    }
}
