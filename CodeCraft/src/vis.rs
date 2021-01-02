use crate::my_strategy::UnitOrder;
use crate::DebugInterface;
use itertools::{Itertools, MinMaxResult};
use model::*;
use std::collections::HashMap;

pub fn display_turret_fields(player_view: &PlayerView, debug_interface: &mut DebugInterface) {
    for turret in player_view.entities.iter().filter(|e| {
        use EntityType::*;
        matches!(
            e.entity_type,
            Turret | House | BuilderBase | MeleeBase | RangedBase | Wall
        )
    }) {
        let color = if turret.player_id == Some(player_view.my_id) {
            Color::blue()
        } else {
            Color::red()
        };
        let properties = &player_view.entity_properties[&turret.entity_type];

        display_entity_range(
            turret,
            properties.sight_range,
            color.set_a(0.2),
            player_view,
            debug_interface,
        );
        if let Some(attack) = &properties.attack {
            display_entity_range(
                turret,
                attack.attack_range,
                color.set_a(0.5),
                player_view,
                debug_interface,
            );
        }
    }
}

pub fn display_builder_fields(player_view: &PlayerView, debug_interface: &mut DebugInterface) {
    for builder in player_view.entities.iter().filter(|e| {
        e.player_id == Some(player_view.my_id) && e.entity_type == EntityType::BuilderUnit
    }) {
        let color = Color::green().set_a(0.1);
        let properties = &player_view.entity_properties[&builder.entity_type];

        display_entity_range(
            builder,
            properties.sight_range,
            color,
            player_view,
            debug_interface,
        );
    }
}

pub fn display_all_fields(player_view: &PlayerView, debug_interface: &mut DebugInterface) {
    for entity in player_view.entities.iter() {
        let color = if entity.player_id == Some(player_view.my_id) {
            Color::blue()
        } else {
            Color::red()
        };
        let properties = &player_view.entity_properties[&entity.entity_type];

        display_entity_range(
            entity,
            properties.sight_range,
            color.set_a(0.1),
            player_view,
            debug_interface,
        );
    }
}

pub fn display_entity_range(
    entity: &Entity,
    range: i32,
    color: Color,
    _player_view: &PlayerView,
    debug_interface: &mut DebugInterface,
) {
    for cell in entity.range_cells(range as usize) {
        debug_interface.fill_cell(cell.x, cell.y, color);
    }
}

pub fn visualize_path(path: &[Vec2I32], debug_interface: &mut DebugInterface) {
    visualize_path_color(path, model::Color::blue(), debug_interface);
}
pub fn visualize_path_color(
    path: &[Vec2I32],
    color: model::Color,
    debug_interface: &mut DebugInterface,
) {
    for cell in path.iter() {
        debug_interface.mark_cell(cell.x, cell.y, color)
    }
}

pub fn visualize_attacks(
    entity_dict: &HashMap<i32, Entity>,
    unit_orders: &HashMap<i32, UnitOrder>,
    debug_interface: &mut DebugInterface,
) {
    let color_from = Color::blue();
    let color_to = Color::red();

    for (unit_id, order) in unit_orders.iter() {
        if let UnitOrder::Attack { enemy_id } = order {
            let origin = entity_dict.get(&unit_id).unwrap().center_pos_f32();
            let target = entity_dict.get(&enemy_id).unwrap().center_pos_f32();

            debug_interface.line_gradient(
                ColoredVertex {
                    world_pos: Some(origin),
                    screen_offset: Default::default(),
                    color: color_from,
                },
                ColoredVertex {
                    world_pos: Some(target),
                    screen_offset: Default::default(),
                    color: color_to,
                },
            )
        }
    }
}

pub fn visualize_influence_map(map: &[i32], _hue: i32, debug_interface: &mut DebugInterface) {
    let (min, h_min, max, h_max) = match map.iter().minmax() {
        MinMaxResult::NoElements => return,
        MinMaxResult::OneElement(v) => (v, 75.0, v, 75.0),
        MinMaxResult::MinMax(v1, v2) => (v1, 0.0, v2, 75.0),
    };
    let base = min.abs().max(*max);

    for (i, val) in map.iter().enumerate() {
        let x = i / 80;
        let y = i % 80;
        if *val > 0 {
            let normalized_value = *val as f32 / base as f32;
            let color = Color::from_hsv(h_max as f32, normalized_value, 0.75);
            debug_interface.fill_cell(x as i32, y as i32, color);
        } else {
            let normalized_value = val.abs() as f32 / base as f32;
            let color = Color::from_hsv(h_min as f32, normalized_value, 0.75);
            debug_interface.fill_cell(x as i32, y as i32, color);
        }
    }
}

pub fn visualize_influence_map_one_color(
    map: &[i32],
    hue: i32,
    debug_interface: &mut DebugInterface,
) {
    let max = *map.iter().max().unwrap_or(&0);
    let min = 0;

    let base = max - min;

    for (i, val) in map.iter().enumerate() {
        let x = i / 80;
        let y = i % 80;

        let normalized_value = *val as f32 / base as f32;
        let color = Color::from_hsv(hue as f32, normalized_value, 0.75);
        debug_interface.fill_cell(x as i32, y as i32, color);
    }
}
