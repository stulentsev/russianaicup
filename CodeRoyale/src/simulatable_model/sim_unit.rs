use crate::model::*;

#[derive(Clone)]
pub struct SimUnit {
    pub last_position: Vec2,
    pub last_velocity: Vec2,
    pub last_aim: f64,
    
    // unit fields
    pub id: i32,
    pub player_id: i32,
    pub health: f64,
    pub shield: f64,
    pub extra_lives: i32,
    pub position: Vec2,
    pub remaining_spawn_time: Option<f64>,
    pub velocity: Vec2,
    pub direction: Vec2,
    pub aim: f64,
    pub action: Option<Action>,
    pub health_regeneration_start_tick: i32,
    pub weapon: Option<i32>,
    pub next_shot_tick: i32,
    pub ammo: Vec<i32>,
    pub shield_potions: i32,
}

impl From<&Unit> for SimUnit {
    fn from(unit: &Unit) -> Self {
        Self {
            last_position: unit.position,
            last_velocity: unit.velocity,
            last_aim: unit.aim,

            id: unit.id,
            player_id: unit.player_id,
            health: unit.health,
            shield: unit.shield,
            extra_lives: unit.extra_lives,
            position: unit.position,
            remaining_spawn_time: unit.remaining_spawn_time,
            velocity: unit.velocity,
            direction: unit.direction,
            aim: unit.aim,
            action: unit.action.clone(),
            health_regeneration_start_tick: unit.health_regeneration_start_tick,
            weapon: unit.weapon.clone(),
            next_shot_tick: unit.next_shot_tick,
            ammo: unit.ammo.clone(),
            shield_potions: unit.shield_potions,
        }
    }
}