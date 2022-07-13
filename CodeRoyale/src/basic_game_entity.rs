use ai_cup_22::model::*;

pub trait BasicGameEntity {
    fn id(&self) -> i32;
    fn position(&self) -> Vec2;
    fn velocity(&self) -> Vec2;
    fn is_still_relevant(&self, current_tick: i32) -> bool;
    fn mark_seen(&mut self, tick: i32);
}

impl BasicGameEntity for Unit {
    fn id(&self) -> i32 {
        self.id
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn velocity(&self) -> Vec2 {
        self.velocity
    }

    fn is_still_relevant(&self, current_tick: i32) -> bool {
        current_tick - self.seen_on_tick < 50
    }

    fn mark_seen(&mut self, tick: i32) {
        self.seen_on_tick = tick;
    }
}

impl BasicGameEntity for Loot {
    fn id(&self) -> i32 {
        self.id
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn velocity(&self) -> Vec2 {
        Vec2::zero()
    }

    fn is_still_relevant(&self, current_tick: i32) -> bool {
        current_tick - self.seen_on_tick < 50
    }

    fn mark_seen(&mut self, tick: i32) {
        self.seen_on_tick = tick;
    }
}

impl BasicGameEntity for Projectile {
    fn id(&self) -> i32 {
        self.id
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn velocity(&self) -> Vec2 {
        self.velocity
    }

    fn is_still_relevant(&self, current_tick: i32) -> bool {
        let not_seen_for = (current_tick - self.seen_on_tick) as f64;
        let ticks_per_second = 30.0;
        let max_life = 1.0;
        let life_spent_per_tick = max_life / ticks_per_second;
        let life_remaining = self.life_time - not_seen_for * life_spent_per_tick;
        life_remaining > 0.0 && current_tick - self.seen_on_tick < 50
    }

    fn mark_seen(&mut self, tick: i32) {
        self.seen_on_tick = tick;
    }
}