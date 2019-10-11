use failure::{bail, ensure, format_err, Error};

use bitintr::Tzcnt;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::zobrist::{color_hash, compute_hash, update_hash};
//const ALL_BITS_SET: u64 = 0xFFFFFFFFFFFFFFFF;

const A_FILE: u64 = 0x8080808080808080;
// const B_FILE: u64 = 0x4040404040404040;
// const G_FILE: u64 = 0x0202020202020202;
const H_FILE: u64 = 0x0101010101010101;
const NOT_A_FILE: u64 = !A_FILE;
const NOT_H_FILE: u64 = !H_FILE;
const RANK_1: u64 = 0xFF;
// const RANK_2: u64 = 0xFF00;
// const RANK_7: u64 = 0xFF000000000000;
const RANK_8: u64 = 0xFF00000000000000;
// const NOT_RANK_1: u64 = !RANK_1;
// const NOT_RANK_8: u64 = !RANK_8;
// const NOT_EDGE: u64 = NOT_A_FILE & NOT_H_FILE & NOT_RANK_1 & NOT_RANK_8;

// const TRAPS: u64 = 0x0000240000240000;
// const TRAP_C3: u64 = 0x200000;
// const TRAP_F3: u64 = 0x40000;
// const TRAP_C6: u64 = 0x200000000000;
// const TRAP_F6: u64 = 0x40000000000;
// const TRAP_F3_IX: u8 = 18;
// const TRAP_C3_IX: u8 = 21;
// const TRAP_F6_IX: u8 = 42;
// const TRAP_C6_IX: u8 = 45;
const TRAP_INDICES: [usize; 4] = [18, 21, 42, 45];
const TRAP_NEIGHBORS: [u64; 4] = [0x40A0400, 0x20502000, 0x40A0400000000, 0x20502000000000];

#[derive(PartialEq)]
pub enum EndState {
    WhiteWin,
    BlackWin,
    Neither,
}

impl From<Side> for EndState {
    fn from(item: Side) -> EndState {
        match item {
            Side::White => EndState::WhiteWin,
            Side::Black => EndState::BlackWin,
        }
    }
}

pub trait Bitboard {
    fn bitscan_forward(self) -> usize;
    fn isolate_lsb(self) -> u64;
    fn negate(self) -> u64;
}

