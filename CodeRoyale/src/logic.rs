use ai_cup_22::debugging::Color;
use crate::{DebugInterface, MyStrategy};
use ai_cup_22::model::*;

#[allow(dead_code)]
#[allow(unused_variables)]
impl MyStrategy {
    pub fn drink_shield(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        if unit.shield < self.constants.max_shield && unit.shield_potions > 0 {
            Some(UnitOrder {
                target_velocity: Vec2::zero(),
                target_direction: Vec2::zero(),
                action: Some(ActionOrder::UseShieldPotion {}),
            })
        } else {
            None
        }
    }

    pub fn pick_up_shield(&self, unit: &Unit, game: &Game) -> Option<UnitOrder> {
        if unit.shield_potions >= self.constants.max_shield_potions_in_inventory {
            return None;
        }
        let nearest_potion: Option<&Loot> = game
            .loot
            .iter()
            .filter(|loot| matches!(loot.item, Item::ShieldPotions{..}))
            .find(|loot| self.constants.unit_radius >= unit.position.distance_to(&loot.position));

        nearest_potion.map(|potion| {
            UnitOrder {
                target_velocity: Vec2::zero(),
                target_direction: potion.position.sub(&unit.position),
                action: Some(ActionOrder::Pickup { loot: potion.id }),
            }
        })
    }
    pub fn go_to_shield(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        if unit.shield_potions >= self.constants.max_shield_potions_in_inventory {
            return None;
        }

        let nearest_potion: Option<&Loot> = game
            .loot
            .iter()
            .filter(|loot| matches!(loot.item, Item::ShieldPotions{..}))
            .filter(|loot| loot.position.distance_to(&game.zone.current_center) <= game.zone.current_radius * 0.9)
            .min_by_key(|loot| unit.position.distance_to(&loot.position) as i32);

        nearest_potion.map(|potion| {
            if let Some(debug) = debug_interface.as_mut() {
                debug.add_segment(unit.position, potion.position, 0.2, Color::blue());
            }

            UnitOrder {
                target_velocity: potion.position.sub(&unit.position).mul(self.constants.max_unit_forward_speed),
                target_direction: potion.position.sub(&unit.position),
                action: None,
            }
        })
    }
    pub fn shoot_at_enemy(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        let weapon_idx = unit.weapon?;
        let weapon: &WeaponProperties = &self.constants.weapons[weapon_idx as usize];
        if unit.ammo[weapon_idx as usize] <= 0 {
            return None;
        }

        let nearest_enemy: Option<&Unit> = game
            .units
            .iter()
            .filter(|u| u.player_id != game.my_id)
            .filter(|enemy| enemy.is_hittable_by(unit, &self.constants))
            .filter(|enemy| unit.position.distance_to(&enemy.position) < weapon.projectile_life_time * weapon.projectile_speed)
            .min_by_key(|enemy| unit.position.distance_to(&enemy.position) as i32);

        nearest_enemy.map(|enemy| {
            UnitOrder {
                target_velocity: Vec2::zero(),
                target_direction: enemy.position.sub(&unit.position),
                action: Some(ActionOrder::Aim { shoot: true }),
            }
        })
    }
    pub fn scan_perimeter(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        Some(UnitOrder {
            target_velocity: Vec2::zero(),
            target_direction: Vec2 { x: -unit.direction.y, y: unit.direction.x },
            action: None,
        })
    }
    pub fn go_to_center_of_next_zone(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        if game.zone.next_center.distance_to(&unit.position) < self.constants.unit_radius {
            return None;
        }
        if let Some(debug) = debug_interface.as_mut() {
            debug.add_segment(unit.position, game.zone.next_center, 0.2, Color::blue());
        }
        Some(UnitOrder {
            target_velocity: game.zone.next_center.sub(&unit.position),
            target_direction: game.zone.next_center.sub(&unit.position),
            action: None,
        })
    }
    pub fn avoid_projectiles(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        let threatening_projectiles_exist = game
            .projectiles
            .iter()
            .filter(|p| p.shooter_player_id != game.my_id)
            .any(|p| p.position.distance_to(&unit.position) < p.life_time * p.velocity.length() - self.constants.unit_radius);

        if threatening_projectiles_exist {
            Some(UnitOrder {
                target_velocity: game.zone.next_center.sub(&unit.position),
                target_direction: game.zone.next_center,
                action: None,
            })
        } else {
            None
        }
    }

}