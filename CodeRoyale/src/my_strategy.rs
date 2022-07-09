use std::collections::HashMap;

use ai_cup_22::model::*;

use crate::debug_interface::DebugInterface;

pub struct MyStrategy {
    pub(crate) constants: Constants,
    pub(crate) units: Vec<Unit>,
    pub(crate) my_units: Vec<Unit>,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        Self { constants, units: vec![], my_units: vec![] }
    }
    pub fn get_order(
        &mut self,
        game: &Game,
        mut debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        let mut orders = HashMap::new();

        self.units = game.units.clone();
        self.my_units = game.units.iter().filter(|u| u.player_id == game.my_id).cloned().collect();

        for unit in game.units.iter() {
            if unit.player_id != game.my_id {
                continue;
            }

            if unit.action.is_some() {
                continue;
            }

            self.visualize_sounds(unit, game, &mut debug_interface);

            let maybe_order = None
                // .or_else(|| self.avoid_projectiles(unit, game, &mut debug_interface))
                .or_else(|| self.drink_shield(unit, game, &mut debug_interface))
                .or_else(|| self.pick_up_shield(unit, game))
                // .or_else(|| self.go_to_shield(unit, game, &mut debug_interface))
                .or_else(|| self.shoot_at_enemy(unit, game, &mut debug_interface))
                .or_else(|| self.go_to_center_of_next_zone(unit, game, &mut debug_interface))
                .or_else(|| self.scan_perimeter(unit, game, &mut debug_interface));

            if let Some(order) = maybe_order {
                orders.insert(unit.id, order);
            }
        }
        if let Some(debug) = debug_interface {
            debug.flush();
        }
        Order {
            unit_orders: orders,
        }
    }

    pub fn finish(&mut self) {}
}
