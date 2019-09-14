pub mod game;
pub mod position;
pub mod zobrist;
#[cfg(test)]
mod tests {
    use crate::position;
    use crate::position::{neighbors_of, Piece, Position, Side, Step};
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
    fn test_pos_notation() {
        use std::collections::HashSet;
        let text_pos = include_str!("test_games/pos1.txt");
        let mut pos = Position::from_pos_notation(text_pos.to_string()).unwrap();
        pos.steps_left = 1;
        println!("{:?}", &pos.pieces[..]);
        let note = pos.to_pos_notation();
        println!("{}", note);
        assert_eq!(text_pos, note);
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
            let lsb = position::index_to_lsb(index as u8);
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
