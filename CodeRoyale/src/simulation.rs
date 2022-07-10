use std::cmp::max;
use std::f64::consts::FRAC_PI_2;
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
    pub unit: SimUnit,
    unit_order: UnitOrder,
    constants: Constants,
    result: SimulationResult,
}

impl Simulator {
    pub fn new(game: &Game, constants: &Constants, unit_id: i32, unit_order: UnitOrder) -> Self {
        Self {
            game: SimGame::new(game),
            unit: game.units.iter().find(|u| u.id == unit_id).unwrap().into(),
            unit_order,
            constants: constants.clone(),
            result: Default::default(),
        }
    }

    pub fn simulate_n_ticks(&mut self, n: usize) -> SimulationResult {
        for _ in 0..n {
            self.simulate_tick();
        }
        self.result.clone()
    }

    pub fn simulate_tick(&mut self) {
        // self.simulate_rotation();
        // self.simulate_action();
        self.simulate_unit_movement();
        self.simulate_projectile_movement();
        // self.remove_dead_players();
        // self.regen_health();
    }

    // fn simulate_rotation(&mut self) {
    //     for unit in self.game.units.iter_mut() {
    //
    //     }
    // }

    fn simulate_unit_movement(&mut self) {
        let positions = self.game.units.iter().map(|unit| {
            if unit.id == self.unit.id {
                self.simulate_next_position(&self.unit, self.unit_order.target_velocity)
            } else {
                // check collisions
                unit.velocity / self.constants.ticks_per_second
            }
        }).collect_vec();

        for (idx, unit) in self.game.units.iter_mut().enumerate() {
            unit.position = *positions.get(idx).unwrap();
            if self.unit.id == unit.id {
                self.unit.position = unit.position;
            }
        }
    }

    fn simulate_projectile_movement(&mut self) {
        let delta_time = 1.0 / self.constants.ticks_per_second;

        for projectile in self.game.projectiles.iter_mut() {
            if projectile.life_time > 0.0 {
                projectile.position += projectile.velocity * delta_time;
            }
            if self.constants.obstacles.iter().filter(|o| o.can_shoot_through ).any(|o| o.position.distance_to(&projectile.position) < o.radius) {
                projectile.life_time = -1.0;
                continue;
            }

            if let Some(unit) = self.game.units.iter_mut().find(|u| u.position.distance_to(&projectile.position) < self.constants.unit_radius) {
                projectile.life_time = -1.0;
                let weapon = self.constants.weapons.get(projectile.weapon_type_index as usize).unwrap();
                let shield_damage = if unit.shield > weapon.projectile_damage { weapon.projectile_damage } else { unit.shield };
                unit.shield -= shield_damage;
                unit.health -= weapon.projectile_damage - shield_damage;
                if unit.player_id == self.unit.player_id {
                    self.result.damage_received += weapon.projectile_damage;
                }
            }
        }

        self.game.projectiles = self.game.projectiles.iter().filter(|p| p.life_time > 0.0).cloned().collect();
        self.game.units = self.game.units.iter().filter(|u| u.health > 0.0).cloned().collect();
    }

    fn simulate_next_position(&self, unit: &SimUnit, target_velocity: Vec2) -> Vec2 {
        let delta_time = 1.0 / self.constants.ticks_per_second;
        let delta_acceleration = (target_velocity.length() - unit.velocity.length()).clamp(0.0, self.constants.unit_acceleration * delta_time);

        let current_speed = unit.velocity.length();
        let target_speed = self.constants.max_unit_forward_speed;

        let velocity = Vec2::from_length_and_angle(
            (current_speed + delta_acceleration).clamp(0.0, target_speed),
            target_velocity.angle(),
        );
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
        position
    }
}
