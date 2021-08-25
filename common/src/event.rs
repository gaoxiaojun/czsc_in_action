use crate::point::Point;

#[derive(Debug, Clone)]
pub enum PenEvent {
    First(Point, Point),
    New(Point),
    UpdateTo(Point),
}


#[derive(Debug, Clone)]
pub enum SegmentEvent {
    New(Point, Point),
    New2(Point, Point, Point),
    UpdateTo(Point)
}

impl std::fmt::Display for SegmentEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New(p1,p2) => {
                write!(f,"New({},{})", p1,p2)
            }
            Self::New2(p1,p2,p3) => {
                write!(f,"New2({},{},{})",p1,p2,p3)
            }
            Self::UpdateTo(p1) => {
                write!(f,"UpdateTo({})", p1)
            }
        }
    }
}