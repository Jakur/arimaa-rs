pub mod game;
pub mod position;
pub mod zobrist;
#[cfg(test)]
mod tests {
    use crate::game::Move;
    use crate::position;
    use crate::position::{neighbors_of, Piece, Position, Side, Step};

    static POS1: &'static str = include_str!("test_games/pos1.txt");
    #[test]
    fn new_start() {
        let op = "Ra1 Db1 Rc1 Rd1 De1 Rf1 Cg1 Rh1 Ra2 Hb2 Cc2 Ed2 Me2 Rf2 Hg2 Rh2
        ha7 mb7 cc7 dd7 ee7 cf7 hg7 rh7 ra8 rb8 rc8 rd8 de8 rf8 rg8 rh8";
        let pos = Position::from_opening_str(op);
        assert!(pos.is_some());
        let pos = pos.unwrap();
        let wneighbors = neighbors_of(pos.placement[0]);
        let bneighbors = neighbors_of(pos.placement[1]);
        assert_eq!(wneighbors, 0xFFFFFF); // rows 1, 2, 3
        assert_eq!(bneighbors, 0xFFFFFF0000000000); // rows 6, 7, 8
    }
    #[test]
    fn total_moves() {
        let total = position::total_moves();
        assert!(512 < total && total < 1569); // Somewhat loose bounds for sanity check
    }
    #[test]
    fn test_small_notation() {
        let note = "[ rr r r m h  e c r  r r h dr c dE H    M R RRHR D C  C D R RR R ]";
        let p = Position::from_small_notation(note.to_string(), Side::Black);
        assert!(p.is_ok());
        //println!("{}", p.unwrap().to_pos_notation());
        assert_eq!(note, p.unwrap().to_small_notation());
    }

    fn call_perl(fname: &str) -> String {
        use std::process::Command;
        let prefix = "/home/justin/Downloads/ArimaaMoveCount";
        let res = Command::new(format!("{}/mc", prefix))
            .arg(format!("{}/{}", prefix, fname))
            .output()
            .expect("Failed!")
            .stdout;
        let res = std::str::from_utf8(&res[..]).expect("Invalid UTF-8 string");
        //println!("{}", res);
        res.into()
    }

    fn parse_perl(input: String) -> (Vec<Move>, Vec<String>) {
        let split: Vec<_> = input.split("are unique\n\n").collect();
        //println!("{}", split[1]);
        let mut moves = Vec::new();
        let mut position_strings = Vec::new();
        for (count, line) in split[1].lines().enumerate() {
            if count % 2 == 0 {
                // Move
                moves.push(Move::from_line(line))
            } else {
                position_strings.push(line.to_string());
            }
        }
        (moves, position_strings)
    }
    #[test]
    fn test_perl_call() {
        //use std::collections::HashSet;
        let (moves, position_strings) = parse_perl(call_perl("pos2"));
        //let hashset: HashSet<_> = position_strings.into_iter().collect();
        println!("{}", position::alg_to_index(&['a', '8']).unwrap());
        let init_pos = Position::from_pos_notation(POS1.to_string()).unwrap();
        for (m, pos_string) in moves.into_iter().zip(position_strings.into_iter()) {
            let mut pos = init_pos.clone();
            for step in m.steps.iter() {
                pos.do_step(*step);
            }
            let long = Position::from_small_notation(pos_string.clone(), Side::Black)
                .unwrap()
                .to_pos_notation();
            // let left = pos.to_pos_notation();
            // let right = long.lines();
            // let s1 = match m.steps.get(2) {
            //     Some(s) => format!("{}", s),
            //     None => " ".to_string(),
            // };
            // let s2 = match m.steps.get(3) {
            //     Some(s) => format!("{}", s),
            //     None => " ".to_string(),
            // };
            // println!("Attempting: {} {}", s1, s2);
            // for (l, r) in left.lines().zip(right) {
            //     println!("{}     {}", l, r);
            // }
            // println!("{}", pos.to_pos_notation());
            // println!("{}", long);
            assert_eq!(pos.to_pos_notation(), long);
            assert_eq!(pos.to_small_notation(), pos_string);
        }
        //println!("{}", hashset.len());
    }
    #[test]
    fn test_step_gen() {
        use std::collections::HashMap;
        let pos = Position::from_pos_notation(POS1.to_string()).unwrap();
        // pos.steps_left = 2;
        // 68891 moves from black's pos
        // pos.side = Side::Black;
        let (correct_steps, correct_positions) = parse_perl(call_perl("pos"));
        let c_pos_set: HashMap<_, _> = correct_positions
            .iter()
            .enumerate()
            .map(|(i, p)| (p.as_str(), i))
            .collect();
        //let c_move_set: HashMap<_, _> = correct_steps.iter().enumerate().map(|(i, m)|, ())
        let found_positions = crate::game::Move::all_positions(&pos);
        // assert_eq!(c_pos_set.len(), found_positions.len());
        println!(
            "Correct: {}     Found: {}",
            c_pos_set.len(),
            found_positions.len()
        );
        let mut counter = 0;
        for (found_p, moves) in found_positions {
            if !c_pos_set.contains_key(found_p.to_small_notation().as_str()) {
                counter += 1;
                println!("{:?}", moves);
                println!("{}", found_p.to_pos_notation());
                //break;
            }
        }
        assert_eq!(counter, 0);
    }

