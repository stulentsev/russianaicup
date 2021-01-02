use itertools::Itertools;
use model::*;

pub struct Influence {
    map_size: i32,
    vec_size: i32,
    pub my_influence: Vec<i32>,
    pub enemy_influence: Vec<i32>,
    pub influence_map: Vec<i32>,
    pub tension_map: Vec<i32>,
    pub vulnerability_map: Vec<i32>,
    turret_attack_range: Vec<i32>,
}

impl Influence {
    fn new(map_size: usize) -> Self {
        let vec_size = map_size * map_size;
        Influence {
            map_size: map_size as i32,
            vec_size: vec_size as i32,
            my_influence: Vec::with_capacity(vec_size),
            enemy_influence: Vec::with_capacity(vec_size),
            influence_map: Vec::with_capacity(vec_size),
            tension_map: Vec::with_capacity(vec_size),
            vulnerability_map: Vec::with_capacity(vec_size),
            turret_attack_range: Vec::with_capacity(vec_size),
        }
    }

    pub fn recalculate(&mut self, player_view: &PlayerView) {
        self.my_influence.clear();
        self.enemy_influence.clear();
        self.influence_map.clear();
        self.tension_map.clear();
        self.vulnerability_map.clear();
        self.turret_attack_range.clear();

        for _ in 0..self.vec_size {
            self.my_influence.push(0);
            self.enemy_influence.push(0);
            self.influence_map.push(0);
            self.tension_map.push(0);
            self.vulnerability_map.push(0);
            self.turret_attack_range.push(0);
        }

        self.calculate_my_influence(player_view);
        self.calculate_enemy_influence(player_view);
        self.calculate_turret_attack_range(player_view);
        self.calculate_influence_map();
        self.calculate_tension_map();
        self.calculate_vulnerability_map();
    }

    // return whether any neighbor of this cell has enemy influence
    #[allow(dead_code)]
    pub fn is_adjacent_to_attackable_cell(&self, loc: &Vec2I32) -> bool {
        if self.is_attackable_cell(loc) {
            return true;
        }

        if loc.x > 0 && self.is_attackable_cell(&loc.add_x(-1)) {
            return true;
        }

        if loc.x < 79 && self.is_attackable_cell(&loc.add_x(1)) {
            return true;
        }

        if loc.y > 0 && self.is_attackable_cell(&loc.add_y(-1)) {
            return true;
        }

        if loc.y < 79 && self.is_attackable_cell(&loc.add_y(1)) {
            return true;
        }

        false
    }

    #[allow(dead_code)]
    fn is_attackable_cell(&self, loc: &Vec2I32) -> bool {
        self.enemy_influence_at(loc) > 0
    }

    pub fn my_influence_at(&self, loc: &Vec2I32) -> i32 {
        if loc.x < 0 || loc.x > 79 || loc.y < 0 || loc.y > 79 {
            return 0;
        }
        self.my_influence[vec_to_idx(loc)]
    }

    pub fn enemy_influence_at(&self, loc: &Vec2I32) -> i32 {
        if loc.x < 0 || loc.x > 79 || loc.y < 0 || loc.y > 79 {
            return 0;
        }
        self.enemy_influence[vec_to_idx(loc)]
    }

    #[allow(dead_code)]
    pub fn resulting_influence_at(&self, loc: &Vec2I32) -> i32 {
        if loc.x < 0 || loc.x > 79 || loc.y < 0 || loc.y > 79 {
            return 0;
        }
        self.influence_map[vec_to_idx(loc)]
    }

    pub fn most_threatening_enemy_presence(&self, unit: &Entity) -> Option<Vec2I32> {
        // find min value in influence map, normalized by distance.
        // That is, if roughly the same enemy influence in two points, pick the closest of the two.
        let (idx, _) = self
            .enemy_influence
            .iter()
            .enumerate()
            .filter(|(_, val)| **val > 0)
            .sorted_by_key(|(idx, val)| {
                let loc = Vec2I32::from_flat(*idx as i32);
                (**val as f32 / unit.position.mdist(&loc) as f32) as i32
            })
            .next()?; // .first() is weirdly named in rust
        Some(Vec2I32::from_flat(idx as i32))
    }

    fn calculate_my_influence(&mut self, player_view: &PlayerView) {
        // for each unit, add its attack value to every cell within its attack range
        for unit in player_view
            .entities
            .iter()
            .filter(|e| e.player_id == Some(player_view.my_id))
        {
            for cell in unit.vision_range_cells() {
                if cell.x >= 0 && cell.x < self.map_size && cell.y >= 0 && cell.y < self.map_size {
                    let (added_influence, idx) = self.calculate_added_influence(&unit, cell);
                    self.my_influence[idx as usize] += added_influence;
                }
            }
        }
    }

    fn calculate_enemy_influence(&mut self, player_view: &PlayerView) {
        // for each unit, add its attack value to every cell within its attack range
        for unit in player_view
            .entities
            .iter()
            .filter(|e| e.player_id.is_some() && e.player_id != Some(player_view.my_id))
        {
            let cells = match unit.entity_type {
                EntityType::Turret => vec!(),
                EntityType::BuilderUnit => vec!(),
                _ => unit.vision_range_cells(),
            };
            for cell in cells {
                if cell.x >= 0 && cell.x < self.map_size && cell.y >= 0 && cell.y < self.map_size {
                    let (added_influence, idx) = self.calculate_added_influence(&unit, cell);
                    self.enemy_influence[idx as usize] += added_influence;
                }
            }
        }
    }

    fn calculate_turret_attack_range(&mut self, player_view: &PlayerView) {
        // for each unit, add its attack value to every cell within its attack range
        for unit in player_view
            .entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Turret && e.active && e.player_id != Some(player_view.my_id))
        {
            for cell in unit.attack_range_cells() {
                if cell.x >= 0 && cell.x < self.map_size && cell.y >= 0 && cell.y < self.map_size {
                    let idx = cell.x * self.map_size + cell.y;
                    self.turret_attack_range[idx as usize] = 1;
                }
            }
        }
    }

    pub fn is_turret_attack_at(&self, loc: &Vec2I32) -> bool {
        if loc.x < 0 || loc.x > 79 || loc.y < 0 || loc.y > 79 {
            return false;
        }
        self.turret_attack_range[vec_to_idx(loc)] > 0
    }

    fn calculate_influence_map(&mut self) {
        // for each unit, add its attack value to every cell within its attack range
        for i in 0..self.vec_size as usize {
            let a = self.my_influence[i];
            let b = self.enemy_influence[i];
            self.influence_map[i] = a - b;
        }
    }

    fn calculate_tension_map(&mut self) {
        // for each unit, add its attack value to every cell within its attack range
        for i in 0..self.vec_size as usize {
            let a = self.my_influence[i];
            let b = self.enemy_influence[i];
            self.tension_map[i] = a + b;
        }
    }

    fn calculate_vulnerability_map(&mut self) {
        // for each unit, add its attack value to every cell within its attack range
        for i in 0..self.vec_size as usize {
            let a = self.tension_map[i];
            let b = self.influence_map[i];
            self.vulnerability_map[i] = a - b.abs();
        }
    }

    fn calculate_added_influence(&self, unit: &Entity, cell: Vec2<i32>) -> (i32, i32) {
        let atk = unit.attack_value();
        let rng = unit.attack_range() as i32;

        let dist = cell.mdist(&unit.position);
        let added_influence = if dist <= rng as i32 {
            // max inf within attack range
            atk as f32
        } else {
            // gradual falloff
            atk as f32 * (0.9f32.powi(dist - rng))
        };
        let idx = cell.x * self.map_size + cell.y;
        ((added_influence * unit.health as f32) as i32, idx)
    }
}

impl Default for Influence {
    fn default() -> Self {
        Self::new(80)
    }
}

fn vec_to_idx(vec: &Vec2I32) -> usize {
    vec.x as usize * 80 + vec.y as usize
}
