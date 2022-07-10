use crate::model::*;

#[derive(Clone)]
pub struct SimProjectile {
    pub last_position: Vec2,
    pub last_life_time: f64,

    // fields from Projectile
    pub id: i32,
    pub weapon_type_index: i32,
    pub shooter_id: i32,
    pub shooter_player_id: i32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub life_time: f64,
}

impl From<&Projectile> for SimProjectile {
    fn from(projectile: &Projectile) -> Self {
        Self {
            last_position: projectile.position,
            last_life_time: projectile.life_time,

            id: projectile.id,
            weapon_type_index: projectile.weapon_type_index,
            shooter_id: projectile.shooter_id,
            shooter_player_id: projectile.shooter_player_id,
            position: projectile.position,
            velocity: projectile.velocity,
            life_time: projectile.life_time,
        }
    }
}