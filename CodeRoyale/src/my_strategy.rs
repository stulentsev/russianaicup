use std::collections::HashMap;
use ai_cup_22::debugging::Color;

use ai_cup_22::model::*;

use crate::debug_interface::DebugInterface;
use crate::simulation::Simulator;

pub struct MyStrategy {
    pub(crate) constants: Constants,
    pub(crate) units_by_id: HashMap<i32, Unit>,
    pub(crate) my_units: Vec<Unit>,
    pub(crate) enemy_units: Vec<Unit>,
    pub(crate) targets: HashMap<i32, i32>,
    pub(crate) move_targets: HashMap<i32, i32>,
    pub(crate) seen_loot: HashMap<i32, Loot>,
    pub(crate) seen_projectiles: HashMap<i32, Projectile>,
    pub(crate) current_tick: i32,
    pub(crate) next_positions: HashMap<i32, (Vec2, Vec2, Vec2)>,
    pub(crate) waypoints: HashMap<i32, Vec2>,
    pub(crate) next_imaginary_id: i32,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        Self {
            constants,
            units_by_id: HashMap::new(),
            my_units: vec![],
            enemy_units: vec![],
            targets: HashMap::new(),
            move_targets: HashMap::new(),
            seen_loot: HashMap::new(),
            seen_projectiles: HashMap::new(),
            current_tick: 0,
            next_positions: HashMap::new(),
            waypoints: HashMap::new(),
            next_imaginary_id: -1,
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
        // self.process_sounds(&game);
        self.check_expected_position_vs_actual(game, &mut debug_interface);

        self.show_status_labels_for_units(&mut debug_interface);

        for unit in game.units.iter() {
            if unit.player_id != game.my_id {
                continue;
            }

            self.visualize_sounds(unit, game, &mut debug_interface);
            self.visualize_weapon_ranges(game, &mut debug_interface);
            self.visualize_projectiles(game, &mut debug_interface);
            self.visualize_waypoint(unit, &mut debug_interface);
            self.clear_waypoint_if_in_storm(unit, game);
            self.clear_waypoint_if_reached(unit);

            let target_direction: Vec2 = self.get_direction(unit, game, &mut debug_interface);
            let action: Option<ActionOrder> = self.get_action_order(unit, game, &mut debug_interface);
            let target_velocity: Vec2 = self.get_velocity(unit, game, &mut debug_interface);

            // let (target_direction, target_velocity) = self.go_to_next_point_from_the_hardcoded_list(unit);

            // if let Some(debug) = debug_interface.as_mut() {
            //     debug.add_segment(unit.position, unit.position + target_velocity, 0.2, Color::blue());
            // };

            let unit_order = UnitOrder {
                target_velocity,
                target_direction,
                action,
            };

            self.predict_next_positions(game, unit, &unit_order, &mut debug_interface);
            orders.insert(unit.id, unit_order);
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
