extern crate sdl2;

use sdl2::render::Texture;
use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, render::TextureCreator, video::WindowContext,
};

use std::error::Error;
use std::time::{Duration, SystemTime};

use crate::gamewindow::GameWindow;
use game::{Game, GAMEMAP_COLS, GAMEMAP_ROWS, PIECE_SIZE};

mod fileio;
mod game;
mod gamewindow;
mod pieces;

// helper type for error propagation
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn main() -> Result<()> {
    let mut game = Game::new();
    let mut gw = GameWindow::new()?;
    let texture_creator: TextureCreator<WindowContext> = gw.canvas.texture_creator(); // texture creator must be created in
    gw.tc = Some(&texture_creator);

    let grid_x = ((gw.width - PIECE_SIZE as u32 * GAMEMAP_COLS as u32) / 2) as i32;
    let grid_y = ((gw.height - PIECE_SIZE as u32 * GAMEMAP_ROWS as u32) / 2) as i32;

    let palette: [u32; 8] = [
        // various color palettes to chose from
        //  0xff6961, 0xfb480, 0xf8f38d, 0x42d6a4, 0x08cad1, 0x59adf6, 0x9d94ff, 0xc780e8,

        // toned down
        0xd31e25, 0xd7a32e, 0xd1c02b, 0x369e4b, 0x5db5b7, 0x31407b, 0x8a3f64,
        0x4f2e39,
        // vivid
        //0xff0000, 0xff8000, 0xffff00, 0x00ff00, 0x00ffff, 0x0000ff, 0x8000ff, 0x80ffff,
    ];

    // convert hex values to Vec of textures
    let color_palette: Vec<_> = palette
        .iter()
        .map(|&val| {
            gw.create_tex(Color::RGB(
                (val >> 16) as u8,
                ((val & 0xff00) >> 8) as u8,
                (val & 0xff) as u8,
            ))
            .unwrap()
        })
        .collect();

    'main_loop: loop {
        gw.draw_background()?;

        let (should_quit, can_move) = handle_events(&mut game, &mut gw);

        if can_move {
            // draw current piece
            let piece = &game.piece;
            draw_current_piece(piece, &mut gw, grid_x, grid_y, &color_palette)?;
        }

        if gw.timer_tick(game.level) {
            if !game.change_piece_position(0, 1) && !game.fix_piece() {
                // game over
                game.print_game_info();
                break 'main_loop;
            }
            gw.timer = SystemTime::now();
        }

        draw_other_pieces(&game, &mut gw, grid_x, grid_y, &color_palette)?;

        if should_quit {
            game.print_game_info();
            break 'main_loop;
        }
        gw.canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    //    save_highscores_and_lines(hiscores, 1);
    Ok(())
}

fn draw_current_piece(
    piece: &pieces::Piece,
    gw: &mut GameWindow,
    grid_x: i32,
    grid_y: i32,
    color_palette: &[Texture],
) -> Result<()> {
    for (line_nb, line) in piece.shapes[piece.rot as usize].iter().enumerate() {
        for i in 0..4 {
            if line & (1 << i) == 0 {
                continue;
            }

            gw.draw_tile(
                grid_x + (piece.x + i as isize) as i32 * PIECE_SIZE as i32,
                grid_y + (piece.y + line_nb) as i32 * PIECE_SIZE as i32,
                &color_palette[piece.code as usize - 1],
            )?;
        }
    }
    Ok(())
}

fn draw_other_pieces(
    game: &Game,
    gw: &mut GameWindow,
    grid_x: i32,
    grid_y: i32,
    color_palette: &[Texture],
) -> Result<()> {
    for (line_nb, line) in game.game_map.iter().enumerate() {
        for (case_nb, case) in line.iter().enumerate() {
            if *case != 0 {
                gw.draw_tile(
                    grid_x + case_nb as i32 * PIECE_SIZE as i32,
                    grid_y + line_nb as i32 * PIECE_SIZE as i32,
                    &color_palette[*case as usize - 1],
                )?;
            }
        }
    }
    Ok(())
}

fn handle_events(game: &mut Game, gw: &mut GameWindow) -> (bool, bool) {
    let mut can_move = true;
    let mut quit = false;
    let (mut dx, mut dy) = (0, 0);

    'running: for event in gw.event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                quit = true;
                break 'running;
            }
            // Event::KeyDown {
            //     keycode: Some(Keycode::Down),
            //     ..
            // } => {
            //     *timer = SystemTime::now();
            //     tmp_y += 1;
            // }
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => {
                dx = 1;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => {
                dx = -1;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => {
                game.rotate_piece();
            }
            Event::KeyDown {
                keycode: Some(Keycode::Space),
                ..
            }
            | Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => {
                // go as low as possible
                while game.change_piece_position(0, 1) {
                    dy += 1;
                }
                gw.timer = SystemTime::now(); // resync timer
                can_move = game.fix_piece();
                return (false, can_move);
            }
            _ => {}
        }
    }
    if !game.change_piece_position(dx, dy) && dy != 0 {
        can_move = false;
    }

    (quit, can_move)
}
