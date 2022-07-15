use std::collections::HashMap;
use itertools::Itertools;
use ai_cup_22::model::*;
use crate::{BasicGameEntity, MyStrategy};

impl MyStrategy {
    pub(crate) fn rebuild_indexes(&mut self, game: &Game) {
        self.units_by_id.clear();
        self.my_units.clear();
        self.enemy_units.clear();
        self.targets.clear();

        self.units_by_id = game.units.iter().map(|u| (u.id, u.clone())).collect();
        self.my_units = game.units.iter().filter(|u| u.player_id == game.my_id).cloned().collect();
        self.enemy_units = game.units.iter().filter(|u| u.player_id != game.my_id).cloned().collect();

        self.current_tick = game.current_tick;

        self.update_loot(game);
        self.update_projectiles(game);
    }

    fn update_loot(&mut self, game: &Game) {
        self.seen_loot = self.update_seen_items(&self.seen_loot, &game.loot, game.current_tick);
    }

    fn update_projectiles(&mut self, game: &Game) {
        self.seen_projectiles = self.update_seen_items(&self.seen_projectiles, &game.projectiles, game.current_tick);

        for visible_projectile in game.projectiles.iter() {
            self.seen_projectiles.entry(visible_projectile.id())
                .and_modify(|item| {
                    item.life_time = visible_projectile.life_time;
                    item.position = visible_projectile.position;
                });
        }
    }

    fn update_seen_items<T: BasicGameEntity + Clone>(&self, source: &HashMap<i32, T>, new_items: &[T], current_tick: i32) -> HashMap<i32, T> {
        let visibility_sectors = self.my_units.iter().map(|unit| self.unit_visibility_sector(unit)).collect_vec();
        let item_by_id = source.iter().map(|(id, item)| (item.id(), item)).collect::<HashMap<_, _>>();
        // prune items no longer there
        let mut seen_items: HashMap<i32, T> = source.iter()
            .filter(|(id, item)| { // only currently visible loot
                let visible_at_the_moment = visibility_sectors.iter().any(|sec| sec.cover_point(item.position()));
                if visible_at_the_moment {
                    item_by_id.contains_key(&item.id())
                } else {
                    item.is_still_relevant(current_tick)
                }
            })
            .map(|(id, item)| (*id, item.clone()))
            .collect();


        for visible_item in new_items.iter() {
            seen_items.entry(visible_item.id())
                .and_modify(|item| item.mark_seen(current_tick))
                .or_insert_with(|| {
                    let mut new_loot = visible_item.clone();
                    new_loot.mark_seen(current_tick);
                    new_loot
                });
        }

        seen_items
    }

    pub fn process_sounds(&mut self, game: &Game) {
        let visibility_sectors = self.my_units.iter().map(|unit| self.unit_visibility_sector(unit)).collect_vec();

        for sound in game.sounds.iter() {
            let source = sound.position;
            let visible_at_the_moment = visibility_sectors.iter().any(|sec| sec.cover_point(source));

            if visible_at_the_moment {
                continue
            }

            let id = self.gen_imaginary_id();
            let unit_that_heard = self.my_units.iter().find(|u| u.id == sound.unit_id).unwrap();
            match sound.type_index {
                1 => {
                    self.seen_projectiles.entry(id).or_insert(Projectile{
                        id,
                        weapon_type_index: 0,
                        shooter_id: 0,
                        shooter_player_id: 0,
                        position: source,
                        velocity: (unit_that_heard.position - source).clamp(30.0),
                        life_time: 1.0,
                        seen_on_tick: game.current_tick,
                    });
                },
                2 => {
                    self.seen_projectiles.entry(id).or_insert(Projectile{
                        id,
                        weapon_type_index: 1,
                        shooter_id: 0,
                        shooter_player_id: 0,
                        position: source,
                        velocity: (unit_that_heard.position - source).clamp(20.0),
                        life_time: 1.0,
                        seen_on_tick: game.current_tick,
                    });
                },
                3 => {
                    self.seen_projectiles.entry(id).or_insert(Projectile{
                        id,
                        weapon_type_index: 2,
                        shooter_id: 0,
                        shooter_player_id: 0,
                        position: source,
                        velocity: (unit_that_heard.position - source).clamp(40.0),
                        life_time: 1.0,
                        seen_on_tick: game.current_tick,
                    });

                },
                // 4 => "wand hit",
                // 5 => "staff hit",
                // 6 => "bow hit",
                _ => ()
            };

        }
    }

    fn gen_imaginary_id(&mut self) -> i32 {
        self.next_imaginary_id -= 1;
        self.next_imaginary_id
    }
}