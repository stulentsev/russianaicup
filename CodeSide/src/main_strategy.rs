use crate::{DrawDebug, GameStrategy};
use model::*;
use simulation::*;
use crate::pathfinding::{absdiff, astar};
use std::collections::HashMap;
use std::ops::{Sub, Add};

const PATHFINDING_PPT: f64 = 1.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct WpCoord(i32);

impl WpCoord {
    fn from_f64(x: f64) -> Self {
        Self((x / PATHFINDING_PPT) as i32)
    }

    fn to_i32(&self) -> i32 {
        self.0 as i32
    }

    fn to_u32(&self) -> u32 {
        self.0 as u32
    }

    fn to_f32(&self) -> f32 {
        self.to_f64() as f32
    }

    fn to_f64(&self) -> f64 {
        self.0 as f64 * PATHFINDING_PPT
    }

    fn eq_f64(&self, other: f64) -> bool {
        self.0 == (other / PATHFINDING_PPT) as i32
    }
}

impl Sub for WpCoord {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 as i32 - other.0 as i32)
    }
}

impl Add for WpCoord {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self(self.0 as i32 + other.0 as i32)
    }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Waypoint {
    pub x: WpCoord,
    pub y: WpCoord,

    pub can_jump: bool,
    pub jump_time_msec: i32,
    pub jump_speed: i32,
    pub can_cancel: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Pos(WpCoord, WpCoord);

impl Pos {
    fn from_vec64(vec: &Vec2F64) -> Self {
        Self(WpCoord::from_f64(vec.x), WpCoord::from_f64(vec.y))
    }
}

impl Waypoint {
    fn create(x: WpCoord, y: WpCoord) -> Self {
        Self {
            x,
            y,
            can_jump: true,
            jump_time_msec: 550,
            jump_speed: 10,
            can_cancel: true,
        }
    }
    fn create_fall(x: WpCoord, y: WpCoord) -> Self {
        Self {
            x,
            y,
            can_jump: false,
            jump_time_msec: 0,
            jump_speed: 0,
            can_cancel: false,
        }
    }

    fn bounding_box(&self) -> BoundingBox {
        let radius = 1.0;
        BoundingBox {
            bottom_left: Vec2F64 { x: self.x.to_f64(), y: self.y.to_f64() },
            size: Vec2F64 { x: radius * 2.0, y: radius * 2.0 },
        }
    }


    fn distance(&self, other: &Pos) -> u32 {
        (absdiff(self.x, other.0) + absdiff(self.y, other.1)).to_u32()
    }

    fn successors(&self, game: &Game, unit: &Unit) -> Vec<(Waypoint, u32)> {
        let jump_time_per_wp_msec = (1000.0 / (10.0 * PATHFINDING_PPT)) as i32;

        fn jumpable_from(tile: Tile) -> bool {
            match tile {
                Tile::Wall | Tile::Platform | Tile::Ladder => true,
                _ => false
            }
        }

        fn jumpable_through(tile: Tile) -> bool {
            match tile {
                Tile::Empty | Tile::Platform | Tile::Ladder => true,
                _ => false
            }
        }
        let jumpable_diagonal = |fx: f64, fy: f64| -> bool {
            let x = fx as i32;
            let y = fy as i32;
            jumpable_through(game.level.cell_at_i32(x, y)) &&
                jumpable_through(game.level.cell_at_i32(x, y + 1)) &&
                jumpable_through(game.level.cell_at_i32(x, y - 1))
        };
        let enemy_is_on_waypoint = |wp: &Waypoint| -> bool {
            game.units.iter().
                filter(|u| u.player_id != unit.player_id).
                any(|u| {
                    u.position.x as i32 == wp.x.to_f64() as i32 &&
                        (u.position.y as i32 == wp.y.to_f64() as i32 || (u.position.y + 1.0) as i32 == wp.y.to_f64() as i32)
                })
        };

        let within_mine_blast_radius = |wp: &Waypoint| {
            game.mines.iter().any(|mine| {
                wp.bounding_box().intersects(&mine.explosion_bounding_box())
            })
        };

        let mut tiles: Vec<Waypoint> = vec![];

        let mut add_successor = |wp: Waypoint| {
            let tile = game.level.cell_at_f64(wp.x.to_f64(), wp.y.to_f64());
            if tile != Tile::Wall /* && !within_mine_blast_radius(&wp)*/ {
                tiles.push(wp)
            }
        };

        let top = game.level.cell_at_f64(self.x.to_f64(), self.y.to_f64() + 1.0);
        let bottom = game.level.cell_at_f64(self.x.to_f64(), self.y.to_f64() - 1.0);
        let bottom_left = game.level.cell_at_f64(self.x.to_f64() - 1.0, self.y.to_f64() - 1.0);
        let bottom_right = game.level.cell_at_f64(self.x.to_f64() + 1.0, self.y.to_f64() - 1.0);
        let left = game.level.cell_at_f64(self.x.to_f64() - 1.0, self.y.to_f64());
        let right = game.level.cell_at_f64(self.x.to_f64() + 1.0, self.y.to_f64());
        let top_right = game.level.cell_at_f64(self.x.to_f64() + 1.0, self.y.to_f64() + 1.0);

        let mut can_jump = self.can_jump && self.jump_time_msec > jump_time_per_wp_msec;
        let mut jump_speed = game.properties.unit_jump_speed as i32;
        let mut can_cancel = false;
        let mut jump_time_msec = (game.properties.unit_jump_time * 1000.0) as i32;

        if Tile::Ladder == game.level.cell_at_f64(self.x.to_f64(), self.y.to_f64()) {
            can_jump = true;
            jump_speed = game.properties.unit_jump_speed as i32;
            can_cancel = true;
            jump_time_msec = (game.properties.unit_jump_time * 1000.0) as i32;
        }

        if Tile::JumpPad == game.level.cell_at_f64(self.x.to_f64(), self.y.to_f64()) {
            can_jump = true;
            jump_speed = game.properties.jump_pad_jump_speed as i32;
            can_cancel = false;
            jump_time_msec = (game.properties.jump_pad_jump_time * 1000.0) as i32;
        }

        let jump_tiles = self.jump_speed / 10;

        if can_jump {
            if jumpable_through(game.level.cell_at_f64(self.x.to_f64(), self.y.to_f64() + 1.0)) &&
                jumpable_through(game.level.cell_at_f64(self.x.to_f64(), self.y.to_f64() + 2.0)) {
                add_successor(Waypoint {
                    x: self.x,
                    y: self.y + WpCoord(jump_tiles),
                    can_jump,
                    jump_time_msec: self.jump_time_msec - jump_time_per_wp_msec,
                    jump_speed: self.jump_speed,
                    can_cancel: self.can_cancel,
                })
            };

            if jumpable_diagonal(self.x.to_f64() - 1.0, self.y.to_f64() + jump_tiles as f64) {
                add_successor(Waypoint {
                    x: self.x - WpCoord(1),
                    y: self.y + WpCoord(jump_tiles),
                    can_jump,
                    jump_time_msec: self.jump_time_msec - jump_time_per_wp_msec,
                    jump_speed: self.jump_speed,
                    can_cancel: self.can_cancel,
                })
            };
            if jumpable_diagonal(self.x.to_f64() + 1.0, self.y.to_f64() + jump_tiles as f64) {
                add_successor(Waypoint {
                    x: self.x + WpCoord(1),
                    y: self.y + WpCoord(jump_tiles),
                    can_jump,
                    jump_time_msec: self.jump_time_msec - jump_time_per_wp_msec,
                    jump_speed: self.jump_speed,
                    can_cancel: self.can_cancel,
                });
            };
        }
        if !can_jump || self.can_cancel {
            add_successor(Waypoint {
                x: self.x,
                y: self.y - WpCoord(1),
                can_jump,
                jump_time_msec: 0,
                jump_speed: 0,
                can_cancel: false,
            });

            if jumpable_diagonal(self.x.to_f64() - 1.0, self.y.to_f64() - 1.0) {
                add_successor(Waypoint {
                    x: self.x - WpCoord(1),
                    y: self.y - WpCoord(1),
                    can_jump,
                    jump_time_msec: 0,
                    jump_speed: 0,
                    can_cancel: false,
                });
            }

            if jumpable_diagonal(self.x.to_f64() + 1.0, self.y.to_f64() - 1.0) {
                add_successor(Waypoint {
                    x: self.x + WpCoord(1),
                    y: self.y - WpCoord(1),
                    can_jump,
                    jump_time_msec: 0,
                    jump_speed: 0,
                    can_cancel: false,
                });
            }
        }
        add_successor(Waypoint {
            x: self.x - WpCoord(1),
            y: self.y,
            can_jump,
            jump_time_msec,
            jump_speed,
            can_cancel,
        });
        add_successor(Waypoint {
            x: self.x + WpCoord(1),
            y: self.y,
            can_jump,
            jump_time_msec,
            jump_speed,
            can_cancel,
        });

        if jumpable_from(bottom) {
            if jumpable_from(bottom_right) {
                add_successor(Waypoint::create(self.x + WpCoord(1), self.y));
            }
            if jumpable_from(bottom_left) {
                add_successor(Waypoint::create(self.x - WpCoord(1), self.y));
            }
        }

        tiles.into_iter().map(|p| (p, 1)).collect()
    }
}

static COLOR_RED50: ColorF32 = ColorF32 {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 0.5,
};
static COLOR_GREEN50: ColorF32 = ColorF32 {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 0.5,
};
static COLOR_BLUE50: ColorF32 = ColorF32 {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 0.5,
};
static COLOR_YELLOW50: ColorF32 = ColorF32 {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 0.5,
};

static DEFAULT_LINE_WIDTH: f32 = 0.1;

enum TargetType {
    None,
    Weapon,
    HealthPack,
    Enemy,
}

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

pub struct MainStrategy {
    saved_paths: HashMap<i32, Vec<Waypoint>>,
    targets: HashMap<i32, Pos>,
    jump_origins: HashMap<i32, Pos>,
    player_id: i32,
    going_to_hp: bool,
}

impl MainStrategy {
    pub fn new() -> Self {
        Self {
            saved_paths: HashMap::new(),
            targets: HashMap::new(),
            jump_origins: HashMap::new(),
            player_id: -1,
            going_to_hp: false,
        }
    }

    fn add_target(&mut self, unit: &Unit, pos: Pos) {
        self.targets.insert(unit.id, pos);
    }

    fn is_targeted_by_other_unit(&self, unit: &Unit, pos: Pos) -> bool {
        for (other_uid, other_pos) in &self.targets {
            if pos == *other_pos && *other_uid != unit.id {
                return true;
            }
        }
        return false;
    }

    fn current_target_for_unit(&self, unit: &Unit) -> Option<&Pos> {
        self.targets.get(&unit.id)
    }

    fn remove_target_for_unit(&mut self, unit: &Unit) {
        self.targets.remove(&unit.id);
    }

    fn remove_target(&mut self, pos: Pos) {
        self.targets.retain(|uid, other_pos| pos != *other_pos);
    }

    fn start_jump(&mut self, unit: &Unit, pos: Pos) {
        self.jump_origins.insert(unit.id, pos);
    }

    fn is_jump_origin(&self, unit: &Unit, pos: Pos) -> bool {
        match self.jump_origins.get(&unit.id) {
            Some(jump_origin) => jump_origin.1 == pos.1,
            None => false,
        }
    }

    fn clear_jump_origin(&mut self, unit: &Unit) {
        self.jump_origins.remove(&unit.id);
    }

    // if wounded, go to nearest HP
    // if no weapon, go to nearest preferable weapon
    // if there are mines and path is empty, go to nearest mine
    // if there are no mines, go to nearest enemy (re-path every 5 ticks or so)
    fn choose_path(&mut self, unit: &Unit, game: &Game, debug: &mut DrawDebug) {
        self.going_to_hp = false;
        if unit.need_healing(game) && game.has_unpicked_hp() {
            // path to nearest HP
            let nearest_hp = game.loot_boxes.iter().filter(|lb| {
                match lb.item {
                    Item::HealthPack { .. } => {
                        !has_enemy_between(game, unit, lb, debug) && !self.is_targeted_by_other_unit(
                            unit,
                            Pos::from_vec64(&lb.position),
                        )
                    }
                    _ => false,
                }
            }).min_by_key(|lb| distance_sqr(&lb.position, &unit.position));
            if let Some(hp) = nearest_hp {
                let path = find_path(game, unit, hp.position).unwrap_or(vec![]);
                self.set_unit_path(unit, path);
                self.going_to_hp = true;
                if let Some(tpos) = self.current_target_for_unit(unit) {
                    debug.log(format_args!("Going to HP: {:?},{:?}", tpos.0, tpos.1))
                }
                return;
            }
        }
        if unit.weapon.is_none() {
            // path to nearest preferable weapon
            let nearest_weapon = game.loot_boxes.iter().filter(|lb| {
                match lb.item {
                    Item::Weapon { weapon_type: WeaponType::RocketLauncher } => false,
                    Item::Weapon { .. } => {
                        !has_enemy_between(game, unit, lb, debug) && !self.is_targeted_by_other_unit(
                            unit,
                            Pos::from_vec64(&lb.position),
                        )
                    }
                    _ => false,
                }
            }).min_by_key(|lb| distance_sqr(&lb.position, &unit.position));
            if let Some(weapon) = nearest_weapon {
                let path = find_path(game, unit, weapon.position).unwrap_or(vec![]);
                self.set_unit_path(unit, path);
                return;
            }
        }
        if game.has_unpicked_mines() {
            // path to nearest mine
            let nearest_mine = game.loot_boxes.iter().filter(|lb| {
                match lb.item {
                    Item::Mine { .. } => {
                        !has_enemy_between(game, unit, lb, debug) && !self.is_targeted_by_other_unit(
                            unit,
                            Pos::from_vec64(&lb.position),
                        )
                    }
                    _ => false,
                }
            }).min_by_key(|lb| distance_sqr(&lb.position, &unit.position));
            if let Some(mine) = nearest_mine {
                let path = find_path(game, unit, mine.position).unwrap_or(vec![]);
                self.set_unit_path(unit, path);
                return;
            }
        }
        // path to nearest enemy
        let nearest_enemy = game.units.iter().filter(|lb| {
            lb.player_id != unit.player_id
        }).min_by_key(|lb| distance_sqr(&lb.position, &unit.position));

        if let Some(enemy) = nearest_enemy {
            let path = find_path(game, unit, enemy.position).unwrap_or(vec![]);
            self.set_unit_path(unit, path);
        }
    }

    fn set_unit_path(&mut self, unit: &Unit, path: Vec<Waypoint>) {
        if let Some(wp) = path.last() {
            let pos = Pos(wp.x, wp.y);
            self.add_target(unit, pos);

            self.saved_paths.insert(unit.id, path);
        } else {
            self.remove_target_for_unit(unit);
            self.saved_paths.remove(&unit.id);
        }
    }
}

impl GameStrategy for MainStrategy {
    fn get_action(&mut self, unit: &Unit, game: &Game, debug: &mut DrawDebug) -> UnitAction {
        self.player_id = unit.player_id;

        draw_grid(game, debug);

        if game.current_tick % 12 == 0 {
            self.choose_path(unit, game, debug);
        }

        let mut empty_path = vec![];
        let mut saved_path = self.saved_paths.get_mut(&unit.id).unwrap_or(&mut empty_path);

        if !saved_path.is_empty() {
            debug.log(format_args!("PATH LENGTH {}", saved_path.len()));
            draw_path(&saved_path, debug);
        } else {
            debug.log(format_args!("NO PATH"))
        }

        //        debug.log(format_args!(
//            "{:?}, stand {}, ground {}, ladder {}",
//            unit.jump_state,
//            unit.stand,
//            unit.on_ground,
//            unit.on_ladder,
//        ));
        let nearest_enemy = game
            .units
            .iter()
            .filter(|other| other.player_id != unit.player_id)
            .min_by_key(|a| distance_sqr(&a.position, &unit.position));

        let mut target_pos = unit.position;
        let mut target_type = TargetType::None;
        let mut target_bounding_box = None;

        let is_last_wp = saved_path.len() < 2;
        saved_path.retain(|wp| {
            absdiff(wp.x, WpCoord::from_f64(unit.position.x)) > WpCoord(0) ||
                absdiff(wp.y, WpCoord::from_f64(unit.position.y)) > WpCoord(0) &&
                    absdiff(wp.y, WpCoord::from_f64(unit.position.y + unit.size.y)) > WpCoord(0)
        });
        let unit_wp = Waypoint::create(WpCoord::from_f64(unit.position.x), WpCoord::from_f64(unit.position.y));
        let target_wp = saved_path.first().unwrap_or(&unit_wp);
        target_pos = Vec2F64 { x: target_wp.x.to_f64() + 0.5, y: target_wp.y.to_f64() };

        debug.log(format_args!("Target pos: {:?}", target_pos));
        debug.log(format_args!("Unit pos: {:?}", unit.position));

        let mut shoot = false;
        let mut aim = Vec2F64 { x: 0.0, y: 0.0 };

        if let Some(enemy) = nearest_enemy {
            aim = enemy.position - unit.position;

            shoot = should_shoot(game, unit, enemy, &aim, debug);
        }

        let mut jump = should_jump(&target_pos, target_type, unit, game);

        let swap_weapon = unit.weapon.is_none() || unit.weapon.unwrap().typ == WeaponType::RocketLauncher;
//        let plant_mine = false;
        let plant_mine = unit.mines > 0 && self.going_to_hp && next_to_enemy(game, unit);
//        let plant_mine = unit.mines > 0 && next_to_enemy(game, unit);


        let mut velocity = if target_pos.x < unit.position.x {
            -game.properties.unit_max_horizontal_speed
        } else {
            game.properties.unit_max_horizontal_speed
        };

//        if unit.has_weapon() && (unit.position.x as usize) >= game.level.tiles.len() / 3 && velocity > 0.0 {
//            velocity = 0.0;
//        }


        let reload = false;
        let mut jump_down = (target_pos.y as usize) < (unit.position.y as usize) && (target_pos.x as usize) == (unit.position.x as usize);

        highlight_target(target_bounding_box, debug);


        // quickly restart jump
        let pos = Pos::from_vec64(&unit.position);
        if jump {
            let is_ground = game.level.cell_at_i32(pos.0.to_i32(), (pos.1 - WpCoord(1)).to_i32()).can_jump_from();
            if !self.is_jump_origin(unit, pos) && is_ground && !unit.on_ground {
                self.clear_jump_origin(unit);
                jump = false;
            } else {
                self.start_jump(unit, pos);
            }
        } else {
            self.clear_jump_origin(unit);
        }

        let (velocity, jump, jump_down) = execute_evasive_maneuver(game, unit, velocity, jump, jump_down, debug);

        UnitAction {
            velocity,
            jump,
            jump_down,
            aim,
            shoot,
            reload,
            swap_weapon,
            plant_mine,
        }
    }
}

fn execute_evasive_maneuver(game: &Game, unit: &Unit, velocity: f64, jump: bool, jump_down: bool, debug: &mut DrawDebug) -> (f64, bool, bool) {
    if safe_move(game, unit, velocity, jump, debug) {
        debug.log(format_args!("Continue {} {} {}", velocity, jump, jump_down));
        return (velocity, jump, jump_down);
    }

    let velocity_steps: i32 = 10;

    for j in vec![true, false] {
        for vs in -velocity_steps..=velocity_steps {
            let v = game.properties.unit_max_horizontal_speed / (velocity_steps as f64) * (vs as f64);
            if safe_move(game, unit, v, j, debug) {
                debug.log(format_args!("Evasion {} {} {}", v, j, jump_down));
                return (v, j, jump_down);
            }
        }
    }

    debug.log(format_args!("No escape {} {} {}", velocity, jump, jump_down));

    (0.0, false, false)
}

fn safe_move(game: &Game, unit: &Unit, velocity: f64, jump: bool, debug: &mut DrawDebug) -> bool {
    let simulation_depth_in_ticks = 100usize;
    let updates_per_tick = 10usize;
    let alpha_step = 0.45 / (simulation_depth_in_ticks as f32);
    let enemy_bullets: Vec<&Bullet> = game.bullets.iter().filter(|b| b.unit_id != unit.id).collect();
    let mut stopped_bullets = std::collections::HashSet::new();

    for tick_no in 0..simulation_depth_in_ticks {
        for microtick_no in 0..updates_per_tick {
            let time_advance = (tick_no as f64) + (microtick_no as f64 * 1.0 / updates_per_tick as f64);

            let simulated_unit = unit_state_next_tick(game, unit, velocity, jump, time_advance as f64);

            for (bullet_idx, eb) in enemy_bullets.iter().enumerate() {
                if stopped_bullets.contains(&bullet_idx) {
                    continue;
                }
                let simulated_bullet = bullet_state_next_tick(game, eb, time_advance as f64);
                if microtick_no == 0 {
//                    debug.bbox(simulated_unit.bounding_box(), COLOR_GREEN50.alpha(0.5 - alpha_step * (time_advance as f32)));
                    debug.bbox(simulated_bullet.bounding_box(), COLOR_RED50.alpha(0.5 - alpha_step * (time_advance as f32)));
                }

                if simulated_unit.bounding_box().intersects(&simulated_bullet.bounding_box()) {
                    return false;
                }

                if bullet_hits_wall(game, &simulated_bullet) {
                    if simulated_unit.bounding_box().intersects(&simulated_bullet.explosion_bounding_box()) {
                        return false;
                    }
                    stopped_bullets.insert(bullet_idx);
                }
            }
        }
    }

    return true;
}

fn bullet_hits_wall(game: &Game, bullet: &Bullet) -> bool {
    if bullet.position.x < 0.0 ||
        bullet.position.y < 0.0 ||
        bullet.position.x as usize >= game.level.tiles.len() ||
        bullet.position.y as usize >= game.level.tiles[0].len() {
        return true;
    }
    Tile::Wall == game.level.tiles[bullet.position.x as usize][bullet.position.y as usize]
}

fn bullet_hits_unit(game: &Game, bullet: &Bullet) -> bool {
    game.units.iter()
        .filter(|u| u.id != bullet.unit_id)
        .any(|u| u.bounding_box().intersects(&bullet.bounding_box()))
}

// TODO: intersection of a line with another line could be done simpler
fn is_line_of_fire(game: &Game, unit: &Unit, enemy: &Unit) -> bool {
    let Vec2F64 { x: x0, y: y0 } = unit.center(); // account for rocket launcher's larger projectiles
    let Vec2F64 { x: x1, y: y1 } = enemy.center();

    let num_points = 100.0;
    let dx = (x1 - x0) / num_points;
    let dy = (y1 - y0) / num_points;

    let mut x2 = x0;
    let mut y2 = y0;
    while (x2 - x1).abs() > 1.0 {
        let cell = game.level.cell_at_f64(x2, y2);
        if Tile::Empty != cell {
            return false;
        }
        if friendly_is_at(game, unit, x2, y2) {
            return false;
        }
        y2 += dy;
        x2 += dx;
    }
    true
}

fn friendly_is_at(game: &Game, firing_unit: &Unit, x: f64, y: f64) -> bool {
    game.units
        .iter()
        .filter(|u| u.player_id == firing_unit.player_id)
        .filter(|u| u.id != firing_unit.id)
        .any(|unit| unit.bounding_box().has_point(x, y))
}

fn chance_to_hit(unit: &Unit, enemy: &Unit, debug: &mut crate::DrawDebug) -> f32 {
    if unit.weapon.is_none() || unit.weapon.as_ref().unwrap().last_angle.is_none() {
        return 0.0;
    }
    let weapon = &unit.weapon.as_ref().unwrap();
    let spread = weapon.spread;

    let bullet_origin = unit.position.clone() + Vec2F64 { x: 0.0, y: 1.0 };
    let enemy_top = enemy.position.clone() + Vec2F64 { x: -0.5, y: 2.0 };
    let enemy_bottom = enemy.position.clone() + Vec2F64 { x: 0.5, y: 0.0 };

    let origin_to_top = enemy_top - bullet_origin;
    let origin_to_bottom = enemy_bottom - bullet_origin;

    let needed_spread = origin_to_top.angle_with(&origin_to_bottom);
    let actual_spread = spread * 2.0;

    debug.line(bullet_origin, enemy_top, DEFAULT_LINE_WIDTH, COLOR_BLUE50);
    debug.line(
        bullet_origin,
        enemy_bottom,
        DEFAULT_LINE_WIDTH,
        COLOR_BLUE50,
    );

//    println!(
//        "spread: needed {}, actual: {}",
//        needed_spread, actual_spread
//    );

    (needed_spread / actual_spread) as f32
}

fn should_shoot(game: &Game, unit: &Unit, enemy: &Unit, aim: &Vec2F64, debug: &mut crate::DrawDebug) -> bool {
    let line_of_fire = is_line_of_fire(&game, &unit, &enemy);
    let color = if line_of_fire {
        COLOR_GREEN50
    } else {
        COLOR_RED50
    };

    debug.line(unit.center(), enemy.center(), DEFAULT_LINE_WIDTH, color);
    let spread_acceptable = true;
//        let spread_acceptable = chance_to_hit(unit, enemy, debug) >= 0.6;
    let damage_score = calculate_self_harm_score(game, unit, enemy, aim, debug);
    let self_harm_acceptable = damage_score >= 0;

    line_of_fire && spread_acceptable && self_harm_acceptable
}

// Given unit's position, weapon and spread/angle, what is the expected self-damage?
fn calculate_self_harm_score(game: &Game, unit: &Unit, enemy: &Unit, aim: &Vec2F64, debug: &mut crate::DrawDebug) -> i32 {
    match &unit.weapon {
        None => 0,
        Some(weapon) if weapon.typ == WeaponType::RocketLauncher => {
            let angle = weapon.last_angle.unwrap_or(0.0);
            let orig_velocity = aim;

            let spread_steps = 10;
            let mut score = 0;
            for sstep in -spread_steps..=spread_steps {
                let angle_step = weapon.spread / spread_steps as f64;
                let delta_angle: f64 = angle_step * sstep as f64;

                let bullet = Bullet {
                    weapon_type: weapon.typ,
                    unit_id: unit.id,
                    player_id: unit.player_id,
                    position: unit.center().clone(),
                    velocity: orig_velocity.rotate(delta_angle),
                    damage: weapon.params.bullet.damage,
                    size: weapon.params.bullet.size,
                    explosion_params: weapon.params.explosion,
                };
                let blast: BoundingBox = trace_bullet(game, bullet.clone());
//                debug.bbox(blast.clone(), COLOR_YELLOW50.alpha(0.1));

                // damage to my units
                game.units.iter().
                    filter(|u| u.player_id == unit.player_id).
                    for_each(|u| {
                        let dmg = bullet.explosion_damage();

                        if u.bounding_box().intersects(&blast) {
                            score -= if dmg > u.health { game.properties.kill_score } else { dmg };
                        }
                    });

                // damage to enemies
                game.units.iter().
                    filter(|u| u.player_id != unit.player_id).
                    for_each(|u| {
                        let dmg = bullet.explosion_damage();

                        if u.bounding_box().intersects(&blast) {
                            score += if dmg > u.health { game.properties.kill_score } else { dmg };
                        }
                    });
            }

            score
        }
        _ => 0,
    }
}

// simulate rocket until it hits a wall. Return resulting explosion.
fn trace_bullet(game: &Game, bullet: Bullet) -> BoundingBox {
    let mut bullet = bullet.clone();
    loop {
        if bullet_hits_wall(game, &bullet) || bullet_hits_unit(game, &bullet) {
            return bullet.explosion_bounding_box();
        }
        bullet.position.x += bullet.velocity.x / game.properties.ticks_per_second;
        bullet.position.y -= bullet.velocity.y / game.properties.ticks_per_second;
    }
}

fn should_jump(target_pos: &Vec2F64, target_type: TargetType, unit: &Unit, game: &Game) -> bool {
    if target_pos.y > unit.position.y {
        return true;
    }
    if let TargetType::HealthPack = target_type {
        return true;
    }

    if target_pos.x > unit.position.x
        && game.level.tiles[unit.position.x as usize + 1][unit.position.y as usize] == Tile::Wall
    {
        return true;
    }

    if target_pos.x < unit.position.x
        && game.level.tiles[unit.position.x as usize - 1][unit.position.y as usize] == Tile::Wall
    {
        return true;
    }

    false
}

fn has_enemy_between(game: &Game, unit: &Unit, loot: &LootBox, debug: &mut DrawDebug) -> bool {
    let path = find_path(game, unit, loot.position).unwrap_or(vec![]);
    draw_path(&path, debug);

    game.units
        .iter()
        .filter(|other| other.player_id != unit.player_id)
        .any(|enemy| {
            path.iter().any(|wp| {
                wp.x.eq_f64(enemy.position.x) &&
                    (wp.y.eq_f64(enemy.position.y) || wp.y.eq_f64(enemy.position.y + 1.0))
            })
        })
}

fn highlight_target(bounding_box: Option<BoundingBox>, debug: &mut DrawDebug) {
    if let Some(bbox) = bounding_box {
        debug.bbox(bbox, COLOR_GREEN50);
    }
}

fn draw_grid(game: &Game, debug: &mut DrawDebug) {
    let max_i = game.level.tiles.len();
    let max_j = game.level.tiles[0].len();
    let color = COLOR_YELLOW50.alpha(0.01);
    let width = 0.05f32;

    for i in 0..max_i {
        for j in 0..max_j {
            let x = i as f32;
            let y = j as f32;

            debug.line(Vec2F32 { x: 0.0, y }, Vec2F32 { x: max_i as f32, y }, width, color.clone());
            debug.line(Vec2F32 { x, y: 0.0 }, Vec2F32 { x, y: max_j as f32 }, width, color.clone());
        }
    }
}

fn draw_path(tiles: &Vec<Waypoint>, debug: &mut DrawDebug) {
    let color = COLOR_GREEN50.alpha(0.5);
    let size = 0.1f32;

    for wp in tiles {
        debug.rect(
            Vec2F32 { x: wp.x.to_f32() + 0.5 - size, y: wp.y.to_f32() + 0.5 - size },
            Vec2F32 { x: size * 2.0, y: size * 2.0 },
            color,
        );
    }

    for wp_pair in tiles.windows(2) {
        let wp = &wp_pair[0];
        let wp2 = &wp_pair[1];
        debug.line(
            Vec2F32 { x: wp.x.to_f32() + 0.5, y: wp.y.to_f32() + 0.5 },
            Vec2F32 { x: wp2.x.to_f32() + 0.5, y: wp2.y.to_f32() + 0.5 },
            0.1,
            color,
        );
    }
}

fn bench_pathfinding(game: &Game, unit: &Unit) {
    println!("Measuring path findings per 20ms");

    use std::time::Instant;

    let limit = std::time::Duration::from_millis(20);

    let start = Instant::now();
    let mut finished = 0;
    for x in 0..1000 {
        let mut should_break = false;
        for loot_box in &game.loot_boxes {
            let goal = Pos::from_vec64(&loot_box.position);
            let result = astar(
                &Waypoint {
                    x: WpCoord::from_f64(unit.position.x),
                    y: WpCoord::from_f64(unit.position.y),
                    can_jump: unit.jump_state.can_jump,
                    jump_time_msec: (unit.jump_state.max_time * 1000.0) as i32,
                    jump_speed: unit.jump_state.speed as i32,
                    can_cancel: unit.jump_state.can_cancel,
                },
                |p| p.successors(game, unit),
                |p| p.distance(&goal) / 2,
                |p| p.x == goal.0 && p.y == goal.1,
            );
            finished += 1;
            if start.elapsed() >= limit {
                should_break = true;
                break;
            }
        }
        if should_break {
            break;
        }
    }
    println!("Done {}, elapsed {:?}", finished, start.elapsed());
}

fn find_path(game: &Game, unit: &Unit, pos: Vec2F64) -> Option<Vec<Waypoint>> {
    let goal = Pos::from_vec64(&pos);

    let result = astar(
        &Waypoint {
            x: WpCoord::from_f64(unit.position.x),
            y: WpCoord::from_f64(unit.position.y.ceil()),
            can_jump: unit.jump_state.can_jump,
            jump_time_msec: (unit.jump_state.max_time * 1000.0) as i32,
            jump_speed: unit.jump_state.speed as i32,
            can_cancel: unit.jump_state.can_cancel,
        },
        |p| p.successors(game, unit),
        |p| p.distance(&goal) / 2,
        |p| p.x == goal.0 && p.y == goal.1,
    );

    match result {
        Some((path, length)) => {
            let bbox = unit.bounding_box();
            let no_unit_origin_points = path.into_iter().filter(|wp| {
                !bbox.has_point(wp.x.to_f64(), wp.y.to_f64())
            }).collect();
            Some(no_unit_origin_points)
        },
        _ => None
    }
}

fn next_to_enemy(game: &Game, unit: &Unit) -> bool {
    game.units.iter().filter(|u| {
        unit.player_id != u.player_id
    }).any(|enemy| {
        enemy.position.y as usize == unit.position.y as usize &&
            absdiff(enemy.position.x, unit.position.x) < 1.5
    })
}