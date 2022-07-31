extern crate sdl2;

use game::{Game, LEVEL_TIMES, PIECE_SIZE, WIN_HEIGHT, WIN_WIDTH};

use sdl2::event::Event;

use sdl2::image::{init, InitFlag};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
use std::error::Error;
use std::time::{Duration, SystemTime};

mod fileio;
mod game;
mod pieces;
// helper type for error propagation
type Result<T> = std::result::Result<T, Box<dyn Error>>;
// hide window mechanics internals tuff in a struct
struct GameWindow {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    timer: SystemTime,
    width: u32,
    height: u32,
}
// do ugly stuff

fn create_texture_rect<'a>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    r: u8,
    g: u8,
    b: u8,
    width: u32,
    height: u32,
) -> Option<Texture<'a>> {
    // We'll want to handle failures outside of this function.
    if let Ok(mut square_texture) = texture_creator.create_texture_target(None, width, height) {
        canvas
            .with_texture_canvas(&mut square_texture, |texture| {
                texture.set_draw_color(Color::RGB(r, g, b));
                texture.clear();
            })
            .expect("Failed to color a texture");
        Some(square_texture)
    } else {
        None
    }
}

fn prepare_window(width: u32, height: u32) -> Result<GameWindow> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video()?;
    init(InitFlag::JPG | InitFlag::PNG)?;

    let window = video_subsystem
        .window("rust-sdl2 demo", width, height)
        .position_centered()
        .opengl()
        .build()?;

    let canvas = window.into_canvas().build()?;
    let pump = sdl_context.event_pump()?;

    let timer = SystemTime::now();
    let gw = GameWindow {
        canvas,
        event_pump: pump,

        timer,
        width,
        height,
    };

    Ok(gw)
}
fn is_time_over(game: &Game, timer: &SystemTime) -> bool {
    match timer.elapsed() {
        Ok(elapsed) => {
            let millis = elapsed.as_secs() as u32 * 1000 + elapsed.subsec_millis();
            millis > LEVEL_TIMES[game.level as usize - 1]
        }
        Err(_) => false,
    }
}

pub fn main() -> Result<()> {
    let mut game = Game::new();

    let mut gw = prepare_window(WIN_WIDTH, WIN_HEIGHT)?;
    let mut i: u8 = 0;

    let grid_x = (gw.width - PIECE_SIZE as u32 * 10) as i32 / 2;
    let grid_y = (gw.height - PIECE_SIZE as u32 * 16) as i32 / 2;
    let texture_creator: TextureCreator<_> = gw.canvas.texture_creator();

    macro_rules! texture {
        ($r:expr, $g:expr, $b:expr) => {
            create_texture_rect(
                &mut gw.canvas,
                &texture_creator,
                $r,
                $g,
                $b,
                PIECE_SIZE as u32,
                PIECE_SIZE as u32,
            )
            .unwrap()
        };
    }

    let textures = [
        texture!(255, 69, 69),
        texture!(255, 220, 69),
        texture!(237, 150, 37),
        texture!(171, 99, 237),
        texture!(77, 149, 239),
        texture!(39, 218, 225),
        texture!(45, 216, 47),
    ];
    let grid = texture!(0, 0, 0);
    let border = texture!(255, 255, 255);
    'running: loop {
        i = (i + 1) % 255;
        gw.canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        gw.canvas.clear();

        gw.canvas.copy(
            &border,
            None,
            Rect::new(
                (gw.width - PIECE_SIZE as u32 * 10) as i32 / 2 - 10,
                (gw.height - PIECE_SIZE as u32 * 16) as i32 / 2 - 10,
                PIECE_SIZE as u32 * 10 + 20,
                PIECE_SIZE as u32 * 16 + 20,
            ),
        )?;
        gw.canvas.copy(
            &grid,
            None,
            Rect::new(
                (gw.width - PIECE_SIZE as u32 * 10) as i32 / 2,
                (gw.height - PIECE_SIZE as u32 * 16) as i32 / 2,
                PIECE_SIZE as u32 * 10,
                PIECE_SIZE as u32 * 16,
            ),
        )?;

        // if game.current_piece.is_none() {
        //     let current_piece = Piece::random_piece();
        //     if !game.test_current_position(&current_piece) {
        //         game.print_game_info();
        //         break 'running;
        //     }
        //     game.current_piece = Some(current_piece);
        //}

        let mut quit = false;
        if !handle_events(&mut game, &mut quit, &mut gw.timer, &mut gw.event_pump) {
            let piece = &game.piece;
            for (line_nb, line) in piece.shapes[piece.rot as usize].iter().enumerate() {
                for i in 0..4 {
                    if line & (1 << i) == 0 {
                        continue;
                    }
                    // The new part is here:
                    gw.canvas.copy(
                        &textures[piece.code as usize - 1],
                        None,
                        Rect::new(
                            grid_x + (piece.x + i as isize) as i32 * PIECE_SIZE as i32,
                            grid_y + (piece.y + line_nb) as i32 * PIECE_SIZE as i32,
                            PIECE_SIZE as u32,
                            PIECE_SIZE as u32,
                        ),
                    )?;
                }
            }
        }
        if quit {
            game.print_game_info();
            break 'running;
        }

        if is_time_over(&game, &gw.timer) {
            let x = game.piece.x;
            let y = game.piece.y + 1;

            if !game.change_piece_position(x, y) && !game.make_permanent() {
                game.print_game_info();
                break 'running;
            }
            gw.timer = SystemTime::now();
        }

        for (line_nb, line) in game.game_map.iter().enumerate() {
            for (case_nb, case) in line.iter().enumerate() {
                if *case == 0 {
                    continue;
                }
                gw.canvas.copy(
                    &textures[*case as usize - 1],
                    None,
                    Rect::new(
                        grid_x + case_nb as i32 * PIECE_SIZE as i32,
                        grid_y + line_nb as i32 * PIECE_SIZE as i32,
                        PIECE_SIZE as u32,
                        PIECE_SIZE as u32,
                    ),
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
    quit: &mut bool,
    timer: &mut SystemTime,
    event_pump: &mut sdl2::EventPump,
) -> bool {
    let mut make_permanent = false;

    let mut tmp_x = game.piece.x;
    let tmp_y = game.piece.y;
    'running: for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                *quit = true;
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
                tmp_x += 1;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => {
                tmp_x -= 1;
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
                let nx = game.piece.x;
                let mut ny = game.piece.y;
                while game.change_piece_position(nx, ny + 1) {
                    ny += 1;
                }
                make_permanent = true;
            }
            _ => {}
        }
    }
    if !make_permanent && !game.change_piece_position(tmp_x, tmp_y) && tmp_y != game.piece.y {
        make_permanent = true;
    }

    if make_permanent {
        *timer = SystemTime::now();
        return !game.make_permanent();
    }
    make_permanent
}
