use common::{event::SegmentEvent, point::Point, test_util::load_eurusd_2021};
use plot::plot::TVPlot;

fn main() {
    let bars = load_eurusd_2021();
    let mut fd = m0::analyzer::Analyzer::new();
    let mut pd = m1::analyzer::Analyzer::new();
    let mut sd = m2::analyzer::Analyzer::new();
    let mut segs: Vec<Point> = Vec::new();
    let mut pens: Vec<Point> = Vec::new();

    for bar in &bars {
        let f = fd.on_new_bar(bar);
        if let Some(fx) = f {
            let event = pd.on_new_fractal(fx);
            if let Some(pe) = event {
                let seg_event = sd.on_new_pen_event(pe);
                match seg_event {
                    None => {}
                    Some(e) => match e {
                        SegmentEvent::New(p1, p2, v) => {
                            segs.push(p1);
                            segs.push(p2);
                            let mut for_append = v;
                            pens.append(&mut for_append);
                        }
                        SegmentEvent::New2(p1, p2, p3, v1, v2) => {
                            segs.push(p1);
                            segs.push(p2);
                            segs.push(p3);
                            let mut for_append = v1;
                            pens.append(&mut for_append);
                            let mut for_append2 = v2;
                            pens.append(&mut for_append2);
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    TVPlot::new()
        .add_bar_series(&bars)
        .add_line_series(&pens)
        .add_line_series(&segs)
        .display();
}
