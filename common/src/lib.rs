//! Common模块提供通用的数据类型
//!
//! 

pub mod bar;
pub mod fx;
pub mod time;
pub mod point;
//pub mod segment;
pub mod ringbuffer;
pub mod test_util;
pub mod event;
pub mod direction;

#[macro_export]
macro_rules! print_flush {
    ( $($t:tt)* ) => {
        {
            use std::io::Write;
            let mut h = std::io::stdout();
            write!(h, $($t)* ).unwrap();
            h.flush().unwrap();
        }
    }
}
