use crate::*;

#[derive(Clone, Debug, trans::Trans)]
pub struct Bullet {
    pub weapon_type: WeaponType,
    pub unit_id: i32,
    pub player_id: i32,
    pub position: Vec2F64,
    pub velocity: Vec2F64,
    pub damage: i32,
    pub size: f64,
    pub explosion_params: Option<ExplosionParams>,
}

impl Bullet {
    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            bottom_left: self.position.add_x(-self.size / 2.0).add_y(-self.size / 2.0),
            size: Vec2F64 { x: self.size, y: self.size },
        }
    }

    pub fn explosion_bounding_box(&self) -> BoundingBox {
        match self.explosion_params {
            Some(ExplosionParams { radius, .. }) => {
                BoundingBox {
                    bottom_left: self.position.add_x(-radius).add_y(-radius),
                    size: Vec2F64 { x: radius * 2.0, y: radius * 2.0 },
                }
            }
            _ => self.bounding_box()
        }
    }

    pub fn explosion_damage(&self) -> i32 {
        match self.explosion_params {
            Some(ExplosionParams { damage, .. }) => damage,
            _ => 0
        }
    }
}