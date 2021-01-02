use model::*;

#[derive(Default)]
pub struct OccupancyTracker {
    current_occupancy: Vec<Vec<Option<i32>>>,
    next_tick_occupancy: Vec<Vec<Option<i32>>>,
}

impl OccupancyTracker {
    // pub fn new() -> Self {
    //     Self {
    //         current_occupancy: Vec::with_capacity(6400),
    //         next_tick_occupancy: Vec::with_capacity(6400),
    //     }
    // }

    pub fn set_current(&mut self, occ: Vec<Vec<Option<i32>>>) {
        self.current_occupancy = occ;
        self.next_tick_occupancy = self.current_occupancy.clone();
    }

    pub fn get(&self, x: i32, y: i32) -> Option<i32> {
        self.current_occupancy[x as usize][y as usize]
    }

    pub fn get_next(&self, x: i32, y: i32) -> Option<i32> {
        self.next_tick_occupancy[x as usize][y as usize]
    }

    pub fn maybe_update_next_tick(&mut self, unit: &Entity, action: &EntityAction) {
        if let Some(move_action) = &action.move_action {
            if move_action.target.mdist(&unit.position) == 1 {
                // next cell
                self.current_occupancy[unit.position.x as usize][unit.position.y as usize] = None;
                self.next_tick_occupancy[move_action.target.x as usize]
                    [move_action.target.y as usize] = Some(unit.id);
            }
        }
    }
}
