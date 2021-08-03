use common::fx::Fx;
use common::time::Time;
#[derive(Debug, Clone, Copy)]
pub struct Point {
    time: Time,
    price: f64,
}

impl Point {
    fn new(time: Time, price: f64) -> Self {
        Self { time, price }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    from: Point,
    to: Point,
    extreme_point: Option<Point>,
    merged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeDirection {
    Up,
    Down,
}

impl Line {
    pub fn new(from_time: Time, from_price: f64, to_time: Time, to_price: f64) -> Self {
        Self {
            from: Point::new(from_time, from_price),
            to: Point::new(to_time, to_price),
            extreme_point: None,
            merged: false,
        }
    }
    pub fn new_from_pen(from: &Fx, to: &Fx) -> Self {
        Self {
            from: Point::new(from.time(), from.price()),
            to: Point::new(to.time(), to.price()),
            extreme_point: None,
            merged: false,
        }
    }

    pub fn high(&self) -> f64 {
        if self.from.price > self.to.price {
            self.from.price
        } else {
            self.to.price
        }
    }

    pub fn low(&self) -> f64 {
        if self.from.price < self.to.price {
            self.from.price
        } else {
            self.to.price
        }
    }

    pub fn is_top_fractal(d1: &Line, d2: &Line, d3: &Line) -> bool {
        if d1.high() < d2.high() && d2.high() > d3.high() {
            true
        } else {
            false
        }
    }

    pub fn is_bottom_fractal(s1: &Line, s2: &Line, s3: &Line) -> bool {
        if s1.low() > s2.low() && s2.low() > s3.low() {
            true
        } else {
            false
        }
    }

    pub fn merge(&mut self, rhs: &Line, dir: MergeDirection) -> bool {
        let is_contain_1 = self.high() < rhs.high() && self.low() > rhs.low();
        let is_contain_2 = self.high() > rhs.high() && self.low() < rhs.low();
        let is_contain = is_contain_1 || is_contain_2;

        if !is_contain {
            return false;
        }

        match dir {
            MergeDirection::Up => self.merge_up(rhs),
            MergeDirection::Down => self.merge_down(rhs),
        }

        true
    }

    pub fn merge_up(&mut self, rhs: &Line) {
        let lhs_height = self.to.price - self.from.price;
        let rhs_height = self.to.price - self.from.price;
        let is_same =
            (lhs_height < 0.0 && rhs_height < 0.0) || (lhs_height > 0.0 && rhs_height > 0.0);

        let is_large = (lhs_height.abs() - rhs_height.abs()) > 0.0;

        match (is_large, is_same) {
            (false, true) => {
                self.from.time = self.to.time;
                self.from.price = self.to.price;
                self.to.time = rhs.from.time;
                self.to.price = rhs.from.price;
            }
            (false, false) => {
                self.to.time = rhs.from.time;
                self.to.price = rhs.from.price;
            }
            (true, true) => {
                self.to.time = rhs.to.time;
                self.to.price = rhs.to.price;
            }
            (true, false) => {
                self.from.time = self.to.time;
                self.from.price = self.to.price;
                self.to.time = rhs.to.time;
                self.to.price = rhs.to.price;
            }
        }
    }

    pub fn merge_down(&mut self, rhs: &Line) {
        let lhs_height = self.to.price - self.from.price;
        let rhs_height = self.to.price - self.from.price;
        let is_same =
            (lhs_height < 0.0 && rhs_height < 0.0) || (lhs_height > 0.0 && rhs_height > 0.0);

        let is_large = (lhs_height.abs() - rhs_height.abs()) > 0.0;

        match (is_large, is_same) {
            (false, true) => {
                self.from.time = self.to.time;
                self.from.price = self.to.price;
                self.to.time = rhs.from.time;
                self.to.price = rhs.from.price;
            }
            (false, false) => {
                self.to.time = rhs.from.time;
                self.to.price = rhs.from.price;
            }
            (true, true) => {
                self.to.time = rhs.to.time;
                self.to.price = rhs.to.price;
            }
            (true, false) => {
                self.from.time = self.to.time;
                self.from.price = self.to.price;
                self.to.time = rhs.to.time;
                self.to.price = rhs.to.price;
            }
        }
    }
}
