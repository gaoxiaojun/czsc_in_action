//! 笔规则处理

use common::fx::{FxType, Fx};

pub fn detect_is_pen(f1: &Fx, f2: &Fx) -> bool {
    if f1.fx_type == FxType::Top
        && f2.fx_type == FxType::Bottom
        && f1.has_enough_distance(f2)
        && f2.price() < f1.price()
        && !fx_is_contain(f1,f2)
    {
        return true;
    }

    if f1.fx_type == FxType::Bottom
        && f2.fx_type == FxType::Top
        && f1.has_enough_distance(f2)
        && f2.price() > f1.price()
        && !fx_is_contain(f1,f2)
    {
        return true;
    }

    false
}

// 分型包含规则，第二根Candle的最高最低作为分型区间
pub fn fx_is_contain(lhs: &Fx, rhs: &Fx) -> bool {
    if (lhs.fx_type == FxType::Top && lhs.range_low() < rhs.range_low() && lhs.range_high() < rhs.range_high())
        || (lhs.fx_type == FxType::Bottom && lhs.range_high() > rhs.range_high() && lhs.range_low() > rhs.range_low())
    {
        true
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeAction {
    Keep,
    Replace,
}

// 同类分型合并规则
// 考虑一种特殊情况就是顶分型高点相等或者底分型低点相等
// 处理原则：分型极值点相等不算，必须突破才算
// 也就是说，后分型必须突破前分型才能采用后分型
pub fn merge_same_fx_type(prev: &Fx, next: &Fx) -> MergeAction {
    debug_assert!(prev.fx_type == next.fx_type);
    if prev.fx_type == FxType::Top {
        if next.price > prev.price {
            MergeAction::Replace
        } else {
            MergeAction::Keep
        }
    } else {
        if next.price < prev.price {
            MergeAction::Replace
        } else {
            MergeAction::Keep
        }
    }
}
