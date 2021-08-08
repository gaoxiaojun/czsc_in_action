use crate::point::Point;

#[derive(Debug, Clone)]
pub enum PenEvent {
    First(Point, Point),
    New(Point),
    UpdateTo(Point),
}


#[derive(Debug,Clone)]
pub enum SegmentEvent {
    New(Point, Point, Vec<Point>),
    New2(Point, Point, Point, Vec<Point>, Vec<Point>),
    UpdateTo(Point)
}