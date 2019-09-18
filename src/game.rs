use crate::position::{Position, Step};
use std::collections::HashSet;

pub struct Game {
    pub position: Position,
}

pub struct Move {
    pub steps: Vec<Step>,
}

impl Move {
    pub fn all_moves(position: &Position) -> Vec<Move> {
        // Todo more robust
        let mut total = 0;
        let mut unique_pos = HashSet::new();
        let one_left = position.gen_steps().into_iter().map(|s| {
            // println!("Attempting: {}", s);
            let mut p = position.clone();
            p.do_step(s, false);
            p
        });
        //println!("Length of one_left {}", one_left.len());
        //return Vec::new();
        for p in one_left {
            // println!("Position: {}", p.to_pos_notation());
            let fin = p.gen_steps().into_iter().map(|s| {
                total += 1;
                let mut p = position.clone();
                p.do_step(s, false);
                p
            });
            for f in fin {
                unique_pos.insert(f.current_hash);
            }
        }
        println!("Total moves: {}", total);
        println!("Unique moves: {}", unique_pos.len());
        vec![]
    }
}
