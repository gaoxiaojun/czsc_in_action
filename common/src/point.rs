use crate::time::Time;

#[derive(Debug, Clone, Copy, PartialEq)]
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


impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.time, self.price)
    }
}