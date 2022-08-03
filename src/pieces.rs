use crate::game::GAMEMAP_COLS;

pub struct Piece {
    pub shapes: Vec<Vec<u8>>,
    pub x: isize,
    pub y: usize,
    pub rot: usize,
    pub code: u8,
    pub moves: bool,
}
const PIECETYPES: &str = "ILJOSZT";
const NOPIECETYPES: usize = 7;
pub const PIECEWIDTH: usize = 4;

impl Default for Piece {
    fn default() -> Piece {
        Piece {
            shapes: vec![vec![0]],
            x: (GAMEMAP_COLS) as isize / 2,
            y: 0,
            code: 0,
            rot: 0,
            moves: true,
        }
    }
}
impl Piece {
    pub fn new(pt: char) -> Piece {
        match pt {
            'I' => Piece {
                shapes: vec![vec![0b1111], vec![0b0100, 0b0100, 0b0100, 0b0100]],
                code: 1,
                ..Default::default()
            },
            'L' => Piece {
                shapes: vec![
                    vec![0b111, 0b100, 0b000],
                    vec![0b110, 0b010, 0b010],
                    vec![0b001, 0b111, 0b000],
                    vec![0b100, 0b100, 0b110],
                ],
                code: 2,
                ..Default::default()
            },
            'J' => Piece {
                shapes: vec![
                    vec![0b111, 0b001, 0b000],
                    vec![0b010, 0b010, 0b110],
                    vec![0b100, 0b111, 0b000],
                    vec![0b110, 0b100, 0b100],
                ],
                code: 3,
                ..Default::default()
            },
            'O' => Piece {
                shapes: vec![vec![0b11, 0b11]],
                code: 4,
                ..Default::default()
            },
            'S' => Piece {
                shapes: vec![vec![0b011, 0b110], vec![0b010, 0b011, 0b001]],
                code: 5,
                ..Default::default()
            },
            'Z' => Piece {
                shapes: vec![vec![0b110, 0b011], vec![0b001, 0b011, 0b010]],
                code: 6,
                ..Default::default()
            },
            'T' => Piece {
                shapes: vec![
                    vec![0b111, 0b010],
                    vec![0b010, 0b110, 0b010],
                    vec![0b010, 0b111],
                    vec![0b010, 0b011, 0b010],
                ],
                code: 7,
                ..Default::default()
            },
            _ => unreachable!(),
        }
    }
    pub fn random_piece() -> Piece {
        static mut PREV: usize = NOPIECETYPES;
        let mut rand_nb = rand::random::<usize>() % NOPIECETYPES;
        unsafe {
            if PREV == rand_nb {
                rand_nb = (rand_nb + 1) % NOPIECETYPES;
            }
            PREV = rand_nb;
        }
        Piece::new(PIECETYPES.as_bytes()[rand_nb] as char)
    }
}
