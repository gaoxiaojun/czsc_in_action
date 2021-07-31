use crate::point::Point;

#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub from: Point,
    pub to: Point,
}


impl Segment {
    pub fn new(from: Point, to: Point) -> Self {
        Self {
            from,to
        }
    }
}