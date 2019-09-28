use crate::position::{Position, Side, Step};
use std::collections::{hash_map::Entry, HashMap, HashSet};
//use std::fmt;

pub struct Game {
    pub position: Position,
}
#[derive(Clone)]
pub struct Move {
    pub steps: Vec<Step>,
}

impl Move {
    pub fn new(steps: Vec<Step>) -> Move {
        Move { steps }
    }
    pub fn from_line(line: &str) -> Move {
        let steps = line.split(" ").map(|s| Step::from_notation(s)).collect();
        Move { steps }
    }
    pub fn all_positions(position: &Position) -> HashMap<Position, Vec<Move>> {
        let init_side = position.side;
        let mut in_progress = vec![(position.clone(), Move::new(vec![]))];
        let mut finished: HashMap<_, Vec<Move>> = HashMap::new();
        while in_progress.len() != 0 {
            let (pos, mov) = in_progress.pop().unwrap();
            if pos.side != init_side {
                let entry = finished.entry(pos);
                match entry {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(mov);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![mov]);
                    }
                }
                continue;
            }
            for s in pos.gen_steps().into_iter() {
                let mut next_pos = pos.clone();
                let mut next_move = mov.clone();
                next_pos.do_step(s, false);
                next_move.steps.push(s);
                in_progress.push((next_pos, next_move));
            }
        }
        finished
    }
}
