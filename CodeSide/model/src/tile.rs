#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, trans::Trans)]
pub enum Tile {
    Empty,
    Wall,
    Platform,
    Ladder,
    JumpPad,
}

impl Tile {
    pub fn can_jump_from(&self) -> bool {
        match self {
            Tile::Wall | Tile::Platform | Tile::Ladder => true,
            _ => false
        }
    }
}