impl Bitboard for u64 {
    /// Determines the square index of an isolated bit
    fn bitscan_forward(self) -> usize {
        assert!(self != 0);
        self.tzcnt() as usize
    }
    fn isolate_lsb(self) -> u64 {
        self & self.wrapping_neg()
    }
    fn negate(self) -> u64 {
        self.wrapping_neg()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    White = 0,
    Black = 1,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match &self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum Piece {
    Empty = 0,
    WRabbit = 1,
    WCat,
    WDog,
    WHorse,
    WCamel,
    WElephant,
    BRabbit = 7,
    BCat,
    BDog,
    BHorse,
    BCamel,
    BElephant = 12,
}

impl From<Piece> for char {
    fn from(item: Piece) -> char {
        match item {
            Piece::Empty => ' ',
            Piece::WRabbit => 'R',
            Piece::WCat => 'C',
            Piece::WDog => 'D',
            Piece::WHorse => 'H',
            Piece::WCamel => 'M',
            Piece::WElephant => 'E',
            Piece::BRabbit => 'r',
            Piece::BCat => 'c',
            Piece::BDog => 'd',
            Piece::BHorse => 'h',
            Piece::BCamel => 'm',
            Piece::BElephant => 'e',
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ch = char::from(*self);
        write!(f, "{}", ch)
    }
}

#[derive(Clone, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl From<Direction> for char {
    fn from(item: Direction) -> char {
        match item {
            Direction::North => 'n',
            Direction::East => 'e',
            Direction::South => 's',
            Direction::West => 'w',
        }
    }
}

impl From<char> for Direction {
    fn from(item: char) -> Direction {
        match item {
            'n' => Direction::North,
            'e' => Direction::East,
            's' => Direction::South,
            'w' => Direction::West,
            _ => unimplemented!(),
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dir = char::from(*self);
        write!(f, "{}", dir)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Step {
    Move(Piece, u8, u8), // Piece Source Dest
    Push(Piece, u8, u8), // Piece Source Dest of Pushed Piece
    Place(Piece, u8),    // Piece Dest
    Remove(Piece, u8),   // Piece Square (due to trap)
    Pass,
}

impl Step {
    pub fn from_notation(input: &str) -> Step {
        let chars: Vec<_> = input.chars().collect();
        let piece = Piece::from_u8(piece_char_index(chars[0])).unwrap();
        let sq = alg_to_index(&chars[1..=2]).unwrap() as u8;
        if let Some(c) = chars.get(3) {
            if *c == 'x' {
                return Step::Remove(piece, sq);
            }
            let dir = Direction::from(*c);
            let dest = match dir {
                Direction::East => sq + 1,
                Direction::North => sq + 8,
                Direction::West => sq - 1,
                Direction::South => sq - 8,
            };
            Step::Move(piece, sq, dest)
        } else {
            // Place is the only 3 char possibility
            Step::Place(piece, sq)
        }
    }
}

impl fmt::Debug for Step {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <fmt::Display>::fmt(&self, f)
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Step::Move(p, source, dest) | Step::Push(p, source, dest) => {
                let piece = char::from(*p);
                let (col, row) = index_to_alg(*source);
                let dir = {
                    if dest > source {
                        if dest - source == 1 {
                            Direction::East
                        } else {
                            // 8
                            Direction::North
                        }
                    } else {
                        if source - dest == 1 {
                            Direction::West
                        } else {
                            // 8
                            Direction::South
                        }
                    }
                };
                write!(f, "{}{}{}{}", piece, col, row, dir)
            }
            Step::Place(p, sq) => {
                let piece = char::from(*p);
                let (col, row) = index_to_alg(*sq);
                write!(f, "{}{}{}", piece, col, row)
            }
            Step::Remove(p, sq) => {
                let piece = char::from(*p);
                let (col, row) = index_to_alg(*sq);
                write!(f, "{}{}{}x", piece, col, row)

            }
            Step::Pass => write!(f, "[Pass]"),
        }
    }
}

#[derive(Clone)]
pub struct Position {
    pub side: Side,
    pub steps_left: u8,
    pub placement: [u64; 2], // white, black bitboards
    pub bitboards: [u64; 13],
    pub last_step: Option<Step>,
    pub pieces: [Piece; 64],
    pub initial_hash: u64,
    pub current_hash: u64,
    pub opp_last: u64, // previous position
    pub my_last: u64,  // one previous to that
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.steps_left == other.steps_left
            && self.side == other.side
            && self.last_step == other.last_step
            && self.bitboards == other.bitboards
    }
}
impl Eq for Position {}

impl Hash for Position {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.current_hash.hash(state);
    }
}

impl Position {
    pub fn new(side: Side, steps_left: u8, bitboards: [u64; 13]) -> Position {
        let mut placement: [u64; 2] = [0, 0];
        let mut pieces: [Piece; 64] = [Piece::Empty; 64];
        for pix in 1..13 {
            if pix < Piece::BRabbit as usize {
                placement[0] |= bitboards[pix];
            } else {
                placement[1] |= bitboards[pix];
            }
            let mut bb = bitboards[pix];
            while bb != 0 {
                let piecebit = bb & !bb; // LSB
                bb ^= piecebit; // Set LSB to 0
                let pieceix = bb.bitscan_forward();
                assert!(pieces[pieceix] == Piece::Empty);
                pieces[pieceix] = Piece::from_u8(pix as u8).unwrap();
            }
        }
        let hash = compute_hash(&pieces, side);
        Position {
            side,
            steps_left,
            placement,
            bitboards,
            last_step: None,
            pieces,
            initial_hash: hash,
            current_hash: hash,
            opp_last: 0,
            my_last: 0,
        }
    }
    pub fn from_pieces(side: Side, steps_left: u8, pieces: [Piece; 64]) -> Position {
        let mut placement: [u64; 2] = [0, 0];
        let mut bitboards: [u64; 13] = [0; 13];
        let mut bit_index = 1;
        for piece in pieces.iter() {
            let piece_index = *piece as usize;
            bitboards[piece_index] |= bit_index;

            bit_index = bit_index << 1;
        }
        for i in 1..=6 {
            placement[0] |= bitboards[i];
            placement[1] |= bitboards[i + 6];
        }
        let hash = compute_hash(&pieces, side);
        Position {
            side,
            steps_left,
            placement,
            bitboards,
            last_step: None,
            pieces,
            initial_hash: hash,
            current_hash: hash,
            opp_last: 0,
            my_last: 0,
        }
    }
    pub fn from_small_notation(notation: String, side: Side) -> Result<Position, Error> {
        let mut pieces: [Piece; 64] = [Piece::Empty; 64];
        let vec: Vec<_> = notation
            .chars()
            .filter(|c| *c != '[' && *c != ']')
            .filter_map(|c| Piece::from_u8(piece_char_index(c)))
            .collect();
        ensure!(vec.len() == 64, format_err!("Wrong number of pieces!"));
        for (index, piece) in NotationIter::new().zip(vec.into_iter()) {
            pieces[index] = piece;
        }
        Ok(Self::from_pieces(side, 4, pieces))
    }
    pub fn to_small_notation(&self) -> String {
        let mut vec = Vec::with_capacity(66);
        vec.push('[');
        for i in NotationIter::new() {
            vec.push(char::from(self.pieces[i]))
        }
        vec.push(']');
        vec.into_iter().collect()
    }
    pub fn from_pos_notation(notation: String) -> Result<Position, Error> {
        let lines: Vec<_> = notation.lines().collect();
        // Todo read initial moves
        let mut turn_info: Vec<char> = lines[0]
            .split_whitespace()
            .next()
            .map(|s| s.chars().collect())
            .ok_or(format_err!("Invalid board notation"))?;
        let side = turn_info.pop().ok_or(format_err!("Invalid side to move"))?;
        let side = match side {
            'g' | 'w' => Side::White,
            's' | 'b' => Side::Black,
            _ => bail!("Invalid side to move"),
        };
        let mut pieces: [Piece; 64] = [Piece::Empty; 64];
        let mut index = 2 * alg_to_index(&['a', '8']).unwrap();
        for line_index in 2..10 {
            let iter = lines
                .get(line_index)
                .and_then(|x| x.split("|").nth(1))
                .ok_or(format_err!("Invalid board notation"))?
                .chars();
            for c in iter.take(16) {
                // Each has one extra whitespace
                let piece = Piece::from_u8(piece_char_index(c));
                //println!("{}", index);
                match piece {
                    Some(piece) => pieces[index / 2] = piece,
                    None => unreachable!(), // Wildcard char match returns 0
                }
                index += 1;
            }
            if index >= 32 {
                index -= 32;
            }
        }
        Ok(Self::from_pieces(side, 4, pieces))
    }
    pub fn to_pos_notation(&self) -> String {
        let top_bot = " +-----------------+\n";
        let end = "   a b c d e f g h\n";
        let mut rows = Vec::with_capacity(8);
        let mut ptr = alg_to_index(&['a', '8']).unwrap();
        for row_num in (1..=8).rev() {
            let r = &self.pieces[ptr..ptr + 8];
            let c_col = match r[2] {
                Piece::Empty if row_num == 6 || row_num == 3 => {
                    'x' // Trap
                }
                _ => r[2].into(),
            };
            let f_col = match r[5] {
                Piece::Empty if row_num == 6 || row_num == 3 => {
                    'x' // Trap
                }
                _ => r[5].into(),
            };
            let row_string = format!(
                "{}| {} {} {} {} {} {} {} {} |\n",
                row_num, r[0], r[1], c_col, r[3], r[4], f_col, r[6], r[7]
            );
            rows.push(row_string);
            if ptr >= 8 {
                ptr -= 8;
            }
        }
        format!(
            "7g Da1n Cc1n\n{}{}{}{}{}{}{}{}{}{}{}",
            top_bot,
            rows[0],
            rows[1],
            rows[2],
            rows[3],
            rows[4],
            rows[5],
            rows[6],
            rows[7],
            top_bot,
            end
        )
    }
    pub fn gen_steps(&self) -> Vec<Step> {
        let mut moves = Vec::new();
        let player_index = self.side as usize;
        let opp_index = (player_index + 1) % 2;
        // Compute frozen
        let wneighbors = neighbors_of(self.placement[0]);
        let bneighbors = neighbors_of(self.placement[1]);
        let mut wstronger = self.placement[0];
        let mut bstronger = self.placement[1];
        let mut frozen = 0;
        let mut stronger = [0; 6]; // For push and pull computation
        for pix in 1..7 {
            // These masks are stronger relative to the current piece
            wstronger ^= self.bitboards[pix];
            bstronger ^= self.bitboards[pix + 6];
            stronger[pix - 1] = wstronger | bstronger;
            frozen |= self.bitboards[pix] & neighbors_of(bstronger) & (!wneighbors);
            frozen |= self.bitboards[pix + 6] & neighbors_of(wstronger) & (!bneighbors);
        }
        match self.last_step {
            // Continue push
            Some(Step::Push(p, source, _dest)) => {
                let pushed_piece_id = p as u8;
                let colorless = pushed_piece_id as usize - (6 * opp_index);
                let followers = neighbors_of(index_to_lsb(source))
                    & stronger[colorless - 1]
                    & self.placement[player_index];
                for lsb in PieceIter::new(followers) {
                    let sq = lsb.bitscan_forward();
                    moves.push(Step::Move(self.pieces[sq], sq as u8, source));
                }
                return moves;
            }
            // Consider pull
            Some(Step::Move(p, source, _dest)) => {
                let puller_type = p as u8;
                let colorless = puller_type as usize - (6 * player_index);
                if colorless > 1 {
                    // is not rabbit
                    // Pulling piece must be strictly stronger, hence - 2

                    let candidates = {
                        assert!(colorless <= 6);
                        neighbors_of(index_to_lsb(source))
                            & self.placement[opp_index]
                            & !stronger[colorless - 2]
                    };
                    for lsb in PieceIter::new(candidates) {
                        let sq = lsb.bitscan_forward();
                        moves.push(Step::Move(self.pieces[sq], sq as u8, source));
                    }
                }
            }
            _ => {}
        }
        // Generate normal steps
        let active_pieces = self.placement[player_index] & !frozen;
        let iter = PieceIter::new(active_pieces);
        for lsb in iter {
            let is_rabbit = lsb & self.bitboards[1 + player_index * 6] != 0;
            let movements = {
                if is_rabbit {
                    rabbit_steps(self.side, lsb) & self.bitboards[0] // Empty adjacent
                } else {
                    neighbors_of(lsb) & self.bitboards[0] // Empty adjacent
                }
            };
            for p in PieceIter::new(movements) {
                let sq = lsb.bitscan_forward();
                moves.push(Step::Move(
                    self.pieces[sq],
                    sq as u8,
                    p.bitscan_forward() as u8,
                ));
            }
        }
        // Initiate pushes
        if self.steps_left > 1 {
            for pix in 1..7 {
                // To be pushed we must have a stronger opponent piece adjacent
                // to us, and we must have an empty tile adjacent to us
                let opp_pix = pix + 6 * opp_index;
                let pushing_candidates = stronger[pix - 1] & self.placement[player_index] & !frozen;
                let pushable = (neighbors_of(pushing_candidates) & self.bitboards[opp_pix])
                    & neighbors_of(self.bitboards[0]);
                let push_iter = PieceIter::new(pushable);
                for lsb in push_iter {
                    for target_lsb in PieceIter::new(neighbors_of(lsb) & self.bitboards[0]) {
                        let sq = lsb.bitscan_forward();
                        moves.push(Step::Push(
                            self.pieces[sq],
                            sq as u8,
                            target_lsb.bitscan_forward() as u8,
                        ));
                    }
                }
            }
        }
        // if self.steps_left != 4 {
        //     moves.push(Step::Pass); // Todo checking zobrist
        // }
        moves.push(Step::Pass);
        moves
    }
    pub fn do_step(&mut self, step: Step) -> EndState {
        // Todo finish
        let res = match step {
            Step::Move(p, source, dest) | Step::Push(p, source, dest) => {
                self.steps_left -= 1;
                let pix = p as usize;
                self.pieces[source as usize] = Piece::Empty;
                self.pieces[dest as usize] = p;
                let change = index_to_lsb(source) | index_to_lsb(dest);
                // xor out relevant changes
                self.bitboards[pix] ^= change;
                if pix <= 6 {
                    self.placement[0] ^= change;
                } else {
                    self.placement[1] ^= change;
                }
                self.bitboards[0] ^= change;
                // Check traps
                for (index, trap_sq) in TRAP_INDICES.iter().enumerate() {
                    let pix = self.pieces[*trap_sq] as usize;
                    let friendly_neighbors = {
                        if pix == 0 {
                            continue; // We don't care if trap is empty
                        } else if pix <= 6 {
                            let x = self.placement[0] & TRAP_NEIGHBORS[index];
                            //dbg!(x);
                            x
                        } else {
                            self.placement[1] & TRAP_NEIGHBORS[index]
                        }
                    };
                    if friendly_neighbors == 0 {
                        let next_step =
                            Step::Remove(Piece::from_u8(pix as u8).unwrap(), *trap_sq as u8);
                        self.do_step(next_step);
                        break; // Each step can affect one trap at most
                    }
                }
                self.current_hash = update_hash(self.current_hash, step);
                if self.steps_left == 0 {
                    self.end_turn()
                } else {
                    self.last_step = Some(step); // This is the only push case
                    EndState::Neither
                }
            }
            Step::Place(p, sq) => {
                // Todo figure out steps left and hash in the opening phase
                let pix = p as usize;
                self.pieces[sq as usize] = p;
                let change = index_to_lsb(sq);
                // xor out relevant changes
                self.bitboards[pix] ^= change;
                if pix <= 6 {
                    self.placement[0] ^= change;
                } else {
                    self.placement[1] ^= change;
                }
                self.bitboards[0] ^= change;
                EndState::Neither
            }
            Step::Remove(p, sq) => {
                let pix = p as usize;
                self.pieces[sq as usize] = Piece::Empty;
                let change = index_to_lsb(sq);
                // xor out relevant changes
                self.bitboards[pix] ^= change;
                if pix <= 6 {
                    self.placement[0] ^= change;
                } else {
                    self.placement[1] ^= change;
                }
                self.bitboards[0] ^= change;
                self.current_hash = update_hash(self.current_hash, step);
                EndState::Neither
            }
            Step::Pass => self.end_turn(),
        };
        res
    }
    pub fn end_turn(&mut self) -> EndState {
        if self.current_hash == self.initial_hash || self.current_hash == self.my_last {
            // Null move or repetition
            return self.side.opposite().into();
        }
        match self.side {
            Side::White => {
                if self.bitboards[1] & RANK_8 != 0 {
                    return EndState::WhiteWin;
                } else if self.bitboards[7] & RANK_1 != 0 {
                    return EndState::BlackWin;
                } else if self.bitboards[7] == 0 {
                    return EndState::WhiteWin;
                } else if self.bitboards[1] == 0 {
                    return EndState::BlackWin;
                }
            }
            Side::Black => {
                if self.bitboards[7] & RANK_1 != 0 {
                    return EndState::BlackWin;
                } else if self.bitboards[1] & RANK_8 != 0 {
                    return EndState::WhiteWin;
                } else if self.bitboards[1] == 0 {
                    return EndState::BlackWin;
                } else if self.bitboards[7] == 0 {
                    return EndState::WhiteWin;
                }
            }
        }
        self.steps_left = 4;
        self.my_last = self.opp_last;
        self.opp_last = self.initial_hash;
        self.side = self.side.opposite();
        self.current_hash ^= color_hash(self.side);
        self.initial_hash = self.current_hash;
        self.last_step = None;
        EndState::Neither
        // Todo repetitions
    }
    pub fn from_opening_str(opening: &str) -> Option<Position> {
        let lines: Vec<&str> = opening.lines().collect();
        let mut pieces = [Piece::Empty; 64];
        if lines.len() != 2 {
            return None;
        }
        for (index, side) in [Side::White, Side::Black].iter().enumerate() {
            let split = lines[index].split_whitespace();
            for alg in split {
                let chs: Vec<_> = alg.chars().collect();
                let val = piece_char_index(chs[0]);
                if let Side::White = side {
                    assert!(val < 7);
                } else {
                    assert!(val >= 7);
                }
                let index = match alg_to_index(&chs[1..]) {
                    Some(index) => index,
                    None => return None,
                };
                pieces[index] = Piece::from_u8(val).unwrap();
            }
        }
        let bitboards = match bitboards_from_pieces(&pieces) {
            Some(bb) => bb,
            None => return None,
        };
        let mut placement = [0, 0];
        placement[0] |=
            bitboards[1] | bitboards[2] | bitboards[3] | bitboards[4] | bitboards[5] | bitboards[6];
        placement[1] |= bitboards[7]
            | bitboards[8]
            | bitboards[9]
            | bitboards[10]
            | bitboards[11]
            | bitboards[12];
        let hash = compute_hash(&pieces, Side::White);
        Some(Position {
            side: Side::White,
            steps_left: 4,
            placement,
            bitboards,
            last_step: None,
            pieces,
            initial_hash: hash,
            current_hash: hash,
            opp_last: 0,
            my_last: 0,
        })
    }
}

pub fn total_moves() -> u32 {
    let mut grand_total = 1; // Initial pass move
    let mut per_square = Vec::new();
    let mut interior_count = 0;
    for x in 0..64 {
        let root: u64 = 1 << x;
        let mut total = 0;
        let mut root_neighbors = neighbors_of(root);
        let num_r_neighbors = root_neighbors.count_ones();
        total += num_r_neighbors; // Standard moves
        while root_neighbors != 0 {
            let lsb = root_neighbors.isolate_lsb();
            assert_eq!(lsb.count_ones(), 1);
            let num_neighbors = neighbors_of(lsb).count_ones() - 1; // Ignore root
            assert!(1 <= num_neighbors && num_neighbors <= 3);
            total += num_neighbors; // Pushes
            total += num_r_neighbors - 1; // Pulls
            root_neighbors &= root_neighbors - 1;
        }
        assert!(8 <= total && total <= 28);
        if total == 28 {
            interior_count += 1;
        }
        per_square.push(total);
        grand_total += total;
    }
    assert_eq!(interior_count, 4 * 4);
    println!("Each square {:?}", per_square);
    let mut cumulative = 0;
    for i in 0..per_square.len() {
        let temp = per_square[i];
        per_square[i] = cumulative;
        cumulative += temp;
    }
    println!("Cumulative Offsets {:?}", per_square);
    println!("Grand Total {}", grand_total);
    return grand_total;
}

/// An iterator over the individual bits of a bitboard
pub struct PieceIter {
    bitboard: u64,
}

impl PieceIter {
    pub fn new(bitboard: u64) -> PieceIter {
        PieceIter { bitboard }
    }
}

impl Iterator for PieceIter {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.bitboard == 0 {
            return None;
        }
        let lsb = self.bitboard.isolate_lsb();
        self.bitboard = self.bitboard ^ lsb;
        Some(lsb)
    }
}

