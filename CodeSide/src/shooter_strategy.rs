use crate::{DrawDebug, GameStrategy};
use model::*;

pub struct ShooterStrategy {}

static DEFAULT_LINE_WIDTH: f32 = 0.1;

impl ShooterStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl GameStrategy for ShooterStrategy {
    fn get_action(&mut self, unit: &Unit, game: &Game, debug: &mut DrawDebug) -> UnitAction {
        #[derive(PartialOrd, PartialEq)]
        struct F {
            i: f64,
        }
        impl Eq for F {}
        impl Ord for F {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.partial_cmp(other).unwrap()
            }
        }

        fn distance_sqr(a: &Vec2F64, b: &Vec2F64) -> F {
            F {
                i: (a.x - b.x).powi(2) + (a.y - b.y).powi(2),
            }
        }
        let nearest_enemy = game
            .units
            .iter()
            .filter(|other| other.player_id != unit.player_id)
            .min_by_key(|a| distance_sqr(&a.position, &unit.position));

        let nearest_weapon = game
            .loot_boxes
            .iter()
            .filter(|loot| {
                if let Item::Weapon { .. } = loot.item {
                    true
                } else {
                    false
                }
            })
            .min_by_key(|a| distance_sqr(&a.position, &unit.position));

        let mut target_pos = unit.position;
        if let (&None, Some(weapon)) = (&unit.weapon, nearest_weapon) {
            target_pos = weapon.position;
        }

        let mut shoot = false;
        let mut aim = Vec2F64 { x: 0.0, y: 0.0 };

        if let Some(enemy) = nearest_enemy {
            aim = enemy.position - unit.position;

            shoot = true
        }
        let mut jump = target_pos.y > unit.position.y;
        if target_pos.x > unit.position.x
            && game.level.tiles[unit.position.x as usize + 1][unit.position.y as usize]
                == Tile::Wall
        {
            jump = true
        }
        if target_pos.x < unit.position.x
            && game.level.tiles[unit.position.x as usize - 1][unit.position.y as usize]
                == Tile::Wall
        {
            jump = true
        }

        let swap_weapon = match (&unit.weapon, nearest_weapon) {
            (None, Some(_)) => true,

            (
                Some(Weapon {
                    typ: WeaponType::RocketLauncher,
                    ..
                }),
                _,
            ) => true,

            //            (
            //                Some(Weapon { typ: WeaponType::Pistol, .. }),
            //                Some(LootBox { item: Item::Weapon { weapon_type: WeaponType::AssaultRifle }, .. })
            //            ) => true,
            //            (
            //                Some(Weapon {
            //                    typ: WeaponType::AssaultRifle,
            //                    ..
            //                }),
            //                Some(LootBox {
            //                    item:
            //                        Item::Weapon {
            //                            weapon_type: WeaponType::Pistol,
            //                        },
            //                    ..
            //                }),
            //            ) => true,
            _ => false,
        };

        let plant_mine = false;

        let velocity = if target_pos.x < unit.position.x {
            -game.properties.unit_max_horizontal_speed
        } else if target_pos.x > unit.position.x {
            game.properties.unit_max_horizontal_speed
        } else {
            0.0
        };

        let reload = false;

        UnitAction {
            velocity,
            jump,
            jump_down: target_pos.y < unit.position.y,
            aim,
            shoot,
            reload,
            swap_weapon,
            plant_mine,
        }
    }
}
