use crate::fx::Fx;

#[derive(Debug, Clone)]
pub enum PenEvent {
    First(Fx, Fx),
    New(Fx),
    UpdateTo(Fx),
}
