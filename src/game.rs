use crate::position::{Position, Step};

pub struct Game {
    pub position: Position,
}

pub struct Move {
    pub steps: Vec<Step>,
}

impl Move {
    fn all_moves(position: &Position) -> Vec<Move> {
        unimplemented!()
    }
}
