/*
新的顶分型t出现时：
if 前点g不存在 then
    return g(t.x, t.y, t.type)
else if 前点g是顶分型 and t.y>g.y then
    replace(g, t.x, t.y) # 这里的含义是用t的属性覆盖g的，但g的唯一标识不变，以此来标识原g的延伸
    return g(x, y, type)
else if 前点g是底分型 and t.y>g.y then
    return g(t.x, t.y, t.type)

新的底分型t出现时：
if 前点g不存在 then
    return g(t.x, t.y, t.type)
else if 前点是底分型 and t.y<g.y then
    replace(g, t.x, t.y) # 这里的含义是用t的属性覆盖g的，但g的唯一标识不变，以此来标识原g的延伸
    return g(x, y, type)
else if 前点是顶分型 and t.y<g.y then
    return g(t.x, t.y, t.type)

*/

use common::{event::SegmentEvent, fx::FxType};
use m2::seq_fx::SeqFx;

#[derive(Debug)]
pub struct SegmentDetector {
    pub fxs: Vec<SeqFx>,
}

impl SegmentDetector {
    pub fn new() -> Self {
        Self { fxs: Vec::new() }
    }

    pub fn on_new_seq_fx(&mut self, fx: SeqFx) -> Option<SegmentEvent> {
        if fx.fractal_type() == FxType::Top {
            if self.fxs.is_empty() {
                self.fxs.push(fx);
            } else {
                let last = self.fxs.get(self.fxs.len() - 1).unwrap();

                if last.fractal_type() == FxType::Top {
                    if fx.price() > last.price() {
                        self.fxs.pop();
                        self.fxs.push(fx);
                    }
                } else {
                    if fx.price() > last.price() {
                        self.fxs.push(fx);
                    }
                }
            }
        } else {
            if self.fxs.is_empty() {
                self.fxs.push(fx);
            } else {
                let last = self.fxs.get(self.fxs.len() - 1).unwrap();

                if last.fractal_type() == FxType::Bottom {
                    if fx.price() < last.price() {
                        self.fxs.pop();
                        self.fxs.push(fx);
                    }
                } else {
                    if fx.price() < last.price() {
                        self.fxs.push(fx);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::point::Point;
    use common::test_util::*;
    use m2::analyzer;
    use plot::plot::TVPlot;

    #[test]
    fn test_seq_fx_detector() {
        let bars = load_eurusd_2021();
        let mut fd = m0::analyzer::Analyzer::new();
        let mut pd = m1::analyzer::Analyzer::new();
        let mut sfxd = m2::analyzer::Analyzer::new();
        let mut sd = SegmentDetector::new();
        let mut count = 0;
        for bar in &bars {
            let f = fd.on_new_bar(bar);
            if let Some(fx) = f {
                let event = pd.on_new_fractal(fx);
                if let Some(pe) = event {
                    let sfx = sfxd.on_new_pen_event(&pe);
                    if sfx.is_some() {
                        sd.on_new_seq_fx(sfx.unwrap());
                        count += 1;
                    }
                }
            }
        }
        println!("Sfx Count {} ", count);
        println!("KeepFx count {}", sd.fxs.len());
        let mut segs: Vec<Point> = Vec::new();
        for f in &sd.fxs {
            let p = f.s2.from;
            segs.push(p);
        }

        TVPlot::new()
            .add_bar_series(&bars)
            .add_line_series(&sfxd.get_pens())
            .add_line_series(&segs)
            .display();
    }
}
