use crate::time::Time;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub time: Time,
    pub price: f64
}

impl Point {
    pub fn new(time: Time, price: f64) -> Self {
        Self {
            time,price
        }
    }
}
