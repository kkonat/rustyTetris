extern crate sdl2;

use fileio::save_highscores_and_lines;

use helpers::ColorFromU32;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    render::{Texture, TextureCreator},
    ttf::FontStyle,
    video::WindowContext,
};

use std::time::{Duration, SystemTime};

use game::{
    Game,
    GameState::{self, Playing},
    GAMEMAP_COLS, GAMEMAP_ROWS, PIECE_SIZE,
};
use gamewindow::GameWindow;
use helpers::Result;

mod fileio;
mod game;
mod gamewindow;
mod helpers;
mod pieces;

pub fn main() -> Result<()> {
    let mut game = Game::new();
    let mut gw = GameWindow::new()?;
    let texture_creator: TextureCreator<WindowContext> = gw.canvas.texture_creator(); // texture creator must be created in
    gw.tc = Some(&texture_creator);

    let ttf_context = sdl2::ttf::init().expect("SDL TTF initialization failed");
    let mut font = ttf_context
        .load_font("assets/000webfont Regular.ttf", 32)
        .expect("Couldn't load the font");

    font.set_style(FontStyle::BOLD);

    let grid_x = ((gw.width - PIECE_SIZE as u32 * GAMEMAP_COLS as u32) / 2) as i32;
    let grid_y = ((gw.height - PIECE_SIZE as u32 * GAMEMAP_ROWS as u32) / 2) as i32;

    let color_palettes: [[u32; 8]; 3] = [
        // various color palettes to chose from
        [
            0xff6961, 0xfb480, 0xf8f38d, 0x42d6a4, 0x08cad1, 0x59adf6, 0x9d94ff, 0xc780e8,
        ],
        // toned down
        [
            0xd31e25, 0xd7a32e, 0xd1c02b, 0x369e4b, 0x5db5b7, 0x31407b, 0x8a3f64, 0x4f2e39,
        ],
        // vivid
        [
            0xff0000, 0xff8000, 0xffff00, 0x00ff00, 0x00ffff, 0x0000ff, 0x8000ff, 0x80ffff,
        ],
    ];

    // convert hex values to Vec of textures
    let texture_palette: Vec<_> = color_palettes[1]
        .iter()
        .map(|&val| gw.create_tex(Color::fromu32(val)).unwrap())
        .collect();
    let mut should_quit;

    'main_loop: loop {
        gw.draw_background()?;
        gw.display_game_information(&game, &font, color_palettes[2])?;
        draw_other_pieces(&game, &mut gw, grid_x, grid_y, &texture_palette)?;
        gw.display_state_info(&game.current_state, &font)?;
        (should_quit) = handle_events(&mut game, &mut gw);
        if game.piece.moves {
            draw_current_piece(&game.piece, &mut gw, grid_x, grid_y, &texture_palette)?;
        }

        if matches!(game.current_state, GameState::Playing)
            && gw.timer_tick(game.level)
            && matches!(game.current_state, Playing)
        {
            if !game.change_piece_position(0, 1) && !game.fix_piece() {
                // game over
                game.current_state = GameState::End;
                game.update_time();
                // gw.print_game_over();
            }
            gw.step_timer = SystemTime::now();
        }
        gw.canvas.present();

        if should_quit {
            game.print_game_info();
            break 'main_loop;
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    save_highscores_and_lines(&game.hiscores, &game.lines)?;
    Ok(())
}

fn handle_events(game: &mut Game, gw: &mut GameWindow) -> bool {
    let mut quit = false;

    let (mut dx, mut dy) = (0, 0);

    'running: for event in gw.event_pump.poll_iter() {
        // This is always active
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                quit = true;
                break 'running;
            }
            _ => {}
        }

        // Process keys active When playing
        match game.current_state {
            GameState::Playing => {
                match event {
                    // Pause
                    Event::KeyDown {
                        keycode: Some(Keycode::P),
                        ..
                    } => {
                        game.update_time();
                        game.current_state = GameState::Paused;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => {
                        gw.step_timer = SystemTime::now();
                        dy += 1;
                    }
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
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        // go as low as possible
                        while game.change_piece_position(0, 1) {
                            dy += 1;
                        }
                        gw.step_timer = SystemTime::now(); // resync timer
                        game.piece.moves = game.fix_piece();
                        return false;
                    }
                    _ => {}
                }
                if !game.change_piece_position(dx, dy) && dy != 0 {
                    game.piece.moves = false;
                }
            }
            GameState::Paused | GameState::Start => {
                if let Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } = event
                {
                    game.time_measure_start = SystemTime::now();
                    game.current_state = GameState::Playing;
                }
            }
            GameState::End => {
                *game = Game::new();
            }
        }
    }

    quit
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
