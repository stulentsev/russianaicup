use super::DebugInterface;
use crate::influence::Influence;
use crate::occupancy::OccupancyTracker;
use crate::pathfinding::{astar, bfs};
use crate::vis::*;
use crate::GameStrategy;
use model::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use EntityType::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash, trans::Trans)]
pub struct BuildQueueItem {
    pub entity_type: EntityType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitOrder {
    None,
    MineResources,
    RepairBuilding {
        building_id: i32,
    },
    HealUnit {
        unit_id: i32,
    },
    MoveAndBuild {
        entity_type: EntityType,
        move_pos: Vec2I32,
        build_pos: Vec2I32,
        start_tick: i32,
    },
    Attack {
        enemy_id: i32,
    },
    FollowPath {
        path: Rc<Vec<Vec2I32>>,
        current_idx: usize,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum RaicRound {
    Round1,
    Round2,
    Finals,
}

impl Default for RaicRound {
    fn default() -> Self {
        Self::Round1 {}
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum UnitType {
    Builder,
    Fighter,
}

#[derive(Default)]
pub struct MyStrategy {
    my_id: i32,
    map_size: i32,
    raic_round: RaicRound,
    me: Player,
    score: i32,
    resource: i32,
    current_tick: i32,
    player_view: PlayerView,
    entity_properties: HashMap<EntityType, EntityProperties>,
    use_melee: bool,

    prioritize_fighters: bool,
    end_game_flag: bool,

    population_use: i32,
    population_provide: i32,

    // convenience lists
    my_buildings: Vec<Entity>,
    my_units: Vec<Entity>,
    my_builders: Vec<Entity>,
    my_archers: Vec<Entity>,
    my_melee: Vec<Entity>,
    my_fighters: Vec<Entity>,
    my_turrets: Vec<Entity>,

    enemy_units: Vec<Entity>,
    enemy_builders: Vec<Entity>,

    entity_dict: HashMap<i32, Entity>,

    unit_orders: HashMap<i32, UnitOrder>,

    occupancy_tracker: OccupancyTracker,

    action: Action,
    last_known_actions: HashMap<i32, EntityAction>,

    fire_focus: HashMap<i32, i32>,

    influence_map: Influence,
}

impl GameStrategy for MyStrategy {
    fn get_action(
        &mut self,
        player_view: &PlayerView,
        debug_interface: Option<&mut DebugInterface>,
    ) -> Action {
        self.action = Action {
            entity_actions: HashMap::new(),
        };
        if player_view.current_tick == 0 {
            self.init_world(player_view);
            self.use_melee = self.raic_round == RaicRound::Round1;
        }

        self.init_tick(player_view);

        if !self.end_game_flag && self.is_end_game() {
            self.end_game_flag = true
        }

        self.occupancy_tracker
            .set_current(self.calculate_cell_occupancy(player_view));

        self.forget_dead_entities(&player_view.entities);
        self.rebuild_indexes(&player_view.entities); // building_repairers, etc.
        self.cancel_stale_orders();
        self.mine_adjacent_resources();

        self.prioritize_fighters = self.is_enemy_close_to_base();
        if self.prioritize_fighters {
            self.cancel_non_fighter_build_orders();
        }

        self.send_workers_to_repair();
        self.send_workers_to_mines();
        self.send_workers_to_heal_archers();
        self.send_workers_to_safe_place();
        self.produce_units();
        if !self.prioritize_fighters {
            self.build_houses();
            self.build_barracks();
            self.build_turrets();
        }
        self.activate_turrets();
        self.focus_fire(); // need to tweak this to make archers run from melee
        self.send_fighters();

        // println!("unit orders: {:?}", self.unit_orders);

        if let Some(debug_interface) = debug_interface {
            debug_interface.send(DebugCommand::Clear {});
            debug_interface.send(DebugCommand::SetAutoFlush { enable: false });

            visualize_attacks(&self.entity_dict, &self.unit_orders, debug_interface);

            debug_interface.flush_vertice_buffers();
            debug_interface.send(DebugCommand::Flush {});
        }

        self.archive_entity_actions();
        self.action.clone()
    }

    fn debug_update(&mut self, player_view: &PlayerView, debug_interface: &mut DebugInterface) {
        debug_interface.send(DebugCommand::Clear {});

        debug_interface.send(DebugCommand::SetAutoFlush { enable: false });

        let state = debug_interface.get_state();

        debug_interface.log_text(format!(
            "cell ({},{})",
            state.mouse_pos_world.x as i32, state.mouse_pos_world.y as i32
        ));
        if !self.influence_map.influence_map.is_empty() {
            debug_interface.log_text(format!(
                "my influence {}",
                self.influence_map.my_influence_at(&Vec2I32::from(state.mouse_pos_world))
            ));
            debug_interface.log_text(format!(
                "enemy influence {}",
                self.influence_map.enemy_influence_at(&Vec2I32::from(state.mouse_pos_world))
            ));
            debug_interface.log_text(format!(
                "turret attack {}",
                self.influence_map.is_turret_attack_at(&Vec2I32::from(state.mouse_pos_world))
            ));
        }

        // if !state.pressed_keys.is_empty() {
        //     println!("{:?}", state.pressed_keys);
        // }
        if state.pressed_keys.contains(&"Num6".to_string()) {
            display_turret_fields(player_view, debug_interface);
        }
        if state.pressed_keys.contains(&"Num7".to_string()) {
            display_builder_fields(player_view, debug_interface);
        }
        if state.pressed_keys.contains(&"Num8".to_string()) {
            display_all_fields(player_view, debug_interface);
        }

        if state.pressed_keys.contains(&"LCtrl".to_string()) {
            self.display_unit_info(state.mouse_pos_world, player_view, debug_interface);
        }

        if state.pressed_keys.contains(&"H".to_string()) {
            visualize_influence_map_one_color(
                &self.influence_map.my_influence,
                100,
                debug_interface,
            );
        }

        if state.pressed_keys.contains(&"J".to_string()) {
            visualize_influence_map_one_color(
                &self.influence_map.enemy_influence,
                0,
                debug_interface,
            );
        }

        if state.pressed_keys.contains(&"K".to_string()) {
            visualize_influence_map(&self.influence_map.influence_map, 100, debug_interface);
        }

        if state.pressed_keys.contains(&"Y".to_string()) {
            visualize_influence_map(&self.influence_map.tension_map, 100, debug_interface);
        }

        if state.pressed_keys.contains(&"U".to_string()) {
            visualize_influence_map(&self.influence_map.vulnerability_map, 100, debug_interface);
        }

        debug_interface.flush_vertice_buffers();
        debug_interface.send(DebugCommand::Flush {});
    }
}

impl MyStrategy {
    pub fn new() -> Self {
        Default::default()
    }

    fn forget_dead_entities(&mut self, current_entities: &[Entity]) {
        let mut new_unit_orders = HashMap::new();
        for entity in current_entities.iter() {
            if self.unit_orders.contains_key(&entity.id) {
                if let Some(order) = self.unit_orders.get(&entity.id) {
                    if *order != UnitOrder::None {
                        new_unit_orders.insert(entity.id, order.clone());
                    }
                } else {
                    println!("found new unit {}", entity.id);
                }
            }
        }
        self.unit_orders = new_unit_orders;
    }

    fn cancel_stale_orders(&mut self) {
        let mut new_unit_orders = HashMap::new();
        for (unit_id, unit_order) in self.unit_orders.iter() {
            if let Some(unit) = self.entity_dict.get(unit_id) {
                match unit_order {
                    UnitOrder::MoveAndBuild {
                        build_pos,
                        start_tick,
                        ..
                    } => {
                        if let Some(building) =
                        self.my_buildings.iter().find(|b| b.position == *build_pos)
                        {
                            // building was built, repair it
                            new_unit_orders.insert(
                                *unit_id,
                                UnitOrder::RepairBuilding {
                                    building_id: building.id,
                                },
                            );
                            self.action
                                .entity_actions
                                .insert(*unit_id, self.make_repair_action(building.id));
                        } else if self.player_view.current_tick > *start_tick + 10 {
                            // building wasn't built, forget this order
                        } else {
                            new_unit_orders.insert(*unit_id, unit_order.clone());
                        }
                    }
                    UnitOrder::RepairBuilding { building_id } => {
                        if let Some(building) = self.entity_dict.get(building_id) {
                            if self.has_adjacent_entity(unit, *building_id)
                                && building.health
                                < self.entity_properties[&building.entity_type].max_health
                            {
                                new_unit_orders.insert(*unit_id, unit_order.clone());
                            }
                        }
                    }
                    UnitOrder::None | UnitOrder::Attack { .. } | UnitOrder::HealUnit {..} => {}
                    UnitOrder::FollowPath { path, current_idx } => {
                        if *current_idx < path.len() - 1 - 1 {
                            // don't go to the last cell of the path
                            let new_idx = current_idx + 1;
                            new_unit_orders.insert(
                                *unit_id,
                                UnitOrder::FollowPath {
                                    path: path.clone(),
                                    current_idx: new_idx,
                                },
                            );
                            let next_cell = path.get(new_idx).unwrap();
                            // println!("advancing unit {} to cell {}", unit_id, next_cell);
                            self.action
                                .entity_actions
                                .insert(*unit_id, self.make_move_to_position_exact(*next_cell));
                        }
                    }
                    UnitOrder::MineResources => {
                        // if self.has_adjacent_resources(unit) {
                        //     new_unit_orders.insert(*unit_id, unit_order.clone());
                        // }
                    }
                }
            }
        }

        self.unit_orders = new_unit_orders;
    }

    fn archive_entity_actions(&mut self) {
        for (unit_id, entity_action) in self.action.entity_actions.iter() {
            self.last_known_actions
                .insert(*unit_id, entity_action.clone());
        }
    }

    fn mine_adjacent_resources(&mut self) {
        for builder in self.my_builders.iter() {
            if self.has_adjacent_resources(builder) {
                let attack_resources = AttackAction {
                    target: None,
                    auto_attack: Some(AutoAttack {
                        pathfind_range: 10,
                        valid_targets: vec![Resource, BuilderUnit],
                    }),
                };
                self.action.entity_actions.insert(
                    builder.id,
                    self.make_action(Some(attack_resources), None, None, None),
                );
                self.unit_orders
                    .insert(builder.id, UnitOrder::MineResources);
            }
        }
    }

    fn has_adjacent_resources(&self, builder: &Entity) -> bool {
        for cell in builder.move_range_cells() {
            if let Some(entity_id) = self.occupancy_tracker.get(cell.x, cell.y) {
                if let Some(Entity {
                                entity_type: Resource,
                                ..
                            }) = self.entity_dict.get(&entity_id)
                {
                    return true;
                }
            }
        }
        false
    }

    fn has_adjacent_entity(&self, builder: &Entity, target_entity_id: i32) -> bool {
        for cell in builder.move_range_cells() {
            if let Some(entity_id) = self.occupancy_tracker.get(cell.x, cell.y) {
                if entity_id == target_entity_id {
                    return true;
                }
            }
        }
        false
    }

    fn rebuild_indexes(&mut self, entities: &[Entity]) {
        self.my_buildings.clear();
        self.my_units.clear();
        self.my_builders.clear();
        self.my_archers.clear();
        self.my_melee.clear();
        self.my_fighters.clear();
        self.my_turrets.clear();

        self.enemy_units.clear();
        self.enemy_builders.clear();

        self.entity_dict.clear();

        self.population_use = 0;
        self.population_provide = 0;

        let my_id = self.my_id;

        for entity in entities.iter() {
            self.entity_dict.insert(entity.id, *entity);
        }

        for entity in entities.iter().filter(|e| e.player_id == Some(my_id)) {
            let properties = &self.entity_properties[&entity.entity_type];

            self.population_use += properties.population_use;
            if entity.active {
                self.population_provide += properties.population_provide;
            }

            match entity.entity_type {
                Turret => {
                    self.my_turrets.push(*entity);
                    self.my_buildings.push(*entity);
                }
                House | Wall | MeleeBase | RangedBase | BuilderBase => {
                    self.my_buildings.push(*entity);
                }
                BuilderUnit => {
                    self.my_builders.push(*entity);
                    self.my_units.push(*entity);
                }
                MeleeUnit => {
                    self.my_melee.push(*entity);
                    self.my_units.push(*entity);
                    self.my_fighters.push(*entity);
                }
                RangedUnit => {
                    self.my_archers.push(*entity);
                    self.my_units.push(*entity);
                    self.my_fighters.push(*entity);
                }
                _ => {}
            }
        }

        for entity in entities.iter().filter(|e| e.player_id != Some(my_id)) {
            match entity.entity_type {
                BuilderUnit => {
                    self.enemy_builders.push(*entity);
                }
                MeleeUnit | RangedUnit => {
                    self.enemy_units.push(*entity);
                }
                _ => {}
            }
        }

        self.influence_map.recalculate(&self.player_view);
    }

    fn send_workers_to_repair(&mut self) {
        // for each building
        // if hp is less than max
        // find closest N builders
        // set attack to None, move to center of the building and repair building id
        for building in self.my_buildings.iter() {
            let entity_properties = &self.entity_properties[&building.entity_type];
            if building.health < entity_properties.max_health {
                let active_repairers = self
                    .unit_orders
                    .values()
                    .filter(|order| matches!(order, UnitOrder::RepairBuilding { building_id } if *building_id == building.id))
                    .count();
                let need_repairers =
                    building.number_of_repairers() as i32 - active_repairers as i32;
                if need_repairers <= 0 {
                    continue;
                }
                // println!("building {} needs {} units to repair (have {} / {})", building.id, need_repairers, active_repairers, building.number_of_repairers());
                let center_pos = building.center_pos();

                let attack_action = None;
                let build_action = None;
                let move_action = Some(MoveAction {
                    target: center_pos,
                    find_closest_position: true,
                    break_through: true,
                });
                let repair_action = Some(RepairAction {
                    target: building.id,
                });
                let entity_action = EntityAction {
                    move_action,
                    build_action,
                    attack_action,
                    repair_action,
                };

                // let mut builders = self.player_view.my_builders();
                let available_builders: Vec<&Entity> = self
                    .my_builders
                    .iter()
                    .filter(|b| {
                        !matches!(
                            self.unit_orders.get(&b.id),
                            // None | Some(UnitOrder::MineResources)
                            Some(UnitOrder::RepairBuilding { .. })
                        )
                    })
                    .filter(|builder| {
                        center_pos.mdist(&builder.position) < building.repair_call_radius()
                    })
                    .collect();
                for b in available_builders.iter().take(need_repairers as usize) {
                    // println!("sending {} to repair {}", b.id, building_id);
                    self.action
                        .entity_actions
                        .insert(b.id, entity_action.clone());
                    // self.unit_orders.entry(b.id).
                    //     and_modify(|order| *order = UnitOrder::RepairBuilding { building_id: *building_id }).
                    //     or_insert(UnitOrder::RepairBuilding { building_id: *building_id });
                    self.unit_orders.insert(
                        b.id,
                        UnitOrder::RepairBuilding {
                            building_id: building.id,
                        },
                    );
                }
            }
        }

        // TODO: take note of who is repairing what
    }

    fn send_workers_to_mines(&mut self) {
        for builder in self.my_builders.iter() {
            if self.unit_orders.contains_key(&builder.id) {
                continue;
            }
            // println!("sending {} to mines", builder_id);
            let (entity_action, unit_order) = self.make_send_to_mines_action(builder);
            self.action.entity_actions.insert(builder.id, entity_action);
            self.unit_orders.insert(builder.id, unit_order);
        }
    }

    fn send_workers_to_heal_archers(&mut self) {
        for builder in self.my_builders.iter() {
            for cell in builder.move_range_cells() {
                if let Some(unit_id) = self.occupancy_tracker.get(cell.x, cell.y) {
                    match self.entity_dict.get(&unit_id) {
                        Some(entity) if entity.entity_type == RangedUnit && entity.health == 5 => {
                            println!("builder {} is healing archer {}", builder.id, entity.id);
                            self.unit_orders.insert(builder.id, UnitOrder::HealUnit {unit_id: entity.id});
                            self.action.entity_actions.insert(builder.id, self.make_repair_action(entity.id));
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    fn send_workers_to_safe_place(&mut self) {
        for builder in self.my_builders.iter() {
            let should_retreat = self.influence_map.enemy_influence_at(&builder.position) > 3 * 10;
            // let should_retreat = self.enemy_units.iter().any(|e| {
            //     is_fighter_unit(e) && e.position.mdist(&builder.position) <= 7
            // } );
            if should_retreat {
                let safe_loc = builder
                    .move_range_cells()
                    .into_iter()
                    .filter(|cell| self.can_move_to(builder, cell))
                    .min_by_key(|cell| {
                        (
                            self.influence_map.enemy_influence_at(cell),
                            cell.mdist(&builder.position),
                        )
                    })
                    .unwrap_or_else(Vec2I32::origin);
                // println!("moving builder {} to ({},{})", builder.id, safe_loc.x, safe_loc.y);
                self.unit_orders.remove(&builder.id);
                self.action
                    .entity_actions
                    .insert(builder.id, self.make_move_to_position_exact(safe_loc));
            }
        }
    }

    fn produce_units(&mut self) {
        for building in self.my_buildings.iter() {
            if !self.prioritize_fighters && matches!(building.entity_type, BuilderBase) {
                self.action
                    .entity_actions
                    .insert(building.id, self.make_builder_action(building));
            }
            if (self.prioritize_fighters || self.use_melee) && matches!(building.entity_type, MeleeBase) {
                self.action
                    .entity_actions
                    .insert(building.id, self.make_fighter_action(building));
            }
            if matches!(building.entity_type, RangedBase) {
                self.action
                    .entity_actions
                    .insert(building.id, self.make_fighter_action(building));
            }
        }
    }

    fn cancel_non_fighter_build_orders(&mut self) {
        for building in self.my_buildings.iter() {
            if matches!(building.entity_type, BuilderBase) {
                self.action
                    .entity_actions
                    .insert(building.id, self.make_empty_action());
            }
        }
    }

    fn make_fighter_action(&self, building: &Entity) -> EntityAction {
        let building_properties = &self.entity_properties[&building.entity_type];
        let buildable_type = building_properties.build.as_ref().unwrap().options[0];
        let threshold = if self.end_game_flag { 200 } else { 199 };

        let build_action = if self.current_cost(buildable_type) < threshold {
            if let Some(position) = self.find_place_to_build_unit(building, UnitType::Fighter) {
                Some(BuildAction {
                    entity_type: buildable_type,
                    position,
                })
            } else {
                None
            }
        } else {
            None
        };

        self.make_action(None, build_action, None, None)
    }

    fn current_cost(&self, entity_type: EntityType) -> i32 {
        let unit_properties = &self.entity_properties[&entity_type];
        let added_cost = self
            .my_units
            .iter()
            .filter(|e| e.entity_type == entity_type)
            .count() as i32;

        unit_properties.initial_cost + added_cost
    }

    fn make_repair_action(&self, building_id: i32) -> EntityAction {
        self.make_action(
            None,
            None,
            Some(RepairAction {
                target: building_id,
            }),
            None,
        )
    }

    fn make_builder_action(&self, building: &Entity) -> EntityAction {
        let building_properties = &self.entity_properties[&building.entity_type];
        let buildable_type = building_properties.build.as_ref().unwrap().options[0];

        let threshold = if self.end_game_flag { 100 } else { 70 };

        let build_action = if self.current_cost(buildable_type) < threshold {
            if let Some(position) = self.find_place_to_build_unit(building, UnitType::Builder) {
                Some(BuildAction {
                    entity_type: buildable_type,
                    position,
                })
            } else {
                None
            }
        } else {
            None
        };

        self.make_action(None, build_action, None, None)
    }

    fn need_population(&self) -> i32 {
        if self.end_game_flag {
            300
        } else {
            150
        }
    }

    // if only one enemy has barracks
    fn is_end_game(&self) -> bool {
        if matches!(self.raic_round, RaicRound::Round2 | RaicRound::Finals if self.current_tick < 400)
        {
            return false;
        }

        let mut first_found_enemy: Option<i32> = None;
        for entity in self
            .player_view
            .entities
            .iter()
            .filter(|e| e.player_id != Some(self.my_id) && is_barrack(e))
        {
            match first_found_enemy {
                Some(player_id) => {
                    if player_id != entity.player_id.unwrap() {
                        return false;
                    }
                }
                None => first_found_enemy = Some(entity.player_id.unwrap()),
            }
        }
        true
    }

    fn build_houses(&mut self) {
        let entity_type = House;
        let house_properties = &self.entity_properties[&entity_type];
        if self.me.resource < house_properties.initial_cost {
            return;
        }

        if self.population_provide >= self.need_population() {
            // println!("population provide {}, not building house", population_provide);
            return;
        } else {
            // we haven't reached max population, but maybe we need barracks?
            if self.need_barracks() && self.count_buildings(House) >= 3 {
                return;
            }
        }

        self.schedule_order_for_building(entity_type);
    }

    fn need_barracks(&self) -> bool {
        self.need_building(RangedBase)
            || self.need_building(MeleeBase)
            || self.need_building(BuilderBase)
    }

    fn schedule_order_for_building(&mut self, building_type: EntityType) {
        if self.unit_orders.values().any(|order| matches!(order, UnitOrder::MoveAndBuild {entity_type: btype, ..} if btype == &building_type)) {
            // have active order for the house, not scheduling more
            return;
        }

        let building_properties = &self.entity_properties[&building_type];

        let vacant_patches = match building_type {
            // Turret => self.find_vacant_patches_radial(building_properties.size),
            Turret => self.find_vacant_patches_close_to_mines(building_properties.size),
            _ => self.find_vacant_patches2(building_type),
        };

        let mut final_patch: Option<(i32, Vec2I32, Vec2I32, i32)> = None;

        for patch in vacant_patches.iter() {
            if self.building_scheduled_at_this_patch(patch) {
                continue;
            }
            for builder in self.my_builders.iter() {
                if !matches!(
                    self.unit_orders.get(&builder.id),
                    None | Some(UnitOrder::MineResources) | Some(UnitOrder::FollowPath{..})
                ) {
                    continue;
                }

                if builder.position.mdist(patch) > 10 {
                    // out of sight
                    continue;
                }

                if let Some(target_move_pos) = self.find_building_adjacent_cell_closest_to_target(
                    *patch,
                    building_properties.size,
                    builder.position,
                ) {
                    let dist = target_move_pos.mdist(&builder.position);
                    if final_patch.is_none() || final_patch.unwrap().0 > dist {
                        final_patch = Some((dist, target_move_pos, *patch, builder.id));
                    }
                }
            }
        }

        if let Some((_dist, move_pos, build_pos, builder_id)) = final_patch {
            let repair_action = None;
            let attack_action = None;

            let move_action = Some(MoveAction {
                target: move_pos,
                find_closest_position: false,
                break_through: true,
            });
            let build_action = Some(BuildAction {
                entity_type: building_type,
                position: build_pos,
            });
            let entity_action = EntityAction {
                move_action,
                build_action,
                attack_action,
                repair_action,
            };
            // println!("builder {} is going from {:?} to {:?} to build structure of size {} at {:?}", builder_id, self.my_builders.get(&builder_id).unwrap().position, move_pos, house_properties.size, build_pos);
            self.action.entity_actions.insert(builder_id, entity_action);
            // self.unit_orders.remove(&builder_id);
            self.unit_orders.insert(
                builder_id,
                UnitOrder::MoveAndBuild {
                    move_pos,
                    build_pos,
                    start_tick: self.player_view.current_tick,
                    entity_type: building_type,
                },
            );
        }
    }

    fn building_scheduled_at_this_patch(&self, patch: &Vec2I32) -> bool {
        self.unit_orders.iter().any(|(_, unit_order)| {
            matches!(unit_order, UnitOrder::MoveAndBuild {build_pos, ..} if build_pos == patch)
        })
    }

    fn build_barracks(&mut self) {
        self.build_barrack(RangedBase);
        self.build_barrack(BuilderBase);

        if self.need_building(RangedBase) && self.use_melee {
            self.build_barrack(MeleeBase);
        }
    }

    fn build_barrack(&mut self, entity_type: EntityType) {
        let properties = &self.entity_properties[&entity_type];
        if self.me.resource < properties.initial_cost {
            // println!("got {} / {} for a {:?}", self.me.resource, properties.initial_cost, entity_type);
            return;
        }

        if !self.need_building(entity_type) {
            return; // already exists
        }
        self.schedule_order_for_building(entity_type);
    }

    fn build_turrets(&mut self) {
        let entity_type = Turret;
        let turret_properties = &self.entity_properties[&entity_type];
        if self.me.resource < turret_properties.initial_cost {
            return;
        }

        // if (self.my_archers.len() as i32) < turret_properties.initial_cost - 30 {
        //     return;
        // }

        if self.need_barracks() {
            return;
        }

        let builder_count_threshold = match self.raic_round {
            RaicRound::Round1 => 30,
            RaicRound::Round2 => 30,
            RaicRound::Finals => 50,
        };

        let count_cap = match self.raic_round {
            RaicRound::Round1 => 50,
            RaicRound::Round2 => 50,
            RaicRound::Finals => 100,
        };

        if self.my_builders.len() < builder_count_threshold {
            // println!("population use {}, not building turret", population_use);
            return;
        }

        if self.count_buildings(entity_type) > count_cap {
            return;
        }

        self.schedule_order_for_building(entity_type);
    }

    fn activate_turrets(&mut self) {
        let sight_range = self.entity_properties[&Turret].sight_range;
        for turret in self.my_turrets.iter() {
            let attack_action = AttackAction {
                target: None,
                auto_attack: Some(AutoAttack {
                    pathfind_range: sight_range,
                    valid_targets: vec![],
                }),
            };
            self.action.entity_actions.insert(
                turret.id,
                self.make_action(Some(attack_action), None, None, None),
            );
        }
    }

    fn focus_fire(&mut self) {
        let my_id = self.my_id;
        let mut unit_id_to_hittable_enemies: HashMap<i32, Vec<i32>> = HashMap::new();
        let mut enemy_id_to_attackers: HashMap<i32, Vec<i32>> = HashMap::new();
        let mut used_units: HashSet<i32> = HashSet::new();

        for entity in self
            .player_view
            .entities
            .iter()
            .filter(|e| e.player_id == Some(my_id))
        {
            let properties = &self.entity_properties[&entity.entity_type];
            if let Some(attack_properties) = &properties.attack {
                let mut enemies: Vec<&Entity> = self
                    .player_view
                    .entities
                    .iter()
                    .filter(|e| {
                        e.player_id.is_some()
                            && e.player_id != Some(my_id)
                            && e.is_within_attack_range(
                            &entity.position,
                            attack_properties.attack_range,
                        )
                    })
                    .collect();
                enemies.sort_by_key(|e| e.health);
                // if !enemies.is_empty() {
                //     println!("unit {}, enemies {:?}", entity.id, enemies.iter().map(|e| e.id).collect::<Vec<i32>>());
                // }

                for enemy in enemies.iter() {
                    unit_id_to_hittable_enemies.entry(entity.id).or_insert_with(Vec::new);
                    unit_id_to_hittable_enemies
                        .get_mut(&entity.id)
                        .unwrap()
                        .push(enemy.id);

                    enemy_id_to_attackers.entry(enemy.id).or_insert_with(Vec::new);

                    enemy_id_to_attackers
                        .get_mut(&enemy.id)
                        .unwrap()
                        .push(entity.id);

                    // unit_id_to_hittable_enemies.entry(entity.id).
                    //     and_modify(|vec| vec.push(enemy.id)).
                    //     or_insert_with(|| Vec::new());
                    // enemy_id_to_attackers.entry(enemy.id).
                    //     and_modify(|vec| vec.push(entity.id)).
                    //     or_insert_with(|| Vec::new());
                }
            }
        }

        // if !unit_id_to_hittable_enemies.is_empty() {
        //     println!("hittable enemies: {:?}", unit_id_to_hittable_enemies);
        // }
        // if !enemy_id_to_attackers.is_empty() {
        //     println!("enemy_id to attackers: {:?}", enemy_id_to_attackers);
        // }

        let mut focused_fire: HashMap<i32, i32> = HashMap::new();

        let mut enemy_ids: Vec<i32> = enemy_id_to_attackers.keys().cloned().collect();
        let preference_order = |eid: &i32| -> (i32, i32, i32) {
            let enemy = self.entity_dict[eid];
            let hp = enemy.health;
            let attacker_damage: i32 = enemy_id_to_attackers[eid]
                .iter()
                .map(|unit_id| {
                    let unit = self.entity_dict.get(unit_id).unwrap();
                    self.entity_properties[&unit.entity_type]
                        .attack
                        .as_ref()
                        .unwrap()
                        .damage
                })
                .sum();
            let can_kill = if attacker_damage * 5 >= hp { 1 } else { 2 };
            let enemy_type = match enemy.entity_type {
                RangedUnit => 1,
                MeleeUnit => 2,
                _ => 3,
            };

            (can_kill, enemy_type, hp)
        };
        enemy_ids.sort_by_key(|eid| preference_order(eid));

        for enemy_id in enemy_ids.iter() {
            let attackers = enemy_id_to_attackers.get_mut(enemy_id).unwrap();
            let enemy = self.entity_dict.get(enemy_id).unwrap();

            let mut total_dmg = *focused_fire.get(enemy_id).unwrap_or(&0);
            let hp = &self.entity_dict.get(enemy_id).unwrap().health;

            if total_dmg >= *hp {
                continue;
            }
            // sort by how many it can hit
            attackers.sort_by_key(|attacker_id| {
                unit_id_to_hittable_enemies
                    .get(attacker_id)
                    .map_or(0, |v| v.len())
            });

            for attacker_id in attackers.iter() {
                if total_dmg >= *hp {
                    break;
                }
                if used_units.contains(&attacker_id) {
                    continue;
                }
                let attacker = self.entity_dict.get(attacker_id).unwrap();
                let atk = &self.entity_properties[&attacker.entity_type]
                    .attack
                    .as_ref()
                    .unwrap()
                    .damage;

                total_dmg += *atk;
                // println!("unit {} is attacking enemy {}", attacker.id, enemy_id);

                self.unit_orders.insert(
                    *attacker_id,
                    UnitOrder::Attack {
                        enemy_id: *enemy_id,
                    },
                );
                self.action
                    .entity_actions
                    .insert(*attacker_id, self.make_focused_attack_action(enemy));
                focused_fire.insert(*enemy_id, total_dmg);
                used_units.insert(*attacker_id);
            }
        }

        // if !focused_fire.is_empty() {
        //     println!("focused fire result: {:?}", focused_fire);
        // }
    }

    fn send_fighters(&mut self) {
        for unit in self.my_units.iter() {
            // don't move fighters who are currently shooting
            if matches!(
                self.unit_orders.get(&unit.id),
                Some(UnitOrder::Attack { .. })
            ) {
                continue;
            }
            if matches!(unit.entity_type, MeleeUnit | RangedUnit) {
                let entity_action = self
                    .make_emergency_defense_action(unit)
                    .or_else(|| self.make_protect_barracks_action(unit))
                    // .or_else(|| self.make_send_to_nearest_threat_action(unit))
                    .or_else(|| self.make_send_to_nearest_enemy_entity_action(unit, |e| is_unit(e)))
                    .or_else(|| self.make_send_to_nearest_enemy_entity_action(unit, |e| is_building(e)))
                    .or_else(|| self.make_send_to_random_corner_action(unit))
                    .or_else(|| self.make_send_to_collection_point_action(unit))
                    .unwrap_or_else(|| self.make_empty_action());

                if let EntityAction {
                    attack_action:
                    Some(AttackAction {
                             target: Some(enemy_id),
                             ..
                         }),
                    ..
                } = entity_action
                {
                    let dmg = unit.attack_value();
                    self.fire_focus
                        .entry(enemy_id)
                        .and_modify(|v| *v += dmg)
                        .or_insert(dmg);
                }

                self.occupancy_tracker
                    .maybe_update_next_tick(unit, &entity_action);
                self.action.entity_actions.insert(unit.id, entity_action);
            }
        }
    }

    fn make_focused_attack_action(&self, enemy: &Entity) -> EntityAction {
        let attack_action = AttackAction {
            target: Some(enemy.id),
            auto_attack: None,
        };
        let move_action = MoveAction {
            target: enemy.center_pos(),
            find_closest_position: true,
            break_through: true,
        };

        self.make_action(Some(attack_action), None, None, Some(move_action))
    }

    fn make_move_to_position_find_closest_action(&self, target_pos: Vec2I32) -> EntityAction {
        let move_action = MoveAction {
            target: target_pos,
            find_closest_position: true,
            break_through: true,
        };

        self.make_action(None, None, None, Some(move_action))
    }

    #[allow(dead_code)]
    fn make_move_to_position_using_astar(
        &self,
        start_pos: &Vec2I32,
        target_pos: &Vec2I32,
    ) -> EntityAction {
        let final_target_pos = if let Some(path) = self.find_path_to_location(start_pos, target_pos)
        {
            *path.get(1).unwrap_or(target_pos)
        } else {
            *target_pos
        };
        let move_action = MoveAction {
            target: final_target_pos,
            find_closest_position: true,
            break_through: true,
        };

        self.make_action(None, None, None, Some(move_action))
    }

    fn make_move_to_position_exact(&self, target_pos: Vec2I32) -> EntityAction {
        let move_action = MoveAction {
            target: target_pos,
            find_closest_position: false,
            break_through: true,
        };

        self.make_action(None, None, None, Some(move_action))
    }

    fn make_send_to_nearest_enemy_entity_action<Filter>(&self, unit: &Entity, filter: Filter) -> Option<EntityAction>
    where Filter: Fn(&Entity) -> bool
    {
        let enemy = self.find_closest_entity(unit, |e| {
            filter(e) && e.player_id != Some(self.my_id)
        })?;

        // let target = enemy.position;
        let target = if self.raic_round == RaicRound::Round1 {
            // open map
            enemy.position
        } else {
            let path = self.find_path_to_closest_enemy(unit, filter)?;
            *path.get(1).unwrap_or(&enemy.position)
        };

        let move_action = MoveAction {
            target,
            find_closest_position: false,
            break_through: true,
        };

        let attack_action = AttackAction {
            target: None,
            // auto_attack: Some(AutoAttack {
            //     pathfind_range: properties.sight_range,
            //     valid_targets: vec![],
            // }),
            auto_attack: None
        };

        Some(self.make_action(Some(attack_action), None, None, Some(move_action)))
    }

    fn make_emergency_defense_action(&self, unit: &Entity) -> Option<EntityAction> {
        if self.prioritize_fighters {
            self.make_send_to_nearest_enemy_entity_action(unit, |e| is_unit(e))
        } else {
            None
        }
    }

    fn make_protect_barracks_action(&self, unit: &Entity) -> Option<EntityAction> {
        if self.raic_round != RaicRound::Round1 || self.current_tick > 200 {
            return None;
        }

        let collection_points = vec![Vec2I32::from_i32(25, 7), Vec2I32::from_i32(7, 25)];

        let move_pos = collection_points[unit.id as usize % collection_points.len()];
        Some(self.make_move_to_position_find_closest_action(move_pos))
    }

    #[allow(dead_code)]
    fn make_send_to_nearest_threat_action(&self, unit: &Entity) -> Option<EntityAction> {
        let properties = &self.entity_properties[&unit.entity_type];

        let threat_loc = self.influence_map.most_threatening_enemy_presence(unit)?;

        let path = self.find_path_to_location(&unit.position, &threat_loc)?;

        let move_action = MoveAction {
            target: *path.get(1)?,
            find_closest_position: false,
            break_through: true,
        };

        let attack_action = AttackAction {
            target: None,
            auto_attack: Some(AutoAttack {
                pathfind_range: properties.sight_range,
                valid_targets: vec![],
            }),
        };

        Some(self.make_action(Some(attack_action), None, None, Some(move_action)))
    }

    #[allow(dead_code)]
    fn make_send_to_a_random_corner(&self, unit: &Entity) -> Option<EntityAction> {
        let default_target = Vec2I32::from_i32(self.map_size - 5, self.map_size - 5);

        let corner_coords = vec![
            Vec2I32::from_i32(self.map_size - 5, 5),
            Vec2I32::from_i32(5, self.map_size),
            default_target,
        ];

        let idx = unit.id as usize % corner_coords.len();
        let target = corner_coords.get(idx).unwrap_or(&default_target);

        let properties = &self.entity_properties[&unit.entity_type];

        let attack_action = AttackAction {
            target: None,
            auto_attack: Some(AutoAttack {
                pathfind_range: properties.sight_range,
                valid_targets: vec![],
            }),
        };

        let move_action = MoveAction {
            target: *target,
            find_closest_position: true,
            break_through: true,
        };
        // println!("unit {}, going to enemy unit at {:?}", unit.id, closest.position );

        Some(self.make_action(Some(attack_action), None, None, Some(move_action)))
    }

    fn make_send_to_collection_point_action(&self, unit: &Entity) -> Option<EntityAction> {
        let collection_points = vec![
            Vec2I32::from_i32(26, 14),
            Vec2I32::from_i32(14, 26),
            Vec2I32::from_i32(21, 21),
        ];

        self.make_send_to_random_location_action(unit, &collection_points)
    }

    fn make_send_to_random_corner_action(&self, unit: &Entity) -> Option<EntityAction> {
        if self.current_tick < 900 {
            return None;
        }

        let corners = vec![
            Vec2I32::from_i32(5, 75),
            Vec2I32::from_i32(75, 5),
            Vec2I32::from_i32(75, 75),
        ];
        self.make_send_to_random_location_action(unit, &corners)
    }

    fn make_send_to_random_location_action(&self, unit: &Entity, locations: &[Vec2I32]) -> Option<EntityAction> {
        let properties = &self.entity_properties[&unit.entity_type];

        let idx = unit.id as usize % locations.len();
        let target = locations
            .get(idx)
            .unwrap();

        let attack_action = AttackAction {
            target: None,
            auto_attack: Some(AutoAttack {
                pathfind_range: properties.sight_range,
                valid_targets: vec![],
            }),
        };

        let move_action = MoveAction {
            target: *target,
            find_closest_position: true,
            break_through: false,
        };
        // println!("unit {}, going to enemy unit at {:?}", unit.id, closest.position );

        Some(self.make_action(Some(attack_action), None, None, Some(move_action)))
    }

    fn make_send_to_mines_action(&self, unit: &Entity) -> (EntityAction, UnitOrder) {
        let properties = &self.entity_properties[&BuilderUnit];

        let pos: Vec2I32;
        let unit_order: UnitOrder;

        if let Some(path) = self.find_path_to_closest_resource(unit) {
            pos = *if path.len() > 1 {
                path.get(1)
            } else {
                path.get(0)
            }
                .unwrap();
            unit_order = UnitOrder::FollowPath {
                path: Rc::new(path),
                current_idx: 0,
            };
        } else {
            pos = self
                .find_closest_entity(unit, |e| e.entity_type == Resource)
                .map_or(Vec2I32::top_right_corner(), |e| e.position);

            unit_order = UnitOrder::MineResources {};
        };

        let move_action = MoveAction {
            target: pos,
            find_closest_position: true,
            break_through: true,
        };
        let attack_resources = AttackAction {
            target: None,
            auto_attack: Some(AutoAttack {
                pathfind_range: properties.sight_range,
                valid_targets: vec![Resource, BuilderUnit],
            }),
        };

        (
            self.make_action(Some(attack_resources), None, None, Some(move_action)),
            unit_order,
        )
    }

    fn find_place_to_build_unit(&self, base: &Entity, unit_type: UnitType) -> Option<Vec2I32> {
        let size = self.entity_properties[&base.entity_type].size;

        let target = match unit_type {
            UnitType::Builder => self
                .find_closest_resource(base)
                .unwrap_or_else(|| base.center_pos()),
            UnitType::Fighter => self.find_closest_enemy(base)?,
            // UnitType::Fighter => self
            //     .find_closest_enemy(base)
            //     .unwrap_or_else(|| base.center_pos()),
        };

        self.find_building_adjacent_cell_closest_to_target(base.position, size, target)
    }

    fn find_path_to_closest_enemy<Filter>(&self, origin: &Entity, filter: Filter) -> Option<Vec<Vec2I32>>
    where Filter: Fn(&Entity) -> bool {
        let start = origin.position;
        let mut seen_nodes = 0;
        let max_seen_nodes = 1500;
        bfs(
            &start,
            |a| -> Vec<Vec2I32> {
                if seen_nodes >= max_seen_nodes {
                    return vec![]
                }
                let possible_successors = vec![
                    Vec2I32::from_i32(a.x + 1, a.y),
                    Vec2I32::from_i32(a.x, a.y + 1),
                    Vec2I32::from_i32(a.x - 1, a.y),
                    Vec2I32::from_i32(a.x, a.y - 1),
                ];
                let successors: Vec<Vec2I32> = possible_successors
                    .iter()
                    .filter(|succ| {
                        if succ.x < 0
                            || succ.y < 0
                            || succ.x >= self.map_size
                            || succ.y >= self.map_size
                            || self.is_cell_in_enemy_turret_attack_range(succ)
                        {
                            false
                        } else if let Some(entity_id) = &self.occupancy_tracker.get(succ.x, succ.y)
                        {
                            let entity = self.entity_dict.get(entity_id).unwrap();
                            !self.is_attacking_enemy(entity)
                                && !(entity.player_id == Some(self.my_id) && (entity.entity_type == BuilderUnit || is_building(entity)))

                            // !is_fighter_unit(entity) // assuming our fighters always move
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect();
                seen_nodes += successors.len();
                successors
            },
            |a| {
                if let Some(entity_id) = &self.occupancy_tracker.get(a.x, a.y) {
                    let entity = self.entity_dict.get(entity_id).unwrap();
                    filter(entity) && entity.player_id != Some(self.my_id)
                } else {
                    false
                }
            },
        )
    }

    fn find_path_to_location(&self, start: &Vec2I32, target: &Vec2I32) -> Option<Vec<Vec2I32>> {
        let (path, _) = astar(
            start,
            |a| -> Vec<(Vec2I32, i32)> {
                let possible_successor = vec![
                    Vec2I32::from_i32(a.x + 1, a.y),
                    Vec2I32::from_i32(a.x, a.y + 1),
                    Vec2I32::from_i32(a.x - 1, a.y),
                    Vec2I32::from_i32(a.x, a.y - 1),
                ];
                possible_successor
                    .iter()
                    .filter(|succ| {
                        if succ.x < 0
                            || succ.y < 0
                            || succ.x >= self.map_size
                            || succ.y >= self.map_size
                        {
                            false
                        } else if let Some(entity_id) = &self.occupancy_tracker.get(succ.x, succ.y)
                        {
                            let entity = self.entity_dict.get(entity_id).unwrap();
                            !(entity.player_id == Some(self.my_id) && is_building(entity))
                                || entity.player_id != Some(self.my_id)
                                || is_fighter_unit(entity) && !self.is_attacking_enemy(entity)
                            // my units are passable

                            // !is_fighter_unit(entity) // assuming our fighters always move
                        } else {
                            true
                        }
                    })
                    .map(|succ| {
                        if let Some(entity_id) = &self.occupancy_tracker.get(succ.x, succ.y) {
                            let entity = self.entity_dict.get(entity_id).unwrap();
                            if entity.entity_type == Resource {
                                (*succ, 4)
                            } else {
                                (*succ, 1)
                            }
                        } else {
                            (*succ, 1)
                        }
                    })
                    .collect()
            },
            |a| a.mdist(&target),
            |a| a == target,
        )?;
        Some(path)
    }

    fn find_path_to_closest_resource(&self, origin: &Entity) -> Option<Vec<Vec2I32>> {
        let start = origin.position;
        let mut seen_nodes = 0;
        let max_seen_nodes = 1000;
        bfs(
            &start,
            |a| -> Vec<Vec2I32> {
                if seen_nodes >= max_seen_nodes {
                    return vec!();
                }
                let possible_successor = vec![
                    Vec2I32::from_i32(a.x + 1, a.y),
                    Vec2I32::from_i32(a.x, a.y + 1),
                    Vec2I32::from_i32(a.x - 1, a.y),
                    Vec2I32::from_i32(a.x, a.y - 1),
                ];
                let result: Vec<Vec2I32> = possible_successor
                    .iter()
                    .filter(|succ| {
                        if succ.x < 0
                            || succ.y < 0
                            || succ.x >= self.map_size
                            || succ.y >= self.map_size
                            || self.is_cell_in_enemy_turret_attack_range(succ)
                            || self.influence_map.enemy_influence_at(succ) > 0
                        {
                            false
                        } else if let Some(entity_id) = &self.occupancy_tracker.get_next(succ.x, succ.y) {
                            let entity = self.entity_dict.get(entity_id).unwrap();
                            entity.entity_type == Resource
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect();
                seen_nodes += result.len();
                result
            },
            |possible_success_cell| {
                if let Some(entity_id) = &self
                    .occupancy_tracker
                    .get(possible_success_cell.x, possible_success_cell.y)
                {
                    let entity = self.entity_dict.get(entity_id).unwrap();
                    entity.entity_type == Resource
                } else {
                    false
                }
            },
        )
    }
    fn find_entity_with_min_value<Filter, Valuer>(
        &self,
        filter: Filter,
        valuer: Valuer,
    ) -> Option<&Entity>
        where
            Filter: Fn(&&Entity) -> bool,
            Valuer: Fn(&Entity) -> i32,
    {
        let mut result: Option<(i32, &Entity)> = None;

        for entity in self.player_view.entities.iter().filter(filter) {
            let val = valuer(entity);
            if result.is_none() || result.unwrap().0 > val {
                result = Some((val, entity))
            }
        }

        match result {
            Some((_, e)) => Some(e),
            None => None,
        }
    }

    fn find_closest_entity<F>(&self, origin: &Entity, filter: F) -> Option<&Entity>
        where
            F: Fn(&&Entity) -> bool,
    {
        let pos = origin.center_pos();
        self.find_entity_with_min_value(filter, |e| pos.mdist(&e.position))
    }
    fn find_closest_resource(&self, origin: &Entity) -> Option<Vec2I32> {
        self.find_closest_entity(origin, |e| e.entity_type == Resource)
            .map(|e| e.position)
    }

    fn find_closest_enemy(&self, origin: &Entity) -> Option<Vec2I32> {
        self.find_closest_entity(origin, |e| e.player_id != Some(self.my_id) && is_unit(e))
            .map(|e| e.position)
    }

    fn is_enemy_close_to_base(&self) -> bool {
        for enemy_unit in self
            .player_view
            .entities
            .iter()
            .filter(|e| e.player_id != Some(self.my_id) && is_fighter_unit(e))
        {
            for my_building in self.my_buildings.iter() {
                let properties = &self.entity_properties[&my_building.entity_type];
                let threshold = match my_building.entity_type {
                    MeleeBase | RangedBase | BuilderBase | House | Turret => {
                        (properties.sight_range as f32 * 1.5) as i32 + properties.size / 2
                    }
                    _ => properties.sight_range,
                };

                let building_pos = my_building.center_pos();
                let dist = building_pos.mdist(&enemy_unit.position);
                if dist <= threshold {
                    // println!("found enemy {:?} close to building {:?}, dist {}", enemy_unit, building_pos, dist);
                    return true;
                }
            }
        }
        false
    }

    fn make_empty_action(&self) -> EntityAction {
        self.make_action(None, None, None, None)
    }

    fn make_action(
        &self,
        attack_action: Option<AttackAction>,
        build_action: Option<BuildAction>,
        repair_action: Option<RepairAction>,
        move_action: Option<MoveAction>,
    ) -> EntityAction {
        EntityAction {
            repair_action,
            build_action,
            move_action,
            attack_action,
        }
    }

    fn calculate_cell_occupancy(&self, player_view: &PlayerView) -> Vec<Vec<Option<i32>>> {
        let sz = self.map_size as usize;
        let mut result = vec![vec![None; sz]; sz];
        player_view.entities.iter().for_each(|e| {
            let properties = player_view.entity_properties.get(&e.entity_type).unwrap();
            for i in e.position.x..e.position.x + properties.size {
                for j in e.position.y..e.position.y + properties.size {
                    result[i as usize][j as usize] = Some(e.id)
                }
            }
        });
        result
    }

    fn init_world(&mut self, player_view: &PlayerView) {
        self.my_id = player_view.my_id;
        self.map_size = player_view.map_size;
        self.raic_round = if player_view
            .entities
            .iter()
            .any(|e| matches!(e.player_id, Some(pid) if pid != self.my_id))
        {
            RaicRound::Round1
        } else if player_view.players.len() == 2 {
            RaicRound::Finals
        } else {
            RaicRound::Round2
        }
    }

    fn init_tick(&mut self, player_view: &PlayerView) {
        self.player_view = player_view.clone();
        self.entity_properties = player_view.entity_properties.clone();
        self.me = player_view.me();
        self.score = self.me.score;
        self.resource = self.me.resource;
        self.current_tick = self.player_view.current_tick;

        self.fire_focus.clear();
    }

    fn need_building(&self, entity_type: EntityType) -> bool {
        match entity_type {
            RangedBase => self.count_buildings(entity_type) < 1,
            MeleeBase => self.use_melee && self.count_buildings(entity_type) < 1,
            BuilderBase => self.count_buildings(entity_type) < 1,
            _ => false,
        }
    }

    #[allow(dead_code)]
    fn count_buildings(&self, entity_type: EntityType) -> usize {
        self.my_buildings
            .iter()
            .filter(|e| e.entity_type == entity_type)
            .count()
    }

    #[allow(dead_code)]
    fn count_units(&self, entity_type: EntityType) -> usize {
        self.my_units
            .iter()
            .filter(|e| e.entity_type == entity_type)
            .count()
    }

    fn can_build_at(&self, x: i32, y: i32, size: i32) -> bool {
        for offset_i in -1..size+1 {
            for offset_j in -1..size+1 {
                let i = x + offset_i;
                let j = y + offset_j;
                if i < 0 || i >= self.map_size || j < 0 || j >= self.map_size {
                    return false;
                }
                if let Some(entity_id) = self.occupancy_tracker.get(x + offset_i, y + offset_j) {
                    if offset_i > -1 || offset_i < size || offset_j > -1 || offset_j < size ||
                        is_building(self.entity_dict.get(&entity_id).unwrap()) {
                        return false;
                    }
                }
            }
        }
        true
    }

    #[allow(dead_code)]
    fn find_vacant_patches(&self, size: i32) -> Vec<Vec2I32> {
        let additional_gap = 0;
        let period = size + 1 + additional_gap;
        let mut result = Vec::new();

        for i in (0..self.map_size / 2).step_by(period as usize) {
            for j in (0..self.map_size / 2).step_by(period as usize) {
                let ii = i + additional_gap;
                let jj = j + additional_gap;
                if ii <= 13 && jj <= 13 {
                    continue; // restricted corner
                }
                if self.can_build_at(ii, jj, size) {
                    result.push(Vec2I32 { x: ii, y: jj });
                }
            }
        }
        result
    }

    fn find_vacant_patches_close_to_mines(&self, size: i32) -> Vec<Vec2I32> {
        let additional_gap = 0;
        let period = size + 1 + additional_gap;
        let mut result = Vec::new();

        for i in (0..self.map_size / 2).step_by(period as usize) {
            for j in (0..self.map_size / 2).step_by(period as usize) {
                let ii = i + additional_gap;
                let jj = j + additional_gap;


                if self.can_build_at(ii, jj, size) {
                    let build_loc = Vec2I32 { x: ii, y: jj };
                    let nearby_builders = self.my_builders.iter().filter(|b| b.position.mdist(&build_loc) <= 5).count();
                    let nearby_resources = self.player_view.entities.iter().filter(|res| res.entity_type == Resource && res.position.mdist(&build_loc) <= 7).count();
                    if nearby_builders >= 2 && nearby_resources >= 5 {
                        result.push(build_loc);
                    }
                }
            }
        }
        result
    }

    fn find_vacant_patches2(&self, building_type: EntityType) -> Vec<Vec2I32> {
        let mut city_plan = HashMap::new();
        // let base_locations: Vec<(i32, i32)> = (5..50).flat_map(|i| {
        //     (5..50).map(move |j| (i, j))
        // }).collect();
        // let base_locations = vec![
        //     (5, 5),
        //     (11, 5),
        //     (12, 5),
        //     (13, 5),
        //     (5, 11),
        //     (5, 12),
        //     (5, 13),
        //     (11, 11),
        // ];

        let mut base_locations = Vec::new();
        for i in 0..self.map_size {
            for j in 0..self.map_size {
                base_locations.push((i, j));
            }
        }
        city_plan.insert(EntityType::MeleeBase, base_locations.clone());
        city_plan.insert(EntityType::RangedBase, base_locations.clone());
        city_plan.insert(EntityType::BuilderBase, base_locations.clone());

        city_plan.insert(EntityType::House, base_locations.clone());
        city_plan.insert(EntityType::Turret, base_locations);

        let size = &self.entity_properties[&building_type].size;

        match city_plan.get(&building_type) {
            Some(possible_patches) => possible_patches
                .iter()
                .filter(|(i, j)| self.can_build_at(*i, *j, *size))
                .map(|(i, j)| Vec2I32::from_i32(*i, *j))
                .collect(),
            None => Default::default(),
        }
    }

    fn find_building_adjacent_cell_closest_to_target(
        &self,
        patch_pos: Vec2I32,
        size: i32,
        target_pos: Vec2I32,
    ) -> Option<Vec2I32> {
        let mut res: Option<(i32, i32, i32)> = None;
        let Vec2I32 { x, y } = patch_pos;

        let map_size = self.map_size;
        if x < 0 || x + size >= map_size || y < 0 || y + size >= map_size {
            return None;
        }

        let get_dist =
            |i: i32, j: i32| -> i32 { (i - target_pos.x).abs() + (j - target_pos.y).abs() };

        for i in x..x + size {
            if y > 0 && self.occupancy_tracker.get(i, y - 1).is_none(){
                let dist = get_dist(i, y - 1);
                if res.is_none() || res.unwrap().2 > dist {
                    res = Some((i, y - 1, dist))
                }
            }
            if y + size < map_size - 1 && self.occupancy_tracker.get(i, y + size).is_none() {
                let dist = get_dist(i, y + size);
                if res.is_none() || res.unwrap().2 > dist {
                    res = Some((i, y + size, dist))
                }
            }
        }

        for j in y..y + size {
            if x > 0 && self.occupancy_tracker.get(x - 1, j).is_none(){
                let dist = get_dist(x - 1, j);
                if res.is_none() || res.unwrap().2 > dist {
                    res = Some((x - 1, j, dist))
                }
            }
            if x + size < map_size - 1 && self.occupancy_tracker.get(x + size, j).is_none() {
                let dist = get_dist(x + size, j);
                if res.is_none() || res.unwrap().2 > dist {
                    res = Some((x + size, j, dist))
                }
            }
        }

        match res {
            Some((x, y, _)) => Some(Vec2I32 { x, y }),
            None => None,
        }
    }

    fn display_unit_info(
        &self,
        mouse_pos_world: Vec2F32,
        player_view: &PlayerView,
        debug_interface: &mut DebugInterface,
    ) {
        let int_pos: Vec2I32 = mouse_pos_world.into();

        if let Some(unit) = player_view.entities.iter().find(|e| {
            let size = self.entity_properties[&e.entity_type].size;

            e.position.x <= int_pos.x
                && e.position.x + size > int_pos.x
                && e.position.y <= int_pos.y
                && e.position.y + size > int_pos.y
        }) {
            // debug_interface.log_text(format!("{:?} {}, hp: {}", unit.entity_type, unit.id, unit.health).to_string());
            let properties = &self.entity_properties[&unit.entity_type];

            if let Some(atk_props) = &properties.attack {
                debug_interface.unit_label(
                    mouse_pos_world,
                    format!(
                        "#{}, hp: {}/{}, atk: {}",
                        unit.id, unit.health, properties.max_health, atk_props.damage
                    ),
                );

                display_entity_range(
                    unit,
                    atk_props.attack_range,
                    Color::green().set_a(0.4),
                    player_view,
                    debug_interface,
                );
            } else {
                debug_interface.unit_label(
                    mouse_pos_world,
                    format!(
                        "#{}, hp: {}/{}",
                        unit.id, unit.health, properties.max_health
                    ),
                )
            }

            display_entity_range(
                unit,
                properties.sight_range,
                Color::yellow().set_a(0.3),
                player_view,
                debug_interface,
            );

            // highlight whom we are attacking
            if let Some(UnitOrder::Attack { enemy_id }) = self.unit_orders.get(&unit.id) {
                let enemy = self.entity_dict.get(&enemy_id).unwrap();
                debug_interface.fill_cell(enemy.position.x, enemy.position.y, Color::purple());
            }
            // for enemy units, highlight who attacked it
            for (player_unit_id, player_unit_order) in self.unit_orders.iter() {
                if let UnitOrder::Attack { enemy_id } = player_unit_order {
                    if *enemy_id == unit.id {
                        let player_unit = self.entity_dict.get(player_unit_id).unwrap();
                        debug_interface.fill_cell(
                            player_unit.position.x,
                            player_unit.position.y,
                            Color::yellow(),
                        );
                    }
                }
            }

            match unit.entity_type {
                BuilderUnit => {
                    if let Some(path) = self.find_path_to_closest_resource(unit) {
                        visualize_path(&path, debug_interface);
                    }
                },
                RangedUnit => {
                    if let Some(path) = self.find_path_to_closest_enemy(unit, |e| is_unit(e)) {
                        visualize_path(&path, debug_interface);
                    }
                }
                _ => {}
            }


            if let Some(unit_order) = self.unit_orders.get(&unit.id) {
                debug_interface.log_text(format!("order {:?}", unit_order))
            } else {
                debug_interface.log_text("no active order".to_string());
            }

            if let Some(UnitOrder::FollowPath { path, current_idx }) =
            self.unit_orders.get(&unit.id)
            {
                visualize_path_color(
                    &path.iter().skip(*current_idx).cloned().collect::<Vec<Vec2I32>>(),
                    Color::red(),
                    debug_interface,
                );
            }

            if let Some(entity_action) = self.last_known_actions.get(&unit.id) {
                debug_interface.log_text(format!("entity_action {:?}", entity_action))
            } else {
                debug_interface.log_text("no active entity action".to_string());
            }

            if is_building(unit) {
                let active_repairers: Vec<i32> = self
                    .unit_orders
                    .iter()
                    .filter(|(_, order)| matches!(order, UnitOrder::RepairBuilding { building_id } if *building_id == unit.id))
                    .map(|(unit_id, _)|  *unit_id)
                    .collect();

                debug_interface.log_text(format!("repairers: {:?}", active_repairers));
            }
        }
    }

    fn is_attacking_enemy(&self, unit: &Entity) -> bool {
        matches!(
            self.unit_orders.get(&unit.id),
            Some(&UnitOrder::Attack { .. })
        )
    }

    fn is_cell_in_enemy_turret_attack_range(&self, loc: &Vec2I32) -> bool {
        self.influence_map.is_turret_attack_at(loc)
    }

    // we can move to this cell if no other unit will be there on the next tick
    fn can_move_to(&self, _unit: &Entity, target_pos: &Vec2I32) -> bool {
        // TODO: check for unit swap scenarios
        self.occupancy_tracker
            .get_next(target_pos.x, target_pos.y)
            .is_none()
    }
}

fn is_building(e: &Entity) -> bool {
    matches!(
        e.entity_type,
        House | BuilderBase | MeleeBase | RangedBase | Turret | Wall
    )
}

fn is_barrack(e: &Entity) -> bool {
    matches!(e.entity_type, BuilderBase | MeleeBase | RangedBase)
}

fn is_unit(e: &Entity) -> bool {
    matches!(e.entity_type, BuilderUnit | MeleeUnit | RangedUnit)
}

fn is_fighter_unit(e: &Entity) -> bool {
    matches!(e.entity_type, MeleeUnit | RangedUnit)
}
