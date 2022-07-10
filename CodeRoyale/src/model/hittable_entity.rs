use std::f64::consts::FRAC_PI_2;
use crate::model::*;

pub struct HittableEntity {
    pub position: Vec2,
    pub radius: f64,
}

impl HittableEntity {
    pub fn intersects_with(&self, p0: &Vec2, p1: &Vec2) -> bool {
        let p = p0.sub(p1);
        let c = self.position.sub(p1);
        let angle = p.angle_with(&c);
        let perp_radius = c.length() * angle.sin();
        perp_radius <= self.radius &&
            c.length() < p.length() &&
            angle < FRAC_PI_2
    }

    pub fn from_position_and_radius(position: Vec2, radius: f64) -> Self {
        Self {
            position,
            radius,
        }
    }
}

impl From<&Unit> for HittableEntity {
    fn from(unit: &Unit) -> Self {
        Self {
            position: unit.position,
            radius: 1.0,
        }
    }
}

impl From<&Obstacle> for HittableEntity {
    fn from(obstacle: &Obstacle) -> Self {
        Self {
            position: obstacle.position,
            radius: obstacle.radius,
        }
    }
}
