use crate::game::{LEVEL_TIMES, PIECE_SIZE, WIN_HEIGHT, WIN_WIDTH};
use std::{error::Error, time::SystemTime};

use crate::game::{GAMEMAP_COLS, GAMEMAP_ROWS, WIN_MARGIN};
use sdl2::{
    image::{init, InitFlag},
    pixels::Color,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    video::Window,
    video::WindowContext,
    EventPump,
};

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

pub struct GameWindow {
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
    pub timer: SystemTime,
    pub width: u32,
    pub height: u32,
}
type Result<T> = std::result::Result<T, Box<dyn Error>>;
impl GameWindow {
    pub fn new() -> Result<GameWindow> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video()?;
        init(InitFlag::JPG | InitFlag::PNG)?;

        let window = video_subsystem
            .window("rust Tetris", WIN_WIDTH, WIN_HEIGHT)
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
            width: WIN_WIDTH,
            height: WIN_HEIGHT,
        };

        Ok(gw)
    }
    pub fn draw_background(&mut self) -> Result<()> {
        let texture_creator: TextureCreator<_> = self.canvas.texture_creator();
        macro_rules! rgb {
            ($r:expr, $g:expr, $b:expr) => {
                create_texture_rect(
                    &mut self.canvas,
                    &texture_creator,
                    Color::RGB($r, $g, $b),
                    PIECE_SIZE as u32,
                    PIECE_SIZE as u32,
                )
                .unwrap()
            };
        }
        let grid_color = rgb!(0, 0, 0);
        let border_color = rgb!(255, 255, 255);
        static mut BKG_COLOR_R: u8 = 0;
        unsafe {
            self.canvas
                .set_draw_color(Color::RGB(BKG_COLOR_R, 64, 255 - BKG_COLOR_R));
            self.canvas.clear();

            BKG_COLOR_R = (BKG_COLOR_R + 1) % 255;
        }

        self.canvas.copy(
            &border_color,
            None,
            Rect::new(
                ((self.width - PIECE_SIZE * 10) / 2 - WIN_MARGIN) as i32,
                ((self.height - PIECE_SIZE * 16) / 2 - WIN_MARGIN) as i32,
                PIECE_SIZE * GAMEMAP_COLS as u32 + WIN_MARGIN * 2,
                PIECE_SIZE * GAMEMAP_ROWS as u32 + WIN_MARGIN * 2,
            ),
        )?;
        self.canvas.copy(
            &grid_color,
            None,
            Rect::new(
                ((self.width - PIECE_SIZE * 10) / 2) as i32,
                ((self.height - PIECE_SIZE * 16) / 2) as i32,
                PIECE_SIZE * GAMEMAP_COLS as u32,
                PIECE_SIZE * GAMEMAP_ROWS as u32,
            ),
        )?;
        Ok(())
    }
    pub fn draw_tile(&mut self, x: i32, y: i32, color: &Texture) -> Result<()> {
        self.canvas.copy(
            color,
            None,
            Rect::new(x, y, PIECE_SIZE as u32, PIECE_SIZE as u32),
        )?;
        Ok(())
    }
    pub fn is_time_over(&self, level: u32) -> bool {
        match self.timer.elapsed() {
            Ok(elapsed) => {
                let millis = elapsed.as_secs() as u32 * 1000 + elapsed.subsec_millis();
                millis > LEVEL_TIMES[level as usize - 1]
            }
            Err(_) => false,
        }
    }
}