    #[test]
    fn test_pos_notation() {
        use std::collections::HashSet;
        let mut pos = Position::from_pos_notation(POS1.to_string()).unwrap();
        pos.steps_left = 1;
        println!("{:?}", &pos.pieces[..]);
        let note = pos.to_pos_notation();
        println!("{}", note);
        assert_eq!(POS1, note);
        assert_eq!(
            "dc2e",
            format!("{}", Step::Move(position::Piece::BDog, 10, 11))
        );
        let white_steps = include_str!("test_games/steps1.txt");
        let black_steps = include_str!("test_games/steps2.txt");
        for side in &[Side::White, Side::Black] {
            pos.side = *side;
            let steps = pos.gen_steps();
            let correct_steps = {
                if let Side::White = side {
                    white_steps
                } else {
                    black_steps
                }
            };
            let correct_steps: HashSet<_> = correct_steps
                .to_string()
                .lines()
                .map(|s| s.to_string())
                .collect();
            println!("Number of steps: {}", steps.len());
            let found_steps: HashSet<_> = steps.iter().map(|s| format!("{}", s)).collect();
            // for s in steps {
            //     println!("{}", s);
            // }
            let left = correct_steps.difference(&found_steps);
            println!("Correct but not found: ");
            for val in left {
                println!("{}", val);
            }
            let right = found_steps.difference(&correct_steps);
            println!("\nFound but not correct: ");
            for val in right {
                println!("{}", val)
            }
            let index = position::alg_to_index(&['f', '3']).unwrap();
            assert_eq!(position::Piece::WHorse, pos.pieces[index]);
        }

        //assert!(lsb & pos.bitboards[0] == 0)
    }
    #[test]
    fn test_bit_tricks() {
        use position::Bitboard;
        let t1 = 0b0110;
        let t2 = 0b1111;
        let t3 = 0b1000_0000;
        assert_eq!(t1.bitscan_forward(), 1);
        assert_eq!(t2.bitscan_forward(), 0);
        assert_eq!(t3.bitscan_forward(), 7);
        let index = position::alg_to_index(&['f', '3']).unwrap();
        println!("{}", index);
        println!("{:b}", position::index_to_lsb(index as u8));
    }
    #[test]
    fn test_step_display() {
        use position::alg_to_index;
        let source = alg_to_index(&['c', '2']).unwrap();
        let dest = alg_to_index(&['c', '1']).unwrap();
        let step = Step::Move(Piece::WCat, source as u8, dest as u8);
        println!("{} -> {}", source, dest);
        assert_eq!(format!("{}", step), "Cc2s".to_string());
    }
    // #[test]
    // fn gen_zobrist() {
    //     let name = concat!(file!(), ".zobrist");
    //     let mut f = std::fs::File::create(name).unwrap();
    //     crate::zobrist::write_zobrist(&mut f);
    // }
    #[test]
    fn test_cleanup() {
        cleanup_gameroom_logs("/home/justin/Downloads/allgames201301.txt");
    }
    fn cleanup_gameroom_logs(fname: &str) {
        use regex::Regex;

        use std::fs::File;
        use std::io::prelude::*;
        let mut contents = String::new();
        let mut f = File::open(fname).unwrap();
        let re_game = Regex::new(r"game finished with result (.*?)\n").unwrap();
        // Capture move numbers and individual steps
        let re_steps = Regex::new(r"\d{1,3}[wb]|[RrCcDdHhMmEe][a-h]\d\w?").unwrap();
        f.read_to_string(&mut contents).unwrap();
        let mut split = re_game.split(&contents);
        let sp = split.nth(101).unwrap(); // Arbitrary for testing
        let vec: Vec<_> = sp.split("\t1w ").collect();
        let game_str = vec[vec.len() - 1];
        let steps: Vec<_> = re_steps.find_iter(game_str).map(|c| c.as_str()).collect();
        let mut last_move_str = "1w";
        for s in steps.iter().rev() {
            let first_c = s.chars().next();
            if let Some(c) = first_c {
                if c.is_digit(10) {
                    // Last move indicator
                    last_move_str = s;
                    break;
                }
            }
        }
        dbg!(last_move_str);
    }
}
