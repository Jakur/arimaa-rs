use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

const ALL_BITS_SET: u64 = 0xFFFFFFFFFFFFFFFF;

const A_FILE: u64 = 0x8080808080808080;
const B_FILE: u64 = 0x4040404040404040;
const G_FILE: u64 = 0x0202020202020202;
const H_FILE: u64 = 0x0101010101010101;
const NOT_A_FILE: u64 = !A_FILE;
const NOT_H_FILE: u64 = !H_FILE;
const RANK_1: u64 = 0xFF;
const RANK_2: u64 = 0xFF00;
const RANK_7: u64 = 0xFF000000000000;
const RANK_8: u64 = 0xFF00000000000000;
const NOT_RANK_1: u64 = !RANK_1;
const NOT_RANK_8: u64 = !RANK_8;
const NOT_EDGE: u64 = NOT_A_FILE & NOT_H_FILE & NOT_RANK_1 & NOT_RANK_8;

const TRAPS: u64 = 0x0000240000240000;
const TRAP_C3: u64 = 0x200000;
const TRAP_F3: u64 = 0x40000;
const TRAP_C6: u64 = 0x200000000000;
const TRAP_F6: u64 = 0x40000000000;
const TRAP_F3_IX: u8 = 18;
const TRAP_C3_IX: u8 = 21;
const TRAP_F6_IX: u8 = 42;
const TRAP_C6_IX: u8 = 45;

const index64: [usize; 64] = [
    0, 47, 1, 56, 48, 27, 2, 60, 57, 49, 41, 37, 28, 16, 3, 61, 54, 58, 35, 52, 50, 42, 21, 44, 38,
    32, 29, 23, 17, 11, 4, 62, 46, 55, 26, 59, 40, 36, 15, 53, 34, 51, 20, 43, 31, 22, 10, 45, 25,
    39, 14, 33, 19, 30, 9, 24, 13, 18, 8, 12, 7, 6, 5, 63,
]; // Used for bitscanning

pub fn bitScanForward(bitboard: u64) -> usize {
    const DEBRUIJIN64: u64 = 0x03f79d71b4cb0a89;
    assert!(bitboard != 0);
    index64[(((bitboard ^ (bitboard - 1)) * DEBRUIJIN64) >> 58) as usize]
}

pub enum Side {
    White = 0,
    Black = 1,
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

pub enum Step {
    Move(u8, u8, u8), // Source(0-63), Change(0-3), Displacement(0-4)?
    Pass,
}

pub struct Position {
    pub side: Side,
    pub steps_left: i32,
    pub frozen: u64,
    pub placement: [u64; 2], // white, black bitboards
    pub bitboards: [u64; 13],
    pub pieces: [Piece; 64],
}

impl Position {
    pub fn new(side: Side, steps_left: i32, bitboards: [u64; 13]) -> Position {
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
                let pieceix = bitScanForward(bb);
                assert!(pieces[pieceix] == Piece::Empty);
                pieces[pieceix] = Piece::from_usize(pix).unwrap();
            }
        }

        Position {
            side,
            steps_left,
            frozen: 0, // Todo fix
            placement,
            bitboards,
            pieces,
        }
    }
    pub fn gen_moves(&self) -> Vec<Step> {
        let mut moves = Vec::new();
        // Compute frozen
        let wneighbors = neighbors_of(self.placement[0]);
        let bneighbors = neighbors_of(self.placement[1]);
        let mut wstronger = self.placement[0];
        let mut bstronger = self.placement[1];
        let mut frozen = 0;
        for pix in 1..7 {
            // These masks are stronger relative to the current piece
            wstronger ^= self.bitboards[pix];
            bstronger ^= self.bitboards[pix + 6];
            frozen |= self.bitboards[pix] & neighbors_of(bstronger) & (!wneighbors);
            frozen |= self.bitboards[pix + 6] & neighbors_of(wstronger) & (!bneighbors);
        }
        moves // Todo complete
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
                let mut val = match chs[0] {
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
                    _ => return None,
                };
                if let Side::White = side {
                    assert!(val < 7);
                } else {
                    assert!(val >= 7);
                }
                let index = match alg_to_index(&chs[1..]) {
                    Some(index) => index,
                    None => return None,
                };
                pieces[index] = Piece::from_usize(val).unwrap();
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
        Some(Position {
            side: Side::White,
            steps_left: 4,
            frozen: 0,
            placement,
            bitboards,
            pieces,
        })
    }
}

fn rabbit_steps(side: Side, index: u64) -> u64 {
    let mut out = (index & NOT_A_FILE) << 1;
    out |= (index & NOT_H_FILE) >> 1;
    let s = side as u64;
    0
}
pub fn neighbors_of(index: u64) -> u64 {
    ((index & NOT_H_FILE) >> 1) | ((index & NOT_A_FILE) << 1) | (index >> 8) | (index << 8)
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
