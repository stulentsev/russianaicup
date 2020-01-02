use crate::*;
#[derive(Clone, Debug, trans::Trans)]
pub struct Mine {
    pub player_id: i32,
    pub position: Vec2F64,
    pub size: Vec2F64,
    pub state: MineState,
    pub timer: Option<f64>,
    pub trigger_radius: f64,
    pub explosion_params: ExplosionParams,
}

impl Mine {
    pub fn explosion_bounding_box(&self) -> BoundingBox {
        let radius = self.explosion_params.radius;
        BoundingBox {
            bottom_left: self.position.add_x(-radius).add_y(-radius + self.size.y / 2.0),
            size: Vec2F64 { x: radius * 2.0, y: radius * 2.0 },
        }
    }

}