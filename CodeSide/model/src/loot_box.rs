use crate::*;
#[derive(Clone, Debug, trans::Trans)]
pub struct LootBox {
    pub position: Vec2F64,
    pub size: Vec2F64,
    pub item: Item,
}

impl LootBox {
    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            bottom_left: self.position.add_x(-self.size.x / 2.0),
            size: self.size.clone(),
        }
    }
}