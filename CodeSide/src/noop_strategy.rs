use crate::{DrawDebug, GameStrategy};

pub struct NoopStrategy {}

impl NoopStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl GameStrategy for NoopStrategy {
    fn get_action(
        &mut self,
        _unit: &model::Unit,
        _game: &model::Game,
        _debug: &mut DrawDebug,
    ) -> model::UnitAction {
        let velocity = 0.0;
        let jump = false;
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
