use super::*;

/// A unit
#[derive(Clone, Debug)]
pub struct Unit {
    /// Unique id
    pub id: i32,
    /// Id of the player (team) controlling the unit
    pub player_id: i32,
    /// Current health
    pub health: f64,
    /// Current shield value
    pub shield: f64,
    /// Left extra lives of this unit
    pub extra_lives: i32,
    /// Current position of unit's center
    pub position: model::Vec2,
    /// Remaining time until unit will be spawned, or None
    pub remaining_spawn_time: Option<f64>,
    /// Current velocity
    pub velocity: model::Vec2,
    /// Current view direction (vector of length 1)
    pub direction: model::Vec2,
    /// Value describing process of aiming (0 - not aiming, 1 - ready to shoot)
    pub aim: f64,
    /// Current action unit is performing, or None
    pub action: Option<model::Action>,
    /// Tick when health regeneration will start (can be less than current game tick)
    pub health_regeneration_start_tick: i32,
    /// Index of the weapon this unit is holding (starting with 0), or None
    pub weapon: Option<i32>,
    /// Next tick when unit can shoot again (can be less than current game tick)
    pub next_shot_tick: i32,
    /// List of ammo in unit's inventory for every weapon type
    pub ammo: Vec<i32>,
    /// Number of shield potions in inventory
    pub shield_potions: i32,

    pub seen_on_tick: i32,
}

pub enum LootPriority {
    Weapon,
    Shield,
    Ammo,
    Whatever,
}

impl Unit {
    pub fn intersects_with(&self, p0: &Vec2, p1: &Vec2) -> bool {
        HittableEntity::from(self).intersects_with(p0, p1)
    }

    pub fn is_within_fire_range_of(&self, unit: &Unit, constants: &Constants) -> bool {
        if let Some(weapon_idx) = unit.weapon {
            let weapon = &constants.weapons[weapon_idx as usize];
            let fire_range = weapon.projectile_speed * weapon.projectile_life_time;
            self.position.distance_to(&unit.position) <= fire_range
        } else {
            println!("no weapon");
            false
        }
    }

    pub fn priority(&self) -> LootPriority {

        let weapon_idx = self.weapon.unwrap_or(2) as usize;
        let bow_idx = 2;
        let max_ammo = [100, 250, 25];

        for widx in [weapon_idx, bow_idx] {
            if self.ammo[widx] <= max_ammo[widx] / 5 {
                return LootPriority::Ammo
            }
        }

        if self.weapon.is_none() || self.weapon.unwrap() != 2 {
            return LootPriority::Weapon{}
        }

        if self.shield_potions < 3 {
            return LootPriority::Shield
        }

        LootPriority::Whatever
    }

    pub fn weapon_range(&self, constants: &Constants) -> f64 {
        match self.get_weapon(constants) {
            None => 0.0,
            Some(weapon) => weapon.range()
        }
    }

    pub fn get_weapon<'a>(&self, constants: &'a Constants) -> Option<&'a WeaponProperties> {
        Some(&constants.weapons[self.weapon? as usize])
    }
}

impl trans::Trans for Unit {
    fn write_to(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.id.write_to(writer)?;
        self.player_id.write_to(writer)?;
        self.health.write_to(writer)?;
        self.shield.write_to(writer)?;
        self.extra_lives.write_to(writer)?;
        self.position.write_to(writer)?;
        self.remaining_spawn_time.write_to(writer)?;
        self.velocity.write_to(writer)?;
        self.direction.write_to(writer)?;
        self.aim.write_to(writer)?;
        self.action.write_to(writer)?;
        self.health_regeneration_start_tick.write_to(writer)?;
        self.weapon.write_to(writer)?;
        self.next_shot_tick.write_to(writer)?;
        self.ammo.write_to(writer)?;
        self.shield_potions.write_to(writer)?;
        Ok(())
    }
    fn read_from(reader: &mut dyn std::io::Read) -> std::io::Result<Self> {
        let id: i32 = trans::Trans::read_from(reader)?;
        let player_id: i32 = trans::Trans::read_from(reader)?;
        let health: f64 = trans::Trans::read_from(reader)?;
        let shield: f64 = trans::Trans::read_from(reader)?;
        let extra_lives: i32 = trans::Trans::read_from(reader)?;
        let position: model::Vec2 = trans::Trans::read_from(reader)?;
        let remaining_spawn_time: Option<f64> = trans::Trans::read_from(reader)?;
        let velocity: model::Vec2 = trans::Trans::read_from(reader)?;
        let direction: model::Vec2 = trans::Trans::read_from(reader)?;
        let aim: f64 = trans::Trans::read_from(reader)?;
        let action: Option<model::Action> = trans::Trans::read_from(reader)?;
        let health_regeneration_start_tick: i32 = trans::Trans::read_from(reader)?;
        let weapon: Option<i32> = trans::Trans::read_from(reader)?;
        let next_shot_tick: i32 = trans::Trans::read_from(reader)?;
        let ammo: Vec<i32> = trans::Trans::read_from(reader)?;
        let shield_potions: i32 = trans::Trans::read_from(reader)?;
        Ok(Self {
            id,
            player_id,
            health,
            shield,
            extra_lives,
            position,
            remaining_spawn_time,
            velocity,
            direction,
            aim,
            action,
            health_regeneration_start_tick,
            weapon,
            next_shot_tick,
            ammo,
            shield_potions,
            seen_on_tick: -1,
        })
    }
}