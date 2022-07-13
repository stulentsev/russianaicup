use crate::model::{Game, Unit, Projectile};
use super::*;

pub struct SimGame {
    pub my_id: i32,
    pub current_tick: i32,
    pub units: Vec<SimUnit>,
    pub projectiles: Vec<SimProjectile>,
}

impl SimGame {
    pub fn new(game: &Game) -> Self {
        Self {
            my_id: game.my_id,
            current_tick: game.current_tick,
            units: game.units.iter().map(|u| u.into()).collect(),
            projectiles: game.projectiles.iter().map(|p| p.into()).collect(),
        }
    }

    pub fn update(&mut self, game: &Game) {

    }
}