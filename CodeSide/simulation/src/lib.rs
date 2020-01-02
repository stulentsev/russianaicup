use model::*;

pub fn unit_state_next_tick(game: &Game, unit: &Unit, velocity: f64, jump: bool, ticks: f64) -> Unit {
    let x = unit.position.x + velocity / game.properties.ticks_per_second * ticks;
    let mut y = unit.position.y;
    if jump {
        y += unit.jump_state.speed / game.properties.ticks_per_second * ticks
    } else {
        y -= game.properties.unit_fall_speed / game.properties.ticks_per_second * ticks
    }

    Unit {
        position: Vec2F64 { x, y },
        ..(*unit)
    }
}

pub fn bullet_state_next_tick(game: &Game, bullet: &Bullet, ticks: f64) -> Bullet {
    let x = bullet.position.x + bullet.velocity.x / game.properties.ticks_per_second * ticks;
    let y = bullet.position.y + bullet.velocity.y / game.properties.ticks_per_second * ticks;

    Bullet {
        position: Vec2F64 { x, y },
        ..(*bullet)
    }
}