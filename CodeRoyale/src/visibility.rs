use ai_cup_22::debugging::Color;
use crate::{DebugInterface, MyStrategy};
use ai_cup_22::model::*;

pub struct VisibilitySector {
    pub position: Vec2,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
}

impl VisibilitySector {
    pub fn cover_point(&self, point: Vec2) -> bool {
        let unit_radius = 1.0;
        let distance_to_point = point - self.position;
        let angle = distance_to_point.angle();

        let dist = distance_to_point.length();
        let under_the_unit = dist <= unit_radius;
        let in_the_sector = dist < self.radius &&
            angle > self.start_angle &&
            angle < self.start_angle;

        under_the_unit || in_the_sector
    }
}

impl MyStrategy {

    pub fn unit_visibility_sector(&self, unit: &Unit) -> VisibilitySector {
        let view_distance = self.constants.view_distance;
        let field_of_view = self.constants.field_of_view;
        let aim_field_of_view = if let Some(weapon_idx) = unit.weapon {
            self.constants.weapons.get(weapon_idx as usize).unwrap().aim_field_of_view
        } else {
            0.0
        };

        let fov = (field_of_view - (field_of_view - aim_field_of_view) * unit.aim).to_radians();
        let look_direction = unit.direction;
        let start_angle = look_direction.rotate(-fov / 2.0).angle();
        let end_angle = look_direction.rotate(fov / 2.0).angle();

        VisibilitySector {
            position: unit.position,
            radius: view_distance,
            start_angle,
            end_angle,
        }
    }
}