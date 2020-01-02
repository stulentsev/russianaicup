use crate::{DrawDebug, GameStrategy};
use crate::constants::EPS;

pub struct SequenceReplayStrategy {}

impl SequenceReplayStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl GameStrategy for SequenceReplayStrategy {
    fn get_action(
        &mut self,
        unit: &model::Unit,
        game: &model::Game,
        debug: &mut DrawDebug,
    ) -> model::UnitAction {
        let velocity = 1.0;
        let jump = true;
        let jump_down = false;
        let aim = model::Vec2F64 { x: 0.0, y: 0.0 };
        let shoot = false;
        let reload = false;
        let swap_weapon = false;
        let plant_mine = false;

        model::UnitAction {
            velocity,
            jump,
            jump_down,
            aim,
            shoot,
            reload,
            swap_weapon,
            plant_mine,
        }
    }
}
