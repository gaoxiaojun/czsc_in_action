#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let disp = match self {
            &Direction::Up => "上",
            &Direction::Down => "下"
        };
        write!(f, "{}", disp)
    }
}