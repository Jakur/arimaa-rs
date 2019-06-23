
use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;
use std::fs::File;
use std::io::Write;

const NUM_COLORS: usize = 2;
const NUM_PIECES: usize = 13;
const NUM_SQUARES: usize = 64;

include!("table_zobrist.rs");

pub fn get_zobrist(sq: usize, piece: usize, color: usize) -> u64 {
    ZOBRIST_PIECES[sq][piece][color]
}

pub fn write_zobrist(f: &mut File) {
    let mut rng = SmallRng::seed_from_u64(0xDEADBEEF);

    write!(f, "const SIDE_TO_MOVE: u64 = {};\n\n", rng.next_u64()).unwrap();

    write!(
        f,
        "const ZOBRIST_PIECES: [[[u64; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS] = [[[\n"
    )
    .unwrap();
    for i in 0..NUM_COLORS {
        for j in 0..NUM_PIECES {
            for _ in 0..NUM_SQUARES {
                write!(f, "    {},\n", rng.next_u64()).unwrap();
            }
            if j != NUM_PIECES - 1 {
                write!(f, "   ], [\n").unwrap();
            }
        }
        if i != NUM_COLORS - 1 {
            write!(f, "  ]], [[\n").unwrap();
        }
    }
    write!(f, "]]];\n\n").unwrap();
}