use itertools::Itertools;
use ai_cup_22::debugging::*;
use ai_cup_22::model::*;
use crate::*;

#[allow(unused_variables)]
impl MyStrategy {
    pub fn debug_update(
        &mut self,
        displayed_tick: i32,
        debug_interface: &mut DebugInterface,
    ) {
        debug_interface.clear();
        debug_interface.set_auto_flush(false);
        let state = debug_interface.get_state();

        // self.show_weapon_ranges(debug_interface);
        // self.show_vision_ranges(debug_interface);

        let unit_under_cursor = self
            .enemy_units
            .iter()
            .find(|u| state.cursor_world_position.distance_to(&u.position) < self.constants.unit_radius);

        if let Some(unit) = unit_under_cursor {
            let my_units_that_see_this = self.my_units.iter().filter(|mu| self.unit_is_hittable_by(unit, mu, &self.constants, &mut Some(debug_interface))).collect_vec();
            // println!("enemy: {}, my units: {:?} / {}", unit.id, my_units_that_see_this.iter().map(|u| u.id).collect::<Vec<_>>(), self.my_units.len());
            for mu in my_units_that_see_this.iter() {
                debug_interface.add_segment(mu.position, unit.position, 0.2, Color::red());
                debug_interface.add_segment(mu.position, (unit.position - mu.position).rotate(self.angle_for_leading_shot(unit, mu, &mut None)), 0.2, Color::green().a(0.5));
            }
            let text = format!(
                "unit {}, hittable: {}",
                unit.id,
                !my_units_that_see_this.is_empty()
            );
            debug_interface.add_placed_text(unit.position, text, Vec2::zero(), 0.7, Color::blue());
        }
        debug_interface.flush();
    }

    pub(crate) fn visualize_sounds(&self, unit: &Unit, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) {
        if debug_interface.is_none() {
            return;
        }

        let debug = debug_interface.as_mut().unwrap();
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
            debug.add_placed_text(sound.position, label.to_string(), Vec2 { x: 0.5, y: 0.5 }, 1.0, Color::red());
        }
    }

    pub(crate) fn visualize_projectiles(&self, game: &Game, debug_interface: &mut Option<&mut DebugInterface>) {
        if debug_interface.is_none() {
            return;
        }

        let debug = debug_interface.as_mut().unwrap();

        for p in self.seen_projectiles.values() {
            let final_position = p.position + p.velocity * p.life_time;
            debug.add_segment(p.position, final_position, 0.1, Color::green().a(0.4))
        }
    }

    pub(crate) fn visualize_waypoint(&self, unit: &Unit, debug_interface: &mut Option<&mut DebugInterface>) {
        if let Some(wp) = self.waypoints.get(&unit.id) {
            if let Some(debug) = debug_interface.as_mut() {
                debug.add_circle(*wp, 1.3, Color::blue().a(0.6));
                debug.add_placed_text(*wp, format!("wp for unit {}", unit.id), Vec2::zero(), 0.6, Color::blue().a(0.8));
            }
        }
    }

    fn show_weapon_ranges(&self, debug_interface: &mut DebugInterface) {
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
    }

    pub fn show_vision_ranges(&self, debug_interface: &mut DebugInterface) {
        for my_unit in self.my_units.iter() {
            let sector = self.unit_visibility_sector(my_unit);
            debug_interface.add_pie(sector.position, sector.radius, sector.start_angle, sector.end_angle, Color::green().a(0.3));
        }
    }

    pub fn place_label(&self, position: Vec2, text: String, line: usize, debug_interface: &mut Option<&mut DebugInterface>) {
        let offset_y = line as f64 * 0.8;
        if let Some(debug) = debug_interface.as_mut() {
            debug.add_placed_text(position, text, Vec2{x: 0.0, y: offset_y}, 1.3, Color::blue());
        }
    }

    pub fn show_status_labels_for_units(&mut self, mut debug_interface: &mut Option<&mut DebugInterface>) {
        if let Some(debug) = debug_interface.as_mut() {
            for unit in self.my_units.iter() {
                let text = match unit.action {
                    Some(Action { action_type: ActionType::Looting, .. }) => Some("looting"),
                    Some(Action { action_type: ActionType::UseShieldPotion, .. }) => Some("drinking shield"),
                    _ => None,
                };
                if let Some(t) = text {
                    self.place_label(unit.position, t.to_string(), 3, &mut debug_interface);
                }
            }
        }
    }

}