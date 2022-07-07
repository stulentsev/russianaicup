use std::borrow::BorrowMut;
use std::collections::HashMap;
use libc::posix_fadvise;

use ai_cup_22::*;
use ai_cup_22::debugging::Color;
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
        mut debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        let mut orders = HashMap::new();

        for unit in game.units.iter() {
            if unit.player_id != game.my_id {
                continue;
            }

            if unit.action.is_some() {
                continue;
            }

            let maybe_order = None
                .or_else(|| self.drink_shield(unit, game, &mut debug_interface))
                .or_else(|| self.pick_up_shield(unit, game))
                .or_else(|| self.go_to_shield(unit, game, &mut debug_interface))
                .or_else(|| self.go_to_center_of_next_zone(unit, game, &mut debug_interface));

            if let Some(order) = maybe_order {
                orders.insert(unit.id, order);
            }
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

    fn drink_shield(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
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

    fn pick_up_shield(&self, unit: &Unit, game: &Game) -> Option<UnitOrder> {
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

    fn go_to_shield(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        if unit.shield_potions >= self.constants.max_shield_potions_in_inventory {
            return None;
        }
        let nearest_potion: Option<&Loot> = game
            .loot
            .iter()
            .filter(|loot| matches!(loot.item, Item::ShieldPotions{..}))
            .min_by_key(|loot| unit.position.distance_to(&loot.position) as i32);

        nearest_potion.map(|potion| {
            if let Some(mut debug) = debug_interface.as_mut() {
                debug.add_segment(unit.position.clone(), potion.position.clone(), 0.2, Color::blue());
            }

            UnitOrder {
                target_velocity: potion.position.sub(&unit.position).mul(self.constants.max_unit_forward_speed),
                target_direction: potion.position.sub(&unit.position),
                action: None,
            }
        })
    }

    fn shoot_randomly(&self, unit: &Unit, game: &Game) -> Option<UnitOrder> {
        Some(UnitOrder {
            target_velocity: Vec2::zero(),
            target_direction: Vec2 { x: -unit.direction.y, y: unit.direction.x },
            action: Some(ActionOrder::Aim { shoot: true }),
        })
    }

    fn go_to_center_of_next_zone(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        if game.zone.next_center.distance_to(&unit.position) < self.constants.unit_radius {
            return None;
        }
        if let Some(mut debug) = debug_interface.as_mut() {
            debug.add_segment(unit.position.clone(), game.zone.next_center.clone(), 0.2, Color::blue());
        }
        Some(UnitOrder {
            target_velocity: game.zone.next_center.sub(&unit.position),
            target_direction: game.zone.next_center.clone(),
            action: None,
        })
    }
}