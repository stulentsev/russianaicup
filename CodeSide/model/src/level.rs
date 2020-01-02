use crate::*;

#[derive(Clone, Debug, trans::Trans)]
pub struct Level {
    pub tiles: Vec<Vec<Tile>>,
}

impl Level {
    pub fn cell_at_f64(&self, x: f64, y: f64) -> Tile {
        if x < 0.0 || y < 0.0 {
            return Tile::Wall;
        }

        self.tiles[x as usize][y as usize]
    }

    pub fn cell_at_i32(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 ||
            x >= self.tiles.len() as i32 ||
            y >= self.tiles[0].len() as i32 {
            return Tile::Wall;
        }
        self.tiles[x as usize][y as usize]
    }
}