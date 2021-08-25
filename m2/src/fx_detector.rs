use colored::Colorize;
use common::{fx::FxType, ringbuffer::RingBuffer};

use crate::seq::{Relationship, Seq};

// 特征序列分型
#[derive(Debug, Clone)]
pub struct SeqFx {
    pub fx_type: FxType,
    pub k1: Seq,
    pub k2: Seq,
    pub k3: Seq,
}

impl SeqFx {
    pub fn new(fx_type: FxType, k1: Seq, k2: Seq, k3: Seq) -> Self {
        Self {
            fx_type,
            k1,
            k2,
            k3,
        }
    }
}

// 特征序列检测
#[derive(Debug)]
pub struct FxWindow {
    pub window: RingBuffer<Seq>,
    pub full_merge: bool, // 是第一特征序列窗口吗
}

impl FxWindow {
    pub fn new(full_merge: bool) -> Self {
        Self {
            window: RingBuffer::new(3),
            full_merge,
        }
    }

    pub fn check_fx(&self) -> Option<SeqFx> {
        //println!("check_fx {}", self.window.len());
        if self.window.len() > 2 {
            let s1 = self.window.get(-3).unwrap();
            let s2 = self.window.get(-2).unwrap();
            let s3 = self.window.get(-1).unwrap();

            //println!("{:?} -- {:?} -- {:?}", s1, s2, s3);

            let is_top = FxWindow::is_top_fractal(&s1, &s2, &s3);
            let is_bottom = FxWindow::is_bottom_fractal(&s1, &s2, &s3);

            if is_top {
                let fx = SeqFx::new(FxType::Top, s1.clone(), s2.clone(), s2.clone());
                return Some(fx);
            }
            if is_bottom {
                let fx = SeqFx::new(FxType::Bottom, s1.clone(), s2.clone(), s2.clone());
                return Some(fx);
            }
        }
        None
    }

    fn is_top_fractal(s1: &Seq, s2: &Seq, s3: &Seq) -> bool {
        if s1.high() <= s2.high() && s2.high() >= s3.high() {
            true
        } else {
            false
        }
    }

    fn is_bottom_fractal(s1: &Seq, s2: &Seq, s3: &Seq) -> bool {
        if s1.low() >= s2.low() && s2.low() <= s3.low() {
            true
        } else {
            false
        }
    }

    pub fn push(&mut self, seq: Seq) -> Option<SeqFx> {
        if self.window.len() <= 1 {
            self.window.push(seq);
            None
        } else {
            let last = self.window.get_mut(-1).unwrap();

            let relationship = last.check_include_relationship(&seq);

            match relationship {
                Relationship::PrevIncludeNext => {
                    last.merge(&seq, relationship);
                }
                Relationship::NextIncludePrev => {
                    if self.full_merge {//|| split_merge {
                        last.merge(&seq, relationship);
                    } else {
                        let k1 = last.clone();
                        self.window.clear();
                        self.window.push(k1);
                        self.window.push(seq);
                        return None;
                    }
                }
                Relationship::FullEquivalent => {
                    println!("{}", "FULL_EQUIVALENT".red());
                }
                _ => {
                    self.window.push(seq);
                    return self.check_fx();
                }
            }
            None
        }
    }

    pub fn clear(&mut self) {
        self.window.clear();
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }
}

impl std::ops::Index<usize> for FxWindow {
    type Output = Seq;
    fn index(&self, index: usize) -> &Self::Output {
        self.window.get(index as isize).unwrap()
    }
}
