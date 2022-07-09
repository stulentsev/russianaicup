use ai_cup_22::model::*;
use crate::MyStrategy;

impl MyStrategy {
    pub(crate) fn rebuild_indexes(&mut self, game: &Game) {
        self.units_by_id.clear();
        self.my_units.clear();
        self.enemy_units.clear();
        self.targets.clear();

        self.units_by_id = game.units.iter().map(|u| (u.id, u.clone())).collect();
        self.my_units = game.units.iter().filter(|u| u.player_id == game.my_id).cloned().collect();
        self.enemy_units = game.units.iter().filter(|u| u.player_id != game.my_id).cloned().collect();
    }
}