use ai_cup_22::debugging::Color;
use crate::{DebugInterface, MyStrategy};
use ai_cup_22::model::*;
use crate::simulation::Simulator;

struct Vec2Order {
    vec: Vec2,
    description: Option<String>,
}

struct ActionOrderOrder {
    action_order: ActionOrder,
    description: Option<String>,
}

#[allow(dead_code)]
#[allow(unused_variables)]
impl MyStrategy {
    pub fn get_velocity(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Vec2 {
        let order = None
            .or_else(|| self.velocity_avoid_projectiles(unit, game, debug_interface))
            .or_else(|| self.velocity_go_to_shield(unit, game, debug_interface))
            .or_else(|| self.velocity_go_to_ammo(unit, game, debug_interface))
            .or_else(|| self.velocity_go_to_center_of_zone(unit, game, debug_interface));

        if let Some(vec_order) = order {
            if let Some(text) = vec_order.description {
                self.place_label(unit.position, format!("vel: {}", text), 0,debug_interface);
            }
            vec_order.vec
        } else {
            Vec2::zero()
        }
    }

    pub fn get_direction(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Vec2 {
        let order = None
            .or_else(|| self.direction_hittable_enemy(unit, game, debug_interface));
        // .or_else(|| self.direction_look_around(unit, game, debug_interface))

        if let Some(vec_order) = order {
            if let Some(text) = vec_order.description {
                self.place_label(unit.position, format!("dir: {}", text), 1, debug_interface);
            }
            vec_order.vec
        } else {
            unit.velocity
        }
    }

    pub fn get_action_order(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrder> {
        let order = None
            .or_else(|| self.action_pick_up_shield(unit, game, debug_interface))
            .or_else(|| self.action_drink_shield(unit, game, debug_interface))
            .or_else(|| self.action_shoot_at_target(unit, game, debug_interface));

        order.map(|action_order_order| {
            if let Some(text) = action_order_order.description {
                self.place_label(unit.position, format!("act: {}", text), 2, debug_interface);
            }
            action_order_order.action_order
        })
    }

    fn direction_hittable_enemy(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        self
            .enemy_units
            .iter()
            .filter(|e| self.unit_is_hittable_by(e, unit, &self.constants, &mut None))
            .filter(|e| e.is_within_fire_range_of(unit, &self.constants))
            .min_by(|e1, e2| {
                let a1 = unit.direction.angle_with(&(e1.position - unit.position));
                let a2 = unit.direction.angle_with(&(e2.position - unit.position));
                a1.total_cmp(&a2)
            })
            .and_then(|e| {
                self.targets.entry(unit.id).or_insert(e.id);
                Some(e)
            })
            .map(|e| Vec2Order { vec: e.position - unit.position, description: Some("turning to enemy".to_string()) })
    }

    fn direction_look_around(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        Some(Vec2Order {
            vec: Vec2 { x: -unit.direction.y, y: unit.direction.x },
            description: Some("looking around".to_string()),
        })
    }

    fn velocity_avoid_projectiles(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        let threatening_projectiles = self.projectiles_aimed_at_target(game, HittableEntity::from(unit));

        if threatening_projectiles.is_empty() {
            return None;
        }

        let original_direction = if unit.velocity.length() > 0.0 { unit.velocity } else { unit.direction };

        let rotation_angle = (0..360)
            .step_by(45)
            .map(|angle_degree| (angle_degree as f64).to_radians())
            .max_by_key(|angle| {
                let unit_order = UnitOrder {
                    target_velocity: Vec2::from_length_and_angle(self.constants.max_unit_forward_speed, original_direction.angle()).rotate(*angle),
                    target_direction: unit.direction,
                    action: None,
                };
                let mut simulator = Simulator::new(game, &self.constants, unit.id, unit_order);
                let result = simulator.simulate_n_ticks(self.constants.ticks_per_second as usize);
                let score = result.score();
                println!("angle {}, score {}", angle.to_degrees(), score);
                score
            });

        rotation_angle.map(|angle| {
            let velocity = Vec2::from_length_and_angle(self.constants.max_unit_forward_speed, original_direction.angle()).rotate(angle);
            Vec2Order {
                vec: velocity,
                description: Some(format!("avoiding damage, going to {}", (unit.position + velocity).to_short_string())),
            }
        })
    }

    fn velocity_correction_avoid_projectiles(&self, proposed_velocity: Vec2, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Vec2 {
        let threatening_projectiles = self.projectiles_aimed_at_target(game, HittableEntity::from(unit));

        if threatening_projectiles.is_empty() {
            proposed_velocity
        } else {
            if let Some(debug) = debug_interface.as_mut() {
                for p in threatening_projectiles.iter() {
                    let final_position = p.position + p.velocity * p.life_time;
                    debug.add_segment(p.position, final_position, 0.1, Color::red().a(0.4))
                }
            }
            Vec2::zero()
        }
    }

    fn projectiles_aimed_at_target<'a>(&self, game: &'a Game, hittable: HittableEntity) -> Vec<&'a Projectile> {
        game.projectiles.iter()
            .filter(|p| {
                let final_position = p.position + p.velocity * p.life_time;
                hittable.intersects_with(&p.position, &final_position)
            })
            // .filter(|p| {
            //     self.constants.obstacles.iter()
            //         .filter(|o| !o.can_shoot_through)
            //         .any(|o| o.intersects_with(&hittable.position, &p.position))
            // })
            .collect()
    }

    fn velocity_go_to_shield(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        if unit.is_action_cooldown() {
            return None;
        }
        if unit.shield_potions >= self.constants.max_shield_potions_in_inventory {
            return None;
        }

        let predicate = |loot: &Loot| {
            matches!(loot.item, Item::ShieldPotions{..})
        };
        self.velocity_go_to_loot(unit, game, &predicate, debug_interface)
    }

    fn velocity_go_to_ammo(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        if unit.is_action_cooldown() {
            return None;
        }
        let weapon_idx = unit.weapon? as usize;
        let weapon = self.constants.weapons.get(weapon_idx)?;

        if *unit.ammo.get(weapon_idx)? >= weapon.max_inventory_ammo {
            return None;
        }

        let predicate = |loot: &Loot| {
            matches!(loot.item, Item::Ammo{weapon_type_index: weapon_idx, ..})
        };
        self.velocity_go_to_loot(unit, game, &predicate, debug_interface)
    }

    fn velocity_go_to_center_of_zone(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        Some(Vec2Order {
            vec: game.zone.next_center - unit.position,
            description: Some("going to center of zone".to_string()),
        })
    }

    fn velocity_go_to_loot(&self, unit: &Unit, game: &Game, predicate: &dyn Fn(&Loot) -> bool, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        if unit.is_action_cooldown() {
            return None;
        }
        let nearest_loot: Option<&Loot> = self
            .seen_loot
            .values()
            .filter(|loot| predicate(*loot))
            .filter(|loot| loot.position.distance_to(&game.zone.current_center) <= game.zone.current_radius * 0.9)
            .min_by_key(|loot| unit.position.distance_to(&loot.position) as i32);

        nearest_loot
            .filter(|loot| loot.position.distance_to(&unit.position) > self.constants.unit_radius)
            .map(|loot| {
                Vec2Order {
                    vec: (loot.position - unit.position).clamp_min(6.0),
                    description: Some("going to loot".to_string()),
                }
            })
    }

    fn action_drink_shield(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrderOrder> {
        if unit.shield >= self.constants.max_shield {
            return None;
        }

        if unit.shield_potions <= 0 {
            return None;
        }

        Some(ActionOrderOrder {
            action_order: ActionOrder::UseShieldPotion {},
            description: Some("drinking shield".to_string()),
        })
    }

    fn action_shoot_at_target(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrderOrder> {
        let enemy_id = self.targets.get(&unit.id)?;
        let enemy = self.units_by_id.get(enemy_id)?;

        Some(ActionOrderOrder {
            action_order: ActionOrder::Aim { shoot: true },
            description: Some(format!("shooting {} at {}", enemy.id, enemy.position.to_short_string())),
        })
    }

    fn action_pick_up_shield(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrderOrder> {
        if unit.shield_potions >= self.constants.max_shield_potions_in_inventory {
            return None;
        }

        let predicate = |loot: &Loot| {
            matches!(loot.item, Item::ShieldPotions{..})
        };
        self.action_pick_up_loot(unit, game, &predicate, debug_interface)
    }

    fn action_pick_up_ammo(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrderOrder> {
        let weapon_idx = unit.weapon? as usize;
        let weapon = self.constants.weapons.get(weapon_idx)?;

        if *unit.ammo.get(weapon_idx)? >= weapon.max_inventory_ammo {
            return None;
        }

        let predicate = |loot: &Loot| {
            matches!(loot.item, Item::Ammo{weapon_type_index: weapon_idx, ..})
        };
        self.action_pick_up_loot(unit, game, &predicate, debug_interface)
    }

    fn action_pick_up_loot(&self, unit: &Unit, game: &Game, predicate: &dyn Fn(&Loot) -> bool, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrderOrder> {
        if unit.is_action_cooldown() {
            return None;
        }
        let order = self
            .seen_loot
            .values()
            .filter(|loot| predicate(*loot))
            .find(|loot| loot.position.distance_to(&unit.position) <= self.constants.unit_radius)
            .map(|loot| ActionOrder::Pickup { loot: loot.id });

        order.map(|action_order| {
            ActionOrderOrder {
                action_order,
                description: Some("picking up".to_string()),
            }
        })
    }

    pub fn unit_is_hittable_by(&self, enemy: &Unit, unit: &Unit, constants: &Constants, debug_interface: &mut Option<&mut DebugInterface>) -> bool {
        let obstacles_in_los = constants
            .obstacles
            .iter()
            .filter(|o| !o.can_shoot_through)
            .filter(|o| o.intersects_with(&enemy.position, &unit.position))
            .collect::<Vec<_>>();

        if obstacles_in_los.len() > 0 {
            if let Some(debug) = debug_interface.as_mut() {
                for o in obstacles_in_los.iter() {
                    debug.add_circle(o.position, o.radius, Color::red())
                }
            }
            false
        } else {
            true
        }
    }
}