use super::*;
use EntityType::*;
use utils::clamp;

#[derive(Clone, Copy, Debug, trans::Trans)]
pub struct Entity {
    pub id: i32,
    pub player_id: Option<i32>,
    pub entity_type: EntityType,
    pub position: Vec2I32,
    pub health: i32,
    pub active: bool,
}

impl Entity {
    pub fn center_pos(&self) -> Vec2I32 {
        // hardcode
        self.position.add_xy(self.size() / 2)
    }

    pub fn size(&self) -> i32 {
        match self.entity_type {
            MeleeBase | RangedBase | BuilderBase => 5,
            House => 3,
            Turret => 2,
            Wall => 1,
            _ => 1,
        }
    }

    pub fn point_within_bounds(&self, point: &Vec2I32) -> bool {
        !(point.x < self.position.x ||
            point.y < self.position.y ||
            point.x >= self.position.add_xy(self.size()).x ||
            point.y >= self.position.add_xy(self.size()).y
        )
    }

    pub fn center_pos_f32(&self) -> Vec2F32 {
        // hardcode
        let size = match self.entity_type {
            MeleeBase | RangedBase | BuilderBase => 5,
            House => 3,
            Turret => 2,
            Wall => 1,
            _ => 1,
        };
        Vec2F32::from(self.position).add_xy(size as f32 / 2.0)
    }

    pub fn is_within_attack_range(&self, enemy_unit_pos: &Vec2I32, range: i32) -> bool {
        let size = self.size();
        if size == 1 {
            return self.position.mdist(enemy_unit_pos) <= range;
        } else {
            // scan horizontal borders
            for i in self.position.x..self.position.x + size {
                for &j in &[self.position.y, self.position.y + size - 1] {
                    if Vec2I32::from_i32(i, j).mdist(enemy_unit_pos) <= range {
                        return true;
                    }
                }
            }

            // scan vertical borders
            for &i in &[self.position.x, self.position.x + size - 1] {
                for j in self.position.y..self.position.y + size {
                    if Vec2I32::from_i32(i, j).mdist(enemy_unit_pos) <= range {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn move_range_cells(&self) -> Vec<Vec2I32> {
        self.range_cells(1)
    }

    pub fn move_range_cells_and_self(&self) -> Vec<Vec2I32> {
        let mut result = self.range_cells(1);
        result.push(self.position);
        result
    }

    pub fn vision_range_cells(&self) -> Vec<Vec2I32> {
        self.range_cells(10)
    }

    pub fn attack_range_cells(&self) -> Vec<Vec2I32> {
        self.range_cells(self.attack_range())
    }

    pub fn range_cells(&self, range_u: usize) -> Vec<Vec2I32> {
        let size = self.size();

        let ex = self.position.x;
        let ey = self.position.y;
        let range = range_u as i32;

        let mut result = Vec::with_capacity(range_u.pow(2));

        for i in clamp(ex - range, 0, 79)..clamp(ex + size + range, 0, 79) {
            let x2 = if i < ex {
                ex
            } else if i < ex + size {
                i
            } else {
                ex + size - 1
            };

            for j in clamp(ey - range, 0, 79)..clamp(ey + size + range, 0, 79) {
                let y2 = if j < ey {
                    ey
                } else if j < ey + size {
                    j
                } else {
                    ey + size - 1
                };

                let dist = (i - x2).abs() + (j - y2).abs();

                if dist <= range {
                    result.push(Vec2I32::from_i32(i, j));
                }
            }
        }

        result
    }

    pub fn attack_value(&self) -> i32 {
        match self.entity_type {
            Turret | RangedUnit | MeleeUnit => 5,
            BuilderUnit => 1,
            _ => 0,
        }
    }

    pub fn attack_range(&self) -> usize {
        match self.entity_type {
            Turret | RangedUnit => 5,
            BuilderUnit | MeleeUnit => 1,
            _ => 0,
        }
    }

    pub fn number_of_repairers(&self) -> usize {
        // hardcode
        match self.entity_type {
            MeleeBase | RangedBase | BuilderBase => 10,
            House => 3,
            Turret => 3,
            Wall => 1,
            _ => 1,
        }
    }

    pub fn repair_call_radius(&self) -> i32 {
        // hardcode
        match self.entity_type {
            MeleeBase | RangedBase | BuilderBase => 15,
            House => 7,
            Turret => 7,
            Wall => 1,
            _ => 0,
        }
    }
}
