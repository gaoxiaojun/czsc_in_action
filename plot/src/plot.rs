use crate::util::*;
use common::bar::Bar;
use common::point::Point;
use std::vec::Vec;

#[derive(Debug)]
pub struct TVPlot {
    bars: Vec<Bar>,
    lines: Vec<Vec<Point>>,
    builded: bool,
}

impl TVPlot {
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            lines: Vec::new(),
            builded: false,
        }
    }

    pub fn add_bar_series<'a>(&'a mut self, bars: &Vec<Bar>) -> &'a mut TVPlot {
        self.bars = bars.clone();
        self
    }

    pub fn add_line_series<'a>(&'a mut self, lines: &Vec<Point>) -> &'a mut TVPlot {
        self.lines.push(lines.clone());
        self
    }

    pub fn display(&mut self) {
        let blank: Vec<Point> = Vec::new();
        let bars = &self.bars;
        let pens = self.lines.get(0).unwrap_or(&blank);
        let segs = self.lines.get(1).unwrap_or(&blank);
        let _ = draw_bar_tradingview(bars, pens, segs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::test_util::*;
    #[test]
    fn test_display() {
        let bars = load_eurusd_2021();
        TVPlot::new().add_bar_series(&bars).display();
    }
}
