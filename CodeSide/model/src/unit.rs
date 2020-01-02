use crate::*;
#[derive(Copy, Clone, Debug, trans::Trans)]
pub struct Unit {
    pub player_id: i32,
    pub id: i32,
    pub health: i32,
    pub position: Vec2F64,
    pub size: Vec2F64,
    pub jump_state: JumpState,
    pub walked_right: bool,
    pub stand: bool,
    pub on_ground: bool,
    pub on_ladder: bool,
    pub mines: i32,
    pub weapon: Option<Weapon>,
}

impl Unit {
    pub fn center(&self) -> Vec2F64 {
        self.position.clone() + Vec2F64{x: 0.0, y: 1.0}
    }

    pub fn has_weapon(&self) -> bool {
        self.weapon.is_some()
    }

    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            bottom_left: self.position.add_x(-self.size.x / 2.0),
            size: self.size.clone(),
        }
    }

    pub fn need_healing(&self, game: &Game) -> bool {
        return (self.health as f32 / game.properties.unit_max_health as f32) <= 0.8;
    }
}
