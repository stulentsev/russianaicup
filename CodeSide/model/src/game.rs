use crate::*;

#[derive(Clone, Debug, trans::Trans)]
pub struct Game {
    pub current_tick: i32,
    pub properties: Properties,
    pub level: Level,
    pub players: Vec<Player>,
    pub units: Vec<Unit>,
    pub bullets: Vec<Bullet>,
    pub mines: Vec<Mine>,
    pub loot_boxes: Vec<LootBox>,
}

impl Game {
    pub fn has_unpicked_mines(&self) -> bool {
        self.loot_boxes.iter().any(|lb| {
            match lb.item {
                Item::Mine { .. } => true,
                _ => false,
            }
        })
    }

    pub fn has_unpicked_hp(&self) -> bool {
        self.loot_boxes.iter().any(|lb| {
            match lb.item {
                Item::HealthPack { .. } => true,
                _ => false,
            }
        })
    }

}