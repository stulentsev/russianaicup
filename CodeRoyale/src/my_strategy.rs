use std::collections::HashMap;
use libc::posix_fadvise;

use ai_cup_22::*;
use ai_cup_22::model::*;

use crate::debug_interface::DebugInterface;

pub struct MyStrategy {
    constants: Constants,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        Self { constants }
    }
    pub fn get_order(
        &mut self,
        game: &Game,
        debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        let mut orders = HashMap::new();

        for unit in game.units.iter() {
            if unit.player_id != game.my_id {
                continue;
            }

            if unit.action.is_some() {
                continue
            }
            let nearest_potion: Option<&Loot> = game.loot.iter().filter(|loot| matches!(loot.item, Item::ShieldPotions{..})).min_by_key(|loot| loot.position.sub(&unit.position).length() as i32);

            let order = if unit.shield < self.constants.max_shield && unit.shield_potions > 0 {
                UnitOrder {
                    target_velocity: Vec2::zero(),
                    target_direction: Vec2::zero(),
                    action: Some(ActionOrder::UseShieldPotion {}),
                }
            } else if let Some(ref potion) = nearest_potion {
                if unit.shield_potions < self.constants.max_shield_potions_in_inventory {
                    if self.constants.unit_radius >= unit.position.distance_to(&potion.position) {
                        // pickup potion
                        UnitOrder {
                            target_velocity: Vec2::zero(),
                            target_direction: potion.position.sub(&unit.position),
                            action: Some(ActionOrder::Pickup { loot: potion.id }),
                        }
                    } else {
                        // move to potion
                        UnitOrder {
                            target_velocity: potion.position.sub(&unit.position).mul(self.constants.max_unit_forward_speed),
                            target_direction: potion.position.sub(&unit.position),
                            action: None,
                        }
                    }
                } else {
                    // default
                    UnitOrder {
                        target_velocity: Vec2 { x: -unit.position.x, y: -unit.position.y },
                        target_direction: Vec2 { x: -unit.direction.y, y: unit.direction.x },
                        action: Some(ActionOrder::Aim { shoot: true }),
                    }
                }
            } else {
                // default
                UnitOrder {
                    target_velocity: Vec2 { x: -unit.position.x, y: -unit.position.y },
                    target_direction: Vec2 { x: -unit.direction.y, y: unit.direction.x },
                    action: Some(ActionOrder::Aim { shoot: true }),
                }
            };

            orders.insert(unit.id, order);
        }
        Order {
            unit_orders: orders,
        }
    }
    pub fn debug_update(
        &mut self,
        debug_interface: &mut DebugInterface,
    ) {}
    pub fn finish(&mut self) {}

    fn max_forward_speed(&self) -> Vec2 {
        Vec2{
            x: self.constants.max_unit_forward_speed,
            y: self.constants.max_unit_forward_speed,
        }
    }
}