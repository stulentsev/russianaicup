#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, trans::Trans)]
pub enum MineState {
    Preparing,
    Idle,
    Triggered,
    Exploded,
}