pub struct NotationIter {
    index: usize,
}

impl NotationIter {
    pub fn new() -> NotationIter {
        NotationIter {
            index: 71, // Hack to wrap back to a8 first
        }
    }
}

impl Iterator for NotationIter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.index % 8 == 7 {
            if self.index == 7 {
                return None;
            }
            self.index -= 15;
        } else {
            self.index += 1;
        }
        Some(self.index)
    }
}

pub fn rabbit_steps(side: Side, lsb: u64) -> u64 {
    let mut out = (lsb & NOT_A_FILE) << 1;
    out |= (lsb & NOT_H_FILE) >> 1;
    let s = (side as u64) << 3;
    out |= lsb >> (s & 8); // If white
    out |= lsb << (!s & 8); // If black
    out
}

pub fn index_to_lsb(index: u8) -> u64 {
    1 << index
}

pub fn neighbors_of(lsb: u64) -> u64 {
    ((lsb & NOT_H_FILE) >> 1) | ((lsb & NOT_A_FILE) << 1) | (lsb >> 8) | (lsb << 8)
}

pub fn bitboards_from_pieces(pieces: &[Piece]) -> Option<[u64; 13]> {
    if pieces.len() != 64 {
        return None;
    }
    let mut bitboards = [0; 13];
    for (index, piece) in pieces.iter().enumerate() {
        let bit_index: u64 = 1 << index;
        bitboards[*piece as usize] |= bit_index;
    }
    Some(bitboards)
}

pub fn piece_char_index(piece: char) -> u8 {
    match piece {
        'R' => 1,
        'C' => 2,
        'D' => 3,
        'H' => 4,
        'M' => 5,
        'E' => 6,
        'r' => 7,
        'c' => 8,
        'd' => 9,
        'h' => 10,
        'm' => 11,
        'e' => 12,
        _ => 0, // Empty piece otherwise
    }
}

pub fn index_to_alg(index: u8) -> (char, u8) {
    let col = match index % 8 {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => unreachable!(),
    };
    let row = (index / 8) + 1; // 1 indexed
    (col, row)
}

pub fn alg_to_index(chs: &[char]) -> Option<usize> {
    if chs.len() != 2 {
        return None;
    }
    let col = match chs[0] {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => return None,
    };
    let row = match chs[1] {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        '5' => 4,
        '6' => 5,
        '7' => 6,
        '8' => 7,
        _ => return None,
    };
    Some(col + row * 8)
}
