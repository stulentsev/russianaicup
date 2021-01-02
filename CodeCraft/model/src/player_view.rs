use super::*;
#[derive(Clone, Debug, Default, trans::Trans)]
pub struct PlayerView {
    pub my_id: i32,
    pub map_size: i32,
    pub fog_of_war: bool,
    pub entity_properties: std::collections::HashMap<EntityType, EntityProperties>,
    pub max_tick_count: i32,
    pub max_pathfind_nodes: i32,
    pub current_tick: i32,
    pub players: Vec<Player>,
    pub entities: Vec<Entity>,
}

impl PlayerView {
    pub fn me(&self) -> Player {
        self.players.iter().find(|p| p.id == self.my_id).unwrap().clone()
    }
}
