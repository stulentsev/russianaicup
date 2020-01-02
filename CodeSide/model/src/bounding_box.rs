use crate::*;

#[derive(Clone, Debug, trans::Trans)]
pub struct BoundingBox {
    pub bottom_left: Vec2F64,
    pub size: Vec2F64,
}

impl BoundingBox {
    pub fn top_right(&self) -> Vec2F64 {
        Vec2F64 {
            x: self.bottom_left.x + self.size.x,
            y: self.bottom_left.y + self.size.y,
        }
    }

    pub fn intersects(&self, another: &BoundingBox) -> bool {
        let me_tr = self.top_right();
        let an_tr = another.top_right();

        // If one rectangle is on left side of other
        if me_tr.x < another.bottom_left.x || an_tr.x < self.bottom_left.x {
            return false;
        }

        // if one rectangle is above the other
        if self.bottom_left.y > an_tr.y || another.bottom_left.y > me_tr.y {
            return false;
        }

        return true;
    }

    pub fn has_point(&self, x: f64, y: f64) -> bool {
        let bl = &self.bottom_left;
        let tr = self.top_right();

        x >= bl.x && x <= tr.x &&
            y >= bl.y && y <= tr.y
    }
}