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

        self.show_weapon_ranges(debug_interface);

        let unit_under_cursor = self
            .enemy_units
            .iter()
            .find(|u| state.cursor_world_position.distance_to(&u.position) < self.constants.unit_radius);

        if let Some(unit) = unit_under_cursor {
            let my_units_that_see_this = self.my_units.iter().filter(|mu| self.unit_is_hittable_by(unit, mu, &self.constants, &mut Some(debug_interface))).collect::<Vec<_>>();
            // println!("enemy: {}, my units: {:?} / {}", unit.id, my_units_that_see_this.iter().map(|u| u.id).collect::<Vec<_>>(), self.my_units.len());
            for mu in my_units_that_see_this.iter() {
                debug_interface.add_segment(mu.position, unit.position, 0.2, Color::red());
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

        for p in game.projectiles.iter() {
            let final_position = p.position + p.velocity * p.life_time;
            debug.add_segment(p.position, final_position, 0.1, Color::green().a(0.4))
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
}