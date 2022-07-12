use std::collections::HashMap;
use itertools::Itertools;
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

        self.current_tick = game.current_tick;

        self.update_loot(game);
    }

    fn update_loot(&mut self, game: &Game) {
        let visibility_sectors = self.my_units.iter().map(|unit| self.unit_visibility_sector(unit)).collect_vec();
        let loot_by_id = game.loot.iter().map(|loot| (loot.id, loot)).collect::<HashMap<_, _>>();
        // prune items no longer there
        let loot_ids_to_prune = self.seen_loot.values()
            .filter(|loot| { // only currently visible loot
                let visible_at_the_moment = visibility_sectors.iter().any(|sec| sec.cover_point(loot.position));
                let too_old = game.current_tick - loot.seen_on_tick > 50;
                too_old || visible_at_the_moment && !loot_by_id.contains_key(&loot.id)
            })
            .map(|loot| loot.id)
            .collect_vec();

        for id in loot_ids_to_prune.iter() {
            self.seen_loot.remove(id);
        }

        for maybe_new_loot in game.loot.iter() {
            self.seen_loot.entry(maybe_new_loot.id)
                .and_modify(|loot| loot.seen_on_tick = game.current_tick)
                .or_insert_with(|| {
                    let mut new_loot = maybe_new_loot.clone();
                    new_loot.seen_on_tick = game.current_tick;
                    new_loot
                });
        }
    }
}