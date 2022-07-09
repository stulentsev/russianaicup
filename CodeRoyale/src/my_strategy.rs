use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::f64::consts::FRAC_PI_2;

use ai_cup_22::*;
use ai_cup_22::debugging::Color;
use ai_cup_22::model::*;

use crate::debug_interface::DebugInterface;

pub struct MyStrategy {
    constants: Constants,
    units: Vec<Unit>,
    my_units: Vec<Unit>,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        Self { constants, units: vec![], my_units: vec![] }
    }
    pub fn get_order(
        &mut self,
        game: &Game,
        mut debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        let mut orders = HashMap::new();

        self.units = game.units.clone();
        self.my_units = game.units.iter().filter(|u| u.player_id == game.my_id).cloned().collect();

        for unit in game.units.iter() {
            if unit.player_id != game.my_id {
                continue;
            }

            if unit.action.is_some() {
                continue;
            }

            self.visualize_sounds(unit, game, &mut debug_interface);

            let maybe_order = None
                // .or_else(|| self.avoid_projectiles(unit, game, &mut debug_interface))
                .or_else(|| self.drink_shield(unit, game, &mut debug_interface))
                .or_else(|| self.pick_up_shield(unit, game))
                // .or_else(|| self.go_to_shield(unit, game, &mut debug_interface))
                .or_else(|| self.shoot_at_enemy(unit, game, &mut debug_interface))
                .or_else(|| self.go_to_center_of_next_zone(unit, game, &mut debug_interface))
                .or_else(|| self.scan_perimeter(unit, game, &mut debug_interface));

            if let Some(order) = maybe_order {
                orders.insert(unit.id, order);
            }
        }
        if let Some(debug) = debug_interface {
            debug.flush();
        }
        Order {
            unit_orders: orders,
        }
    }

    pub fn debug_update(
        &mut self,
        debug_interface: &mut DebugInterface,
    ) {
        debug_interface.clear();
        debug_interface.set_auto_flush(false);
        let state = debug_interface.get_state();

        for my_unit in self.my_units.iter() {
            if let Some(weapon_idx) = my_unit.weapon {
                let weapon: &WeaponProperties = &self.constants.weapons[weapon_idx as usize];
                let look_angle = my_unit.direction.angle();

                debug_interface.add_pie(
                    my_unit.position,
                    weapon.projectile_life_time * weapon.projectile_speed,
                    look_angle - weapon.aim_field_of_view.to_radians() / 2.0,
                     look_angle + weapon.aim_field_of_view.to_radians() / 2.0,
                    Color::red().a(0.1),
                )
            }
        }

        let unit_under_cursor = self
            .units
            .iter()
            .find(|u| state.cursor_world_position.distance_to(&u.position) < self.constants.unit_radius);

        if let Some(unit) = unit_under_cursor {
            let my_units_that_see_this = self.my_units.iter().filter(|mu| unit.is_hittable_by(mu, &self.constants)).collect::<Vec<_>>();
            // println!("enemy: {}, my units: {:?} / {}", unit.id, my_units_that_see_this.iter().map(|u| u.id).collect::<Vec<_>>(), self.my_units.len());
            for mu in my_units_that_see_this.iter() {
                debug_interface.add_segment(mu.position.clone(), unit.position.clone(), 0.2, Color::red());
            }
            let text = format!(
                "unit {}, hittable: {}",
                unit.id,
                !my_units_that_see_this.is_empty()
            );
            debug_interface.add_placed_text(unit.position.clone(), text, Vec2::zero(), 0.7, Color::blue());
        }
        debug_interface.flush();
    }
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
            .filter(|loot| loot.position.distance_to(&game.zone.current_center) <= game.zone.current_radius * 0.9)
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

    fn shoot_at_enemy(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        if unit.weapon.is_none() {
            return None;
        }
        let weapon_idx = unit.weapon.unwrap();
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

    fn scan_perimeter(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        Some(UnitOrder {
            target_velocity: Vec2::zero(),
            target_direction: Vec2 { x: -unit.direction.y, y: unit.direction.x },
            action: None,
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
            target_direction: game.zone.next_center.sub(&unit.position),
            action: None,
        })
    }

    fn avoid_projectiles(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) -> Option<UnitOrder> {
        let threatening_projectiles_exist = game
            .projectiles
            .iter()
            .filter(|p| p.shooter_player_id != game.my_id)
            .any(|p| p.position.distance_to(&unit.position) < p.life_time * p.velocity.length() - self.constants.unit_radius);

        if threatening_projectiles_exist {
            Some(UnitOrder {
                target_velocity: game.zone.next_center.sub(&unit.position),
                target_direction: game.zone.next_center.clone(),
                action: None,
            })
        } else {
            None
        }
    }

    fn visualize_sounds(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) {
        if debug_interface.is_none() {
            return;
        }

        let mut debug = debug_interface.as_mut().unwrap();
        for sound in game.sounds.iter() {
            let label = match sound.type_index {
                0 => "steps",
                1 => "wand",
                2 => "staff",
                3 => "bow",
                4 => "wand hit",
                5 => "staff hit",
                6 => "bow hit",
                _ => unreachable!("unexpected sound type index {}", sound.type_index)
            };
            debug.add_placed_text(sound.position.clone(), label.to_string(), Vec2 { x: 0.5, y: 0.5 }, 1.0, Color::red());
        }
    }
}
