use std::cmp::max;
use std::f64::consts::{FRAC_PI_2, PI};
use itertools::Itertools;
use ai_cup_22::model::*;
use crate::simulatable_model::*;

#[derive(Default, Clone)]
pub struct SimulationResult {
    pub damage_received: f64,
}

impl SimulationResult {
    pub fn score(&self) -> i32 {
        -self.damage_received as i32
    }
}

pub struct Simulator {
    game: SimGame,
    unit_id: i32,
    unit_order: UnitOrder,
    constants: Constants,
    result: SimulationResult,
}

impl Simulator {
    pub fn new(game: &Game, constants: &Constants, unit_id: i32, unit_order: UnitOrder) -> Self {
        Self {
            game: SimGame::new(game),
            unit_id,
            unit_order,
            constants: constants.clone(),
            result: Default::default(),
        }
    }

    pub fn unit(&self) -> SimUnit {
        self.game.units.iter().find(|u| u.id == self.unit_id).unwrap().clone()
    }
    pub fn simulate_n_ticks(&mut self, n: usize) -> SimulationResult {
        for _ in 0..n {
            self.simulate_tick();
        }
        self.result.clone()
    }

    pub fn simulate_tick(&mut self) {
        self.simulate_rotation();
        // self.simulate_action();
        self.simulate_movement();
        self.simulate_projectile_movement();
        // self.remove_dead_players();
        // self.regen_health();
    }

    // fn simulate_rotation(&mut self) {
    //     for unit in self.game.units.iter_mut() {
    //
    //     }
    // }

    fn simulate_rotation(&mut self) {
        let directions = self.game.units.iter().map(|unit| {
            let direction = if unit.id == self.unit_id {
                self.unit_order.target_direction
            } else {
                unit.direction
            };
            self.simulate_next_direction(&unit, direction)
        }).collect_vec();

        for (idx, unit) in self.game.units.iter_mut().enumerate() {
            unit.direction = *directions.get(idx).unwrap();
        }
    }

    fn simulate_movement(&mut self) {
        let positions = self.game.units.iter().map(|unit| {
            let velocity = if unit.id == self.unit_id {
                self.unit_order.target_velocity
            } else {
                unit.velocity
            };
            self.simulate_next_position(&unit, velocity)
        }).collect_vec();

        for (idx, unit) in self.game.units.iter_mut().enumerate() {
            let (v, p) = *positions.get(idx).unwrap();
            unit.position = p;
            unit.velocity = v;
        }
    }

    fn simulate_projectile_movement(&mut self) {
        let delta_time = 1.0 / self.constants.ticks_per_second;

        for projectile in self.game.projectiles.iter_mut() {
            if projectile.life_time > 0.0 {
                projectile.position += projectile.velocity * delta_time;
            }
            if self.constants.obstacles.iter().filter(|o| o.can_shoot_through).any(|o| o.position.distance_to(&projectile.position) < o.radius) {
                projectile.life_time = -1.0;
                continue;
            }

            if let Some(unit) = self.game.units.iter_mut().find(|u| u.position.distance_to(&projectile.position) < self.constants.unit_radius) {
                projectile.life_time = -1.0;
                let weapon = self.constants.weapons.get(projectile.weapon_type_index as usize).unwrap();
                let shield_damage = if unit.shield > weapon.projectile_damage { weapon.projectile_damage } else { unit.shield };
                unit.shield -= shield_damage;
                unit.health -= weapon.projectile_damage - shield_damage;
                if unit.player_id == self.game.my_id {
                    self.result.damage_received += weapon.projectile_damage;
                }
            }
        }

        self.game.projectiles = self.game.projectiles.iter().filter(|p| p.life_time > 0.0).cloned().collect();
        self.game.units = self.game.units.iter().filter(|u| u.health > 0.0).cloned().collect();
    }

    fn simulate_next_direction(&self, unit: &SimUnit, target_direction: Vec2) -> Vec2 {
        if target_direction.length() < self.constants.unit_radius / 2.0 {
            return unit.direction;
        }

        let delta_time = 1.0 / self.constants.ticks_per_second;
        let a1 = target_direction.arg();
        let a2 = unit.direction.arg();
        let delta_angle = if (a1 - a2).abs() < PI { a1 - a2 } else { a2 - a1 };
        let rotation_speed = self.constants.rotation_speed.to_radians();
        let aim_rotation_speed = if let Some(weapon_idx) = unit.weapon {
            self.constants.weapons[weapon_idx as usize].aim_rotation_speed.to_radians()
        } else {
            rotation_speed
        };
        let rotation_cap = (rotation_speed - (rotation_speed - aim_rotation_speed) * unit.aim) * delta_time;
        let turn_this_tick = delta_angle.clamp(-rotation_cap, rotation_cap);

        unit.direction.rotate(turn_this_tick)
    }

    fn simulate_next_position(&self, unit: &SimUnit, mut target_velocity: Vec2) -> (Vec2, Vec2) {
        let delta_time = 1.0 / self.constants.ticks_per_second;

        let current_speed = unit.velocity.length();
        let target_speed = self.max_speed_for_unit(unit, target_velocity);
        target_velocity = target_velocity.clamp(target_speed);

        let delta_velocity = (target_velocity - unit.velocity).clamp(self.constants.unit_acceleration * delta_time);

        let velocity = unit.velocity + delta_velocity;
        let mut position = unit.position + velocity * delta_time;
        let collision = self.constants.obstacles.iter().find(|o| {
            o.position.distance_to(&position) <= o.radius + self.constants.unit_radius
        });

        if let Some(obs) = collision {
            let pushback_length = obs.radius + self.constants.unit_radius - obs.position.distance_to(&position);
            let normal = obs.position - position;

            let wanted_to_move_by = velocity.length() * delta_time;
            let pushback_vec = Vec2::from_length_and_angle(pushback_length, normal.angle());
            position -= pushback_vec;
            let moved_by_so_far = position.distance_to(&unit.position);
            let movement_left = wanted_to_move_by - moved_by_so_far;

            let angle = normal.angle_with(&target_velocity);
            let velocity_correction = Vec2::from_length_and_angle(velocity.length() * angle.cos(), normal.angle());

            let tangential_velocity = Vec2::from_length_and_angle(movement_left * angle.sin(), (velocity - velocity_correction).angle());
            position += tangential_velocity;
        }
        (velocity, position)
    }

    fn max_speed_for_unit(&self, unit: &SimUnit, target_velocity: Vec2) -> f64 {
        if unit.remaining_spawn_time.is_some() {
            return self.constants.spawn_movement_speed;
        } else {
            let aim_movement_speed_modifier = if let Some(weapon_idx) = unit.weapon {
                self.constants.weapons[weapon_idx as usize].aim_movement_speed_modifier
            } else {
                1.0
            };
            let aim = unit.aim;

            let max_unit_forward_speed = self.constants.max_unit_forward_speed * (1.0 - (1.0 - aim_movement_speed_modifier) * aim);
            let max_unit_backward_speed = self.constants.max_unit_backward_speed * (1.0 - (1.0 - aim_movement_speed_modifier) * aim);

            let d = (max_unit_forward_speed - max_unit_backward_speed) / 2.0;
            let r = (max_unit_forward_speed + max_unit_backward_speed) / 2.0;

            let orig_v = target_velocity;
            let offset = unit.direction;
            // println!("tick {} targ_v angle {}, dir angle {}", self.game.current_tick, orig_v, offset);

            let angle_a = (offset.arg() - orig_v.arg()).abs();
            if f64_approx_eq(angle_a, 0.0) {
                return max_unit_forward_speed
            } else if f64_approx_eq(angle_a,PI) {
                return max_unit_backward_speed
            }

            // println!("angle_a {}", angle_a.to_degrees());
            if angle_a.is_nan() {
                println!("offset: {}, orig_v: {}", offset, orig_v);
            }

            let sin_b = d * angle_a.sin() / r;
            let angle_b = sin_b.asin();
            // println!("angle_b {}", angle_b.to_degrees() );

            let angle_c = PI - angle_a - angle_b;
            // println!("angle_c {}", angle_c.to_degrees());

            let v_len = r * angle_c.sin() / angle_a.sin();
            // println!("v_len {}", v_len);

            v_len
        }
    }

}
pub fn f64_approx_eq(left: f64, right: f64) -> bool {
    let factor = 10f64.powi(7);
    (left * factor).trunc() == (right * factor).trunc()
}

