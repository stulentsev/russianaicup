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
    pub(crate) seen_loot: HashMap<i32, Loot>,
    pub(crate) seen_projectiles: HashMap<i32, Projectile>,
    pub(crate) current_tick: i32,
    next_positions: HashMap<i32, (Vec2, Vec2, Vec2)>,
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
            seen_projectiles: HashMap::new(),
            current_tick: 0,
            next_positions: HashMap::new(),
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

        for unit in self.my_units.iter() {
            if let Some((pos, dir, vel)) = self.next_positions.get(&unit.id) {
                if !pos.approx_equal(unit.position) {
                    println!("tick {}: diff position {}", game.current_tick, (unit.position - *pos).length())
                }

                if (dir.arg() - unit.direction.arg()) > 10f64.powi(-9) {
                    println!(
                        "tick {}: diff direction {}, expected {}, got {}",
                        game.current_tick,
                        (unit.direction.arg() - dir.arg()).to_degrees(),
                        dir.arg().to_degrees(),
                        unit.direction.arg().to_degrees(),
                    );
                    if let Some(debug) = debug_interface.as_mut() {
                      debug.add_segment(unit.position, unit.position + unit.direction, 0.2, Color::green());
                      debug.add_segment(unit.position, unit.position + *dir, 0.2, Color::red());
                    }
                }

                if !vel.approx_equal(unit.velocity) {
                    println!("tick {}: diff velocity {}, expected {}, got {}", game.current_tick, (unit.velocity - *vel).length(), *vel, unit.velocity);
                    if let Some(debug) = debug_interface.as_mut() {
                        debug.add_segment(unit.position, unit.position + unit.velocity, 0.2, Color::green());
                        debug.add_segment(unit.position, unit.position + *vel, 0.2, Color::red());
                        debug.add_segment(unit.position, unit.position + unit.direction, 0.1, Color::blue().a(0.7));
                    }

                }
            }
        }

        if let Some(debug) = debug_interface.as_mut() {
            for unit in self.my_units.iter() {
                let text = match unit.action {
                    Some(Action { action_type: ActionType::Looting, .. }) => Some("looting"),
                    Some(Action { action_type: ActionType::UseShieldPotion, .. }) => Some("drinking shield"),
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

            // let target_direction: Vec2 = self.get_direction(unit, game, &mut debug_interface);
            let action: Option<ActionOrder> = self.get_action_order(unit, game, &mut debug_interface);
            // let target_velocity: Vec2 = self.get_velocity(unit, game, &mut debug_interface);
            let random_point = Vec2 {
                x: vec![100.0, -100.0, -100.0, 100.0, -80.0, -80.0, 80.0][(self.current_tick as usize / 30) % 7],
                y: vec![100.0, 100.0, -100.0, -100.0, -80.0, 80.0, 80.0][(self.current_tick as usize / 30) % 7],
                // x: vec![100.0, -100.0][(self.current_tick as usize / 50) % 2],
                // y: vec![100.0, -100.0][(self.current_tick as usize / 50) % 2],
            };
            let target_direction = random_point - unit.position;
            let target_velocity = random_point - unit.position;

            // if let Some(debug) = debug_interface.as_mut() {
            //     debug.add_segment(unit.position, unit.position + target_velocity, 0.2, Color::blue());
            // };

            let unit_order = UnitOrder {
                target_velocity,
                target_direction,
                action,
            };

            let mut simulation = Simulator::new(game, &self.constants, unit.id, unit_order.clone());
            simulation.simulate_tick();
            let sim_unit = simulation.unit();
            self.next_positions
                .entry(unit.id)
                .and_modify(|(pos, dir, vel)| {
                    *pos = sim_unit.position;
                    *dir = sim_unit.direction;
                    *vel = sim_unit.velocity;
                })
                .or_insert_with(|| (sim_unit.position, sim_unit.direction, sim_unit.velocity));
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
