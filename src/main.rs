extern crate sdl2;

use game::{Game, PIECE_SIZE};

use sdl2::event::Event;

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use std::error::Error;
use std::time::{Duration, SystemTime};

use crate::gamewindow::GameWindow;

mod fileio;
mod game;
mod gamewindow;
mod pieces;

// helper type for error propagation
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
// hide window mechanics internals tuff in a struct

// do ugly stuff

fn create_texture_rect<'a>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    col: Color,
    width: u32,
    height: u32,
) -> Result<Texture<'a>> {
    let mut rect_texture = texture_creator.create_texture_target(None, width, height)?;
    canvas.with_texture_canvas(&mut rect_texture, |texture| {
        texture.set_draw_color(col);
        texture.clear();
    })?;
    Ok(rect_texture)
}

pub fn main() -> Result<()> {
    let mut game = Game::new();
    let mut gw = GameWindow::new()?;

    let grid_x = ((gw.width - PIECE_SIZE as u32 * 10) / 2) as i32;
    let grid_y = ((gw.height - PIECE_SIZE as u32 * 16) / 2) as i32;
    let texture_creator: TextureCreator<_> = gw.canvas.texture_creator();

    macro_rules! rgb {
        ($r:expr, $g:expr, $b:expr) => {
            create_texture_rect(
                &mut gw.canvas,
                &texture_creator,
                Color::RGB($r, $g, $b),
                PIECE_SIZE as u32,
                PIECE_SIZE as u32,
            )
            .unwrap()
        };
    }

    let color_palette = [
        rgb!(181, 2, 2),
        rgb!(207, 173, 25),
        rgb!(232, 127, 14),
        rgb!(145, 69, 213),
        rgb!(90, 159, 242),
        rgb!(15, 209, 247),
        rgb!(7, 236, 10),
    ];

    'main_loop: loop {
        gw.draw_background()?;

        let (should_quit, can_move) = handle_events(&mut game, &mut gw.timer, &mut gw.event_pump);
        if can_move {
            let piece = &game.piece;
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
        }
        if should_quit {
            game.print_game_info();
            break 'main_loop;
        }

        if gw.is_time_over(game.level) {
            if !game.change_piece_position(0, 1) && !game.fix_piece() {
                game.print_game_info();
                break 'main_loop;
            }
            gw.timer = SystemTime::now();
        }

        for (line_nb, line) in game.game_map.iter().enumerate() {
            for (case_nb, case) in line.iter().enumerate() {
                if *case == 0 {
                    continue;
                }

                gw.draw_tile(
                    grid_x + case_nb as i32 * PIECE_SIZE as i32,
                    grid_y + line_nb as i32 * PIECE_SIZE as i32,
                    &color_palette[*case as usize - 1],
                )?;
            }
        }

        gw.canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    //    save_highscores_and_lines(hiscores, 1);
    Ok(())
}

fn handle_events(
    game: &mut Game,

    timer: &mut SystemTime,
    event_pump: &mut sdl2::EventPump,
) -> (bool, bool) {
    let mut should_fix = false;
    let mut can_move = true;
    let mut quit = false;

    let (mut dx, mut dy) = (0, 0);

    'running: for event in event_pump.poll_iter() {
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
                should_fix = true;
                while game.change_piece_position(0, 1) {
                    dy += 1;
                }
            }
            _ => {}
        }
    }
    if should_fix {
        *timer = SystemTime::now();
        can_move = game.fix_piece();
    } else if !game.change_piece_position(dx, dy) && dy != 0 {
        can_move = false;
    }

    (quit, can_move)
}
