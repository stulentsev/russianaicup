use crate::model::{Game, Unit, Projectile};
use super::*;

pub struct SimGame {
    pub units: Vec<SimUnit>,
    pub projectiles: Vec<SimProjectile>,
}

impl SimGame {
    pub fn new(game: &Game) -> Self {
        Self {
            units: game.units.iter().map(|u| u.into()).collect(),
            projectiles: game.projectiles.iter().map(|p| p.into()).collect(),
        }
    }

    pub fn update(&mut self, game: &Game) {

    }
}