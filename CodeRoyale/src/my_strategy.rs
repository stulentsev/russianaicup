use std::collections::HashMap;

use ai_cup_22::model::*;

use crate::debug_interface::DebugInterface;

pub struct MyStrategy {
    pub(crate) constants: Constants,
    pub(crate) units_by_id: HashMap<i32, Unit>,
    pub(crate) my_units: Vec<Unit>,
    pub(crate) enemy_units: Vec<Unit>,
    pub(crate) targets: HashMap<i32, i32>,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        Self {
            constants,
            units_by_id: HashMap::new(),
            my_units: vec![],
            enemy_units: vec![],
            targets: HashMap::new(),
        }
    }
    pub fn get_order(
        &mut self,
        game: &Game,
        mut debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        if let Some(debug) = debug_interface.as_mut() {
            debug.clear();
            debug.set_auto_flush(false);
        }

        let mut orders = HashMap::new();

        self.rebuild_indexes(game);

        for unit in game.units.iter() {
            if unit.player_id != game.my_id {
                continue;
            }

            if unit.action.is_some() {
                continue;
            }

            self.visualize_sounds(unit, game, &mut debug_interface);

            let target_direction: Vec2 = self.get_direction(unit, game, &mut debug_interface);
            let action: Option<ActionOrder> = self.get_action_order(unit, game, &mut debug_interface);
            let target_velocity: Vec2 = self.get_velocity(unit, game, &mut debug_interface);

            // let maybe_order = None
            //     // .or_else(|| self.avoid_projectiles(unit, game, &mut debug_interface))
            //     .or_else(|| self.drink_shield(unit, game, &mut debug_interface))
            //     .or_else(|| self.pick_up_shield(unit, game))
            //     .or_else(|| self.go_to_shield(unit, game, &mut debug_interface))
            //     .or_else(|| self.shoot_at_enemy(unit, game, &mut debug_interface))
            //     .or_else(|| self.go_to_center_of_next_zone(unit, game, &mut debug_interface))
            //     .or_else(|| self.scan_perimeter(unit, game, &mut debug_interface));

            orders.insert(unit.id, UnitOrder {
                target_velocity,
                target_direction,
                action,
            });
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
