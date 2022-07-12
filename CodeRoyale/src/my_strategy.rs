use std::collections::HashMap;
use ai_cup_22::debugging::Color;

use ai_cup_22::model::*;

use crate::debug_interface::DebugInterface;

pub struct MyStrategy {
    pub(crate) constants: Constants,
    pub(crate) units_by_id: HashMap<i32, Unit>,
    pub(crate) my_units: Vec<Unit>,
    pub(crate) enemy_units: Vec<Unit>,
    pub(crate) targets: HashMap<i32, i32>,
    pub(crate) seen_loot: HashMap<i32, Loot>,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        Self {
            constants,
            units_by_id: HashMap::new(),
            my_units: vec![],
            enemy_units: vec![],
            targets: HashMap::new(),
            seen_loot: HashMap::new(),
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

        if let Some(debug) = debug_interface.as_mut() {
            for unit in self.my_units.iter() {
                let text = match unit.action {
                    Some(Action{action_type: ActionType::Looting, ..}) => Some("looting"),
                    Some(Action{action_type: ActionType::UseShieldPotion, ..}) => Some("drinking shield"),
                    _ => None,
                };
                if let Some(t) = text {
                    self.place_label(unit.position, t.to_string(), 3, &mut debug_interface);
                }
            }
        }

        for unit in game.units.iter() {
            if unit.player_id != game.my_id {
                continue;
            }

            self.visualize_sounds(unit, game, &mut debug_interface);
            self.visualize_projectiles(game, &mut debug_interface);

            let target_direction: Vec2 = self.get_direction(unit, game, &mut debug_interface);
            let action: Option<ActionOrder> = self.get_action_order(unit, game, &mut debug_interface);
            let target_velocity: Vec2 = self.get_velocity(unit, game, &mut debug_interface);

            if let Some(debug) = debug_interface.as_mut() {
                debug.add_segment(unit.position, unit.position + target_velocity, 0.2, Color::blue());
            };
            orders.insert(unit.id, UnitOrder {
                target_velocity,
                target_direction,
                action,
            });
        }
        if let Some(debug) = debug_interface.as_mut() {
            debug.flush();
        }
        Order {
            unit_orders: orders,
        }
    }

    pub fn finish(&mut self) {}
}
