pub mod position;
pub mod zobrist;
#[cfg(test)]
mod tests {
    use crate::position;
    use crate::position::{neighbors_of, Position, Side};
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
        let pos1 = include_str!("test_games/pos1.txt");
        let pos = Position::from_pos_notation(pos1.to_string()).unwrap();
        println!("{:?}", &pos.pieces[..]);
        let note = pos.to_pos_notation();
        println!("{}", note);
        assert_eq!(pos1, note);
    }

    #[test]
    fn test_load_zobrist() {
        assert_eq!(602977864700505253, crate::zobrist::get_zobrist(0, 0, 0))
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
