use crate::position::{Position, Side, Step};
use std::collections::{hash_map::Entry, HashMap, HashSet};
//use std::fmt;

pub struct Game {
    pub position: Position,
}

pub struct Move {
    pub steps: Vec<Step>,
}

impl Move {
    pub fn from_line(line: &str) -> Move {
        let steps = line.split(" ").map(|s| Step::from_notation(s)).collect();
        Move { steps }
    }
    pub fn all_moves(position: &Position) -> Vec<Move> {
        // Todo more robust
        let mut total = 0;
        let mut unique_pos = HashSet::new();
        let mut unique_string_pos = HashSet::new();
        let mut hashmap = HashMap::new();
        let one_left = position.gen_steps().into_iter().map(|s| {
            println!("Attempting: {}", s);
            let mut p = position.clone();
            p.do_step(s, false);
            p
        });
        println!("Length of one_left {}", one_left.len());
        //return Vec::new();
        for p in one_left {
            if let Side::Black = p.side {
                //println!("Black to move...");
                total += 1;
                continue;
            }
            // println!("Position: {}", p.to_pos_notation());
            let fin = p.gen_steps().into_iter().map(|s| {
                total += 1;
                let mut p = position.clone();
                p.do_step(s, false);
                p
            });
            for f in fin {
                let small_note = f.to_small_notation();
                unique_pos.insert(f.current_hash);
                let entry = hashmap.entry(f.current_hash);
                match entry {
                    Entry::Occupied(entry) => {
                        if small_note != *entry.get() {
                            println!("Inconsistent!");
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(f.to_small_notation());
                    }
                }
                unique_string_pos.insert(small_note);
            }
        }
        println!("Total moves: {}", total);
        println!("Unique moves: {}", unique_pos.len());
        println!("Unique pos strings: {}", unique_string_pos.len());
        vec![]
    }
}
