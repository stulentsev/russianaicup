use std::f64::consts::PI;
use rand::Rng;
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
    pub fn get_velocity(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Vec2 {
        let order = None
            .or_else(|| self.velocity_avoid_projectiles(unit, game, debug_interface))
            .or_else(|| self.velocity_go_to_weapon(unit, game, debug_interface))
            .or_else(|| self.velocity_go_to_shield(unit, game, debug_interface))
            .or_else(|| self.velocity_go_to_ammo(unit, game, debug_interface))
            .or_else(|| self.velocity_continue_to_waypoint(unit, game, debug_interface))
            .or_else(|| self.velocity_go_to_somewhere_in_the_zone(unit, game, debug_interface));

        if let Some(vec_order) = order {
            if let Some(text) = vec_order.description {
                self.place_label(unit.position, format!("vel: {}", text), 0, debug_interface);
            }
            vec_order.vec
        } else {
            Vec2::zero()
        }
    }

    pub fn get_direction(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Vec2 {
        let order = None
            .or_else(|| self.direction_hittable_enemy(unit, game, debug_interface));
        // .or_else(|| self.direction_look_around(unit, game, debug_interface));

        if let Some(vec_order) = order {
            if let Some(text) = vec_order.description {
                self.place_label(unit.position, format!("dir: {}", text), 1, debug_interface);
            }
            vec_order.vec
        } else {
            unit.velocity
        }
    }

    pub fn get_action_order(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrder> {
        if self.is_action_cooldown(unit) {
            return None;
        }
        let order = None
            .or_else(|| self.action_shoot_at_target(unit, game, debug_interface))
            .or_else(|| self.action_pick_up_weapon(unit, game, debug_interface))
            .or_else(|| self.action_pick_up_shield(unit, game, debug_interface))
            .or_else(|| self.action_drink_shield(unit, game, debug_interface))
            .or_else(|| self.action_pick_up_ammo(unit, game, debug_interface))
            ;

        order.map(|action_order_order| {
            if let Some(text) = action_order_order.description {
                self.place_label(unit.position, format!("act: {}", text), 2, debug_interface);
            }
            action_order_order.action_order
        })
    }

    fn direction_hittable_enemy(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        if unit.weapon.is_none() {
            return None;
        }

        self
            .enemy_units
            .iter()
            .filter(|enemy| enemy.is_within_fire_range_of(unit, &self.constants))
            .min_by(|e1, e2| {
                let a1 = unit.direction.angle_with(&(self.simple_projected_position(e1, unit) - unit.position));
                let a2 = unit.direction.angle_with(&(self.simple_projected_position(e2, unit) - unit.position));
                a1.total_cmp(&a2)
            })
            .and_then(|enemy| {
                self.targets.entry(unit.id).or_insert(enemy.id);
                Some(enemy)
            })
            .map(|enemy| {
                let fire_target = self.simple_projected_position(enemy, unit);

                Vec2Order {
                    vec: fire_target - unit.position,
                    description: Some("turning to enemy".to_string()),
                }
            })
            .filter(|vec_order| {
                self.position_is_hittable_by(&(unit.position + vec_order.vec).into(), unit, &self.constants, debug_interface)
            })
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

        // TODO: simulate complex movements (N velocities, M directions, K ticks), instead of traveling in a straight line
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
                // TODO: prefer positions away from enemy
                // TODO: prefer positions behind an obstacle
                let score = result.score();
                // println!("angle {}, score {}", angle.to_degrees(), score);
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

    fn velocity_steer_around_obstacles(&self, unit: &Unit, vec_order: Vec2Order, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        let delta_time = 1.0 / self.constants.ticks_per_second;
        let mut t = 0;
        let mut max_t = 10;
        let obstacle_in_the_way = loop {
            t += 1;
            if t >= max_t {
                break None;
            }
            let future_position = unit.position + unit.velocity * delta_time * t as f64;
            let obstacle_in_the_way = self.constants.obstacles.iter().find(|o| o.position.distance_to(&future_position) < o.radius + self.constants.unit_radius);
            if obstacle_in_the_way.is_some() {
                break obstacle_in_the_way;
            }
        };

        if obstacle_in_the_way.is_none() {
            return Some(vec_order);
        }

        let obstacle = obstacle_in_the_way.unwrap();

        let velocity = vec_order.vec;
        let turn_indicator = velocity.cross_product(&(obstacle.position - unit.position));
        let leeway = 1.0;
        let opposite = obstacle.radius + self.constants.unit_radius + leeway;
        let hypot = obstacle.position - unit.position;
        let sin_angle = opposite / hypot.length();
        let mut angle = sin_angle.asin();

        if angle.is_nan() { angle = PI / 2.0 };

        // println!("turn {}, opposite {}, hypot {}, angle {}", turn_indicator, opposite, hypot, angle.to_degrees());
        if turn_indicator < 0.0 {
            // turn left
            let corrected_velocity = velocity.rotate(angle).clamp(self.constants.max_unit_forward_speed);
            if let Some(debug) = debug_interface.as_mut() {
                debug.add_segment(unit.position, unit.position + corrected_velocity, 0.1, Color::red());
            }
            Some(Vec2Order { vec: corrected_velocity, description: Some("turning left".to_string()) })
        } else if turn_indicator > 0.0 {
            // turn right
            let corrected_velocity = velocity.rotate(-angle).clamp(self.constants.max_unit_forward_speed);
            if let Some(debug) = debug_interface.as_mut() {
                debug.add_segment(unit.position, unit.position + corrected_velocity, 0.1, Color::red());
            }
            Some(Vec2Order { vec: corrected_velocity, description: Some("turning right".to_string()) })
        } else {
            Some(vec_order)
        }
    }


    fn velocity_correction_avoid_projectiles(&self, proposed_velocity: Vec2, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Vec2 {
        let threatening_projectiles = self.projectiles_aimed_at_target(game, HittableEntity::from(unit));

        if threatening_projectiles.is_empty() {
            proposed_velocity
        } else {
            // if let Some(debug) = debug_interface.as_mut() {
            //     for p in threatening_projectiles.iter() {
            //         let final_position = p.position + p.velocity * p.life_time;
            //         debug.add_segment(p.position, final_position, 0.1, Color::red().a(0.4))
            //     }
            // }
            Vec2::zero()
        }
    }

    fn projectiles_aimed_at_target(&self, game: &Game, hittable: HittableEntity) -> Vec<&Projectile> {
        self.seen_projectiles.values()
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

    fn velocity_go_to_weapon(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        if self.is_action_cooldown(unit) {
            return None;
        }

        if !matches!(unit.priority(), LootPriority::Weapon | LootPriority::Whatever) {
            return None;
        }
        let (bow_idx, _) = self.constants.weapons.iter().enumerate().find(|(_, w)| w.name == "Bow").unwrap();

        match unit.weapon {
            Some(i) if i as usize == bow_idx => {
                return None;
            }
            _ => {
                let predicate = |loot: &Loot| {
                    matches!(loot.item, Item::Weapon{type_index: i} if i as usize == bow_idx)
                };
                self.velocity_go_to_loot(unit, game, &predicate, debug_interface)
            }
        }
    }

    fn velocity_go_to_shield(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        if self.is_action_cooldown(unit) {
            return None;
        }

        if !matches!(unit.priority(), LootPriority::Shield | LootPriority::Whatever) {
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

    fn velocity_go_to_ammo(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        if self.is_action_cooldown(unit) {
            return None;
        }

        if !matches!(unit.priority(), LootPriority::Ammo | LootPriority::Whatever) {
            return None;
        }

        let weapon_idx = unit.weapon? as usize;
        let (bow_idx, _) = self.constants.weapons.iter().enumerate().find(|(_, w)| w.name == "Bow").unwrap();

        let weapon = self.constants.weapons.get(weapon_idx)?;

        if unit.ammo[weapon_idx] >= weapon.max_inventory_ammo {
            return None;
        }

        let predicate = |loot: &Loot| {
            matches!(loot.item, Item::Ammo{weapon_type_index: i, ..} if i as usize == weapon_idx || i as usize == bow_idx)
        };
        self.velocity_go_to_loot(unit, game, &predicate, debug_interface)
    }

    fn velocity_continue_to_waypoint(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        let waypoint = self.waypoints.get(&unit.id)?;

        Some(Vec2Order {
            vec: *waypoint - unit.position,
            description: Some("going to waypoint".to_string()),
        }).and_then(|vec_order| {
            self.velocity_steer_around_obstacles(unit, vec_order, game, debug_interface)
        })
    }

    fn velocity_go_to_somewhere_in_the_zone(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        let center = game.zone.next_center;
        let radius = game.zone.next_radius;

        let mut rng = rand::thread_rng();
        let mut random_point = loop {
            let p = center + Vec2::from_length_and_angle(rng.gen_range(0.0..radius), rng.gen_range(0.0..2.0 * PI));
            if !self.constants.obstacles.iter().any(|o| o.position.distance_to(&p) < (o.radius + self.constants.unit_radius / 2.0) + 0.2) {
                break p;
            }
        };

        self.waypoints.entry(unit.id).and_modify(|w| *w = random_point).or_insert(random_point);

        Some(Vec2Order {
            vec: random_point - unit.position,
            description: Some("going to a random point".to_string()),
        }).and_then(|vec_order| {
            self.velocity_steer_around_obstacles(unit, vec_order, game, debug_interface)
        })
    }

    fn velocity_go_to_loot(&mut self, unit: &Unit, game: &Game, predicate: &dyn Fn(&Loot) -> bool, debug_interface: &mut Option<&mut DebugInterface>) -> Option<Vec2Order> {
        if self.is_action_cooldown(unit) {
            return None;
        }
        let nearest_loot: Option<&Loot> = self
            .seen_loot
            .values()
            .filter(|loot| {
                match self.move_targets.get(&loot.id) {
                    Some(id) if *id == unit.id => true,
                    None => true,
                    _ => false,
                }
            })
            .filter(|loot| predicate(*loot))
            .filter(|loot| loot.position.distance_to(&game.zone.current_center) <= game.zone.current_radius * 0.9)
            .min_by_key(|loot| unit.position.distance_to(&loot.position) as i32)
            .and_then(|loot| {
                self.move_targets.entry(loot.id).or_insert(unit.id);
                Some(loot)
            });

        nearest_loot
            .filter(|loot| loot.position.distance_to(&unit.position) > self.constants.unit_radius)
            .map(|loot| {
                Vec2Order {
                    vec: (loot.position - unit.position).clamp_min(3.0),
                    description: Some("going to loot".to_string()),
                }
            }).and_then(|vec_order| {
            self.velocity_steer_around_obstacles(unit, vec_order, game, debug_interface)
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
        let ammo = unit.ammo[unit.weapon? as usize];

        if ammo == 0 {
            return None;
        }

        Some(ActionOrderOrder {
            action_order: ActionOrder::Aim { shoot: true },
            description: Some(format!("shooting {} at {}", enemy.id, enemy.position.to_short_string())),
        })
            .filter(|vec_order| {
                let fire_target = self.simple_projected_position(enemy, unit);
                self.position_is_hittable_by(&fire_target.into(), unit, &self.constants, debug_interface)
            })
    }

    fn action_pick_up_shield(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrderOrder> {
        if unit.shield_potions >= self.constants.max_shield_potions_in_inventory {
            return None;
        }

        let predicate = |loot: &Loot| {
            matches!(loot.item, Item::ShieldPotions{..})
        };
        self.action_pick_up_loot(unit, game, &predicate, debug_interface)
    }

    fn action_pick_up_ammo(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrderOrder> {
        let weapon_idx = unit.weapon? as usize;
        let (bow_idx, _) = self.constants.weapons.iter().enumerate().find(|(_, w)| w.name == "Bow").unwrap();
        let weapon = self.constants.weapons.get(weapon_idx)?;

        if unit.ammo[weapon_idx] >= weapon.max_inventory_ammo {
            return None;
        }

        let predicate = |loot: &Loot| {
            matches!(loot.item, Item::Ammo{weapon_type_index: i, ..} if i as usize == weapon_idx || i as usize == bow_idx)
        };
        self.action_pick_up_loot(unit, game, &predicate, debug_interface)
    }

    fn action_pick_up_weapon(&mut self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrderOrder> {
        let (bow_idx, _) = self.constants.weapons.iter().enumerate().find(|(_, w)| w.name == "Bow").unwrap();

        match unit.weapon {
            Some(i) if i as usize == bow_idx => {
                return None;
            }
            _ => {
                let predicate = |loot: &Loot| {
                    matches!(loot.item, Item::Weapon{type_index: i} if i as usize == bow_idx)
                };
                self.action_pick_up_loot(unit, game, &predicate, debug_interface)
            }
        }
    }

    fn action_pick_up_loot(&mut self, unit: &Unit, game: &Game, predicate: &dyn Fn(&Loot) -> bool, debug_interface: &mut Option<&mut DebugInterface>) -> Option<ActionOrderOrder> {
        if self.is_action_cooldown(unit) {
            return None;
        }
        let order = self
            .seen_loot
            .values()
            .filter(|loot| predicate(*loot))
            .find(|loot| loot.position.distance_to(&unit.position) <= self.constants.unit_radius)
            .and_then(|loot| {self.move_targets.remove(&loot.id); Some(loot)})
            .map(|loot| ActionOrder::Pickup { loot: loot.id });

        order.map(|action_order| {
            ActionOrderOrder {
                action_order,
                description: Some("picking up".to_string()),
            }
        })
    }

    pub fn position_is_hittable_by(&self, enemy: &HittableEntity, unit: &Unit, constants: &Constants, debug_interface: &mut Option<&mut DebugInterface>) -> bool {
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

    pub fn angle_for_leading_shot(&self, enemy: &Unit, unit: &Unit, debug_interface: &mut Option<&mut DebugInterface>) -> f64 {
        let weapon_idx = unit.weapon.unwrap();
        let weapon = &self.constants.weapons[weapon_idx as usize];
        let projectile_speed = weapon.projectile_speed;
        let enemy_speed = enemy.velocity.length();

        let d = enemy.position - unit.position;
        let angle_b = (enemy.velocity - enemy.position).angle_with(&(d));
        let sin_angle_a = angle_b.sin() * enemy_speed / projectile_speed;
        let angle_a = sin_angle_a.asin(); // abs
        let angle_c = PI - angle_a - angle_b;

        let moving_right = (enemy.position + enemy.velocity).arg() > d.arg();
        let result = if moving_right {
            d.arg() - angle_a
        } else {
            d.arg() + angle_a
        };
        // if let Some(debug) = debug_interface.as_mut() {
        //     debug.add_segment(unit.position, unit.position + Vec2::from_length_and_angle(d.length(), result), 0.15, Color::green().a(0.5));
        // }
        result
    }

    pub fn is_action_cooldown(&self, unit: &Unit) -> bool {
        if let Some(action) = unit.action.as_ref() {
            action.finish_tick > self.current_tick
        } else {
            false
        }
    }

    pub fn go_to_next_point_from_the_hardcoded_list(&self, unit: &Unit) -> (Vec2, Vec2) {
        let random_point = Vec2 {
            x: vec![100.0, -100.0, -100.0, 100.0, -80.0, -80.0, 80.0][(self.current_tick as usize / 30) % 7],
            y: vec![100.0, 100.0, -100.0, -100.0, -80.0, 80.0, 80.0][(self.current_tick as usize / 30) % 7],
        };
        let target_direction = random_point - unit.position;
        let target_velocity = random_point - unit.position;
        (target_direction, target_velocity)
    }

    pub fn clear_waypoint_if_in_storm(&mut self, unit: &Unit, game: &Game) {
        if let Some(wp) = self.waypoints.get(&unit.id) {
            if game.zone.current_center.distance_to(wp) > game.zone.current_radius {
                self.waypoints.remove(&unit.id);
            }
        }
    }

    pub fn clear_waypoint_if_reached(&mut self, unit: &Unit) {
        if let Some(wp) = self.waypoints.get(&unit.id) {
            if unit.position.distance_to(wp) < self.constants.unit_radius / 2.0 {
                self.waypoints.remove(&unit.id);
            }
        }
    }

    pub fn simple_projected_position(&self, enemy: &Unit, unit: &Unit) -> Vec2 {
        let weapon = &self.constants.weapons[unit.weapon.unwrap() as usize];
        let projectile_travel_time = unit.position.distance_to(&enemy.position) / weapon.projectile_speed;
        let remaining_aim_time = weapon.aim_time - weapon.aim_time * unit.aim;
        let fire_target = enemy.position + (enemy.velocity * (projectile_travel_time + remaining_aim_time));
        fire_target
    }
}
