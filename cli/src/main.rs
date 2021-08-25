use common::{event::*, point::Point, test_util::*};
use plot::plot::TVPlot;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "czsc_cli", about = "czsc in action.")]
struct Opt {
    /// Display Chart
    #[structopt(short, long)]
    display: bool,
}

fn main() {
    let opt = Opt::from_args();

    let bars = load_eurusd_2021_05_06();
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
                let pen_event = pe.clone();
                match pen_event {
                    PenEvent::First(a, b) => {
                        pens.push(a);
                        pens.push(b);
                    }

                    PenEvent::New(a) => {
                        pens.push(a);
                    }

                    PenEvent::UpdateTo(a) => {
                        pens.pop();
                        pens.push(a);
                    }
                }

                let seg_event = sd.on_new_pen_event(pe);
                match seg_event {
                    None => {}
                    Some(e) => match e {
                        SegmentEvent::New(p1, p2) => {
                            segs.push(p1);
                            segs.push(p2);
                        }
                        SegmentEvent::New2(p1, p2, p3) => {
                            segs.push(p1);
                            segs.push(p2);
                            segs.push(p3);
                        }
                        SegmentEvent::UpdateTo(p1) => {
                            segs.pop();
                            segs.push(p1);
                        }
                    },
                }
            }
        }
    }

    if opt.display {
        TVPlot::new()
            .add_bar_series(&bars)
            .add_line_series(&pens)
            .add_line_series(&segs)
            .display();
    }
}
