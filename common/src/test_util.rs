use chrono::prelude::*;
use crate::bar::Bar;
use csv;

pub fn load_eurusd_2021() -> Vec<Bar> {
    let csv = include_str!("../data/EU_2021.csv");
    load_datetime_bar(&csv)
}

pub fn load_datetime_bar(csv: &str) -> Vec<Bar> {
    let mut bars: Vec<Bar> = Vec::new();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv.as_bytes());
    for record in reader.records() {
        let record = record.unwrap();
        let timestr: &str = AsRef::<str>::as_ref(&record[0]);
        let dt = NaiveDateTime::parse_from_str(timestr, "%Y.%m.%d %H:%M:%S").unwrap();
        let datetime: DateTime<Utc> = DateTime::from_utc(dt, Utc);
        let time = datetime.timestamp_millis();
        let open = AsRef::<str>::as_ref(&record[1]).parse::<f64>().unwrap();
        let high = AsRef::<str>::as_ref(&record[2]).parse::<f64>().unwrap();
        let low = AsRef::<str>::as_ref(&record[3]).parse::<f64>().unwrap();
        let close = AsRef::<str>::as_ref(&record[4]).parse::<f64>().unwrap();
        let _vol = AsRef::<str>::as_ref(&record[5]).parse::<f64>().unwrap();
        let bar = Bar::new(time, open, high, low, close);
        bars.push(bar);
    }
    bars
}
