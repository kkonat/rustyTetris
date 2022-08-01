use crate::pieces::{Piece, PIECEWIDTH};
pub const WIN_WIDTH: u32 = 600;
pub const WIN_HEIGHT: u32 = 800;
pub const WIN_MARGIN: u32 = 20;
pub const PIECE_SIZE: u32 = 42;

const MAX_LEVELS: usize = 10;

pub const LEVEL_TIMES: [u32; MAX_LEVELS] = [1000, 850, 700, 600, 500, 400, 300, 250, 220, 190];
pub const LEVEL_LINES: [u32; MAX_LEVELS] = [20, 40, 60, 80, 100, 120, 140, 160, 180, 200];

pub const GAMEMAP_ROWS: usize = 16;
pub const GAMEMAP_COLS: usize = 10;

pub struct Game {
    pub game_map: Vec<Vec<u8>>,
    pub level: u32,
    pub score: u32,
    pub lines_cleared: u32,
    pub piece: Piece,
}
impl Game {
    pub fn new() -> Game {
        let mut gm = Vec::new();
        for _ in 0..GAMEMAP_ROWS {
            gm.push(vec![0; GAMEMAP_COLS]);
        }
        Game {
            game_map: gm,
            level: 1,
            score: 0,
            lines_cleared: 0,
            piece: Piece::random_piece(),
        }
    }

    fn compute_score(&mut self) {
        let mut y = 0;
        let mut incr_score = 0;

        while y < self.game_map.len() {
            let mut complete = true;
            for x in &self.game_map[y] {
                if *x == 0 {
                    complete = false;
                    break;
                }
            }
            if complete {
                incr_score += self.level;
                self.game_map.remove(y);
                y -= 1;
            }
            y += 1;
        }
        if self.game_map.is_empty() {
            incr_score += 1000;
        }
        self.score += incr_score;
        while self.game_map.len() < GAMEMAP_ROWS {
            self.lines_cleared += 1;

            if self.lines_cleared > LEVEL_LINES[self.level as usize - 1] {
                self.level += 1;
            }
            self.game_map.insert(0, vec![0; GAMEMAP_COLS]);
        }
    }

    // fixes piece on the game map, creates new piece, returns false if the new pece does not fit
    pub fn fix_piece(&mut self) -> bool {
        let mut score_incr = 0;
        let p = &self.piece;
        let mut shift_y = 0;
        while shift_y < p.shapes[p.rot as usize].len() && p.y + shift_y < GAMEMAP_ROWS {
            let mut shift_x = 0;
            while shift_x < PIECEWIDTH && (p.x + shift_x as isize) < GAMEMAP_COLS as isize {
                if p.shapes[p.rot as usize][shift_y] & (1 << shift_x) != 0 {
                    let x = p.x + shift_x as isize;
                    self.game_map[p.y + shift_y][x as usize] = p.code;
                }
                shift_x += 1;
            }
            shift_y += 1;
        }
        score_incr += self.level;

        self.score += score_incr;
        self.compute_score();
        self.piece = Piece::random_piece();

        self.test_position(None, None, None)
    }

    pub fn test_position(
        &self,
        r: Option<usize>,
        xoffs: Option<isize>,
        yoffs: Option<usize>,
    ) -> bool {
        let (mut tmp_x, mut tmp_y, mut tmp_rot) = (self.piece.x, self.piece.y, self.piece.rot);

        if let Some(..) = xoffs {
            tmp_x = xoffs.unwrap();
        }
        if let Some(..) = yoffs {
            tmp_y = yoffs.unwrap();
        }
        if let Some(..) = r {
            tmp_rot = r.unwrap();
        }

        let p = &self.piece;
        for decal_y in 0..p.shapes[tmp_rot].len() {
            for decal_x in 0..4 {
                let x = tmp_x + decal_x;

                if p.shapes[tmp_rot][decal_y] & (1 << decal_x) != 0 {
                    if tmp_y + decal_y >= GAMEMAP_ROWS || x < 0 || x as usize >= GAMEMAP_COLS {
                        return false;
                    }
                    if self.game_map[tmp_y + decal_y][x as usize] != 0 {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn rotate_piece(&mut self) {
        let mut tmp_rot = self.piece.rot + 1;
        if tmp_rot as usize >= self.piece.shapes.len() {
            tmp_rot = 0;
        }
        let x_offset = [0, -1, 1, -2, 2, -3];
        for x in x_offset.iter() {
            if self.test_position(Some(tmp_rot), None, None) {
                let p = &mut self.piece;
                p.rot = tmp_rot;
                p.x += *x;
                break;
            }
        }
    }

    pub fn change_piece_position(&mut self, dx: isize, dy: usize) -> bool {
        let nx = self.piece.x + dx;
        let ny = self.piece.y + dy;
        if self.test_position(None, Some(nx), Some(ny)) {
            self.piece.x = nx;
            self.piece.y = ny;
            true
        } else {
            false
        }
    }

    pub fn print_game_info(&mut self) {
        println!("Game over...");
        println!("Score: {}", self.score);
        println!("Number of lines: {}", self.lines_cleared);
        println!("Current level: {}", self.level);
    }
}
