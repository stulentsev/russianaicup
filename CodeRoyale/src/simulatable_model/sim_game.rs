use crate::model::*;
use super::*;

pub struct SimGame {
    pub my_id: i32,
    pub current_tick: i32,
    pub units: Vec<SimUnit>,
    pub projectiles: Vec<SimProjectile>,
    pub zone: Zone,
}

impl SimGame {
    pub fn new(game: &Game) -> Self {
        Self {
            my_id: game.my_id,
            current_tick: game.current_tick,
            units: game.units.iter().map(|u| u.into()).collect(),
            projectiles: game.projectiles.iter().map(|p| p.into()).collect(),
            zone: game.zone.clone(),
        }
    }

    pub fn update(&mut self, game: &Game) {

    }
}