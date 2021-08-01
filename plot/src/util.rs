use common::bar::Bar;
use common::point::Point;
use rand::{distributions::Alphanumeric, Rng};
use std::error::Error;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::vec::Vec;
use std::{env, fs::File};

const DEFAULT_HTML_APP_NOT_FOUND: &str = "Could not find default application for HTML files.";

const TV_TEMPLATE: &[u8] = include_bytes!("./template.html");

pub fn rand_name() -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    s
}

fn render_bar_tradingview(bar: &Bar) -> String {
    format!(
        "{{ time:{}, open:{}, high:{}, low:{}, close:{} }}",
        bar.time / 1000,
        bar.open,
        bar.high,
        bar.low,
        bar.close
    )
}

fn render_point_tradingview(point: &Point) -> String {
    format!("{{ time:{}, value: {} }}", point.time / 1000, point.price)
}

fn render_bars_tradingview(bars: &Vec<Bar>, pens: &Vec<Point>, segments: &Vec<Point>) -> String {
    let mut buf = String::new();
    let header = "Data ={ \n";
    let bar_header = "Bar : [\n";
    let bar_bottom = "],\n";
    let line_header = "Pen : [\n";
    let line_bottom = "],\n";
    let segment_header = "Segment : [\n";
    let segment_bottom = "]\n";
    let bottom = "}";
    buf.push_str(header);

    // bar data
    buf.push_str(bar_header);
    let bdata: Vec<String> = bars
        .into_iter()
        .map(|bar| render_bar_tradingview(bar))
        .collect();
    let bar_data = bdata.join(",\n");
    buf.push_str(bar_data.as_str());
    buf.push_str(bar_bottom);

    // line data
    buf.push_str(line_header);
    let fdata: Vec<String> = pens
        .into_iter()
        .map(|p| render_point_tradingview(p))
        .collect();
    let line_data = fdata.join(",\n");
    buf.push_str(line_data.as_str());
    buf.push_str(line_bottom);

    // segment data
    buf.push_str(segment_header);
    let sdata: Vec<String> = segments
        .into_iter()
        .map(|p| render_point_tradingview(p))
        .collect();
    let segment_data = sdata.join(",\n");
    buf.push_str(segment_data.as_str());
    buf.push_str(segment_bottom);
    //
    buf.push_str(bottom);
    buf
}

fn read_template(prefix: String) -> String {
    let mut contents = String::from_utf8_lossy(TV_TEMPLATE).to_string();
    let offset = contents.find("</script>").unwrap();
    contents.insert_str(
        offset + 9,
        &format!(r#"<script src="./{}-chart-data.json"></script>"#, prefix),
    );
    let title_offset = contents.find("</title>").unwrap();
    contents.insert_str(title_offset, &prefix);
    contents
}

pub fn draw_bar_tradingview(
    bars: &Vec<Bar>,
    pens: &Vec<Point>,
    segments: &Vec<Point>,
) -> Result<(), Box<dyn Error>> {
    let prefix = rand_name();
    let rendered = render_bars_tradingview(bars, pens, segments);
    let rendered = rendered.as_bytes();
    let mut temp = env::temp_dir();

    // write data.json
    temp.push(format!("{}-chart-data.json", prefix));
    let temp_path = temp.to_str().unwrap();
    {
        let mut file = File::create(temp_path).unwrap();
        file.write_all(rendered)
            .expect("failed to write html output");
        file.flush().unwrap();
    }
    temp.pop();

    // copy index.html
    temp.push(format!("index_{}.html", prefix));
    let content = read_template(prefix);
    let temp_path = temp.to_str().unwrap();
    {
        let mut file = File::create(temp_path).unwrap();
        file.write_all(content.as_bytes())
            .expect("failed to write html output");
        file.flush().unwrap();
    }
    // display in browser
    show_with_default_app(temp.to_str().unwrap());
    //temp.pop();
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn show_with_default_app(temp_path: &str) {
    Command::new("xdg-open")
        .args(&[temp_path])
        .output()
        .expect(DEFAULT_HTML_APP_NOT_FOUND);
}

#[cfg(target_os = "macos")]
pub fn show_with_default_app(temp_path: &str) {
    Command::new("open")
        .args(&[temp_path])
        .output()
        .expect(DEFAULT_HTML_APP_NOT_FOUND);
}

#[cfg(target_os = "windows")]
pub fn show_with_default_app(temp_path: &str) {
    Command::new("cmd")
        .arg("/C")
        .arg(format!(r#"start {}"#, temp_path))
        .output()
        .expect(DEFAULT_HTML_APP_NOT_FOUND);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rand_name() {
        for _i in 0..10 {
            println!("{}", rand_name());
        }
    }
}
