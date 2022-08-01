use std::time::SystemTime;

use crate::{
    game::{
        GAMEMAP_COLS, GAMEMAP_ROWS, LEVEL_TIMES, PIECE_SIZE, WIN_HEIGHT, WIN_MARGIN, WIN_WIDTH,
    },
    Result,
};

use sdl2::{
    image::{init, InitFlag},
    pixels::Color,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    video::Window,
    video::WindowContext,
    EventPump,
};

pub struct GameWindow {
    pub canvas: Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    pub event_pump: EventPump,
    pub timer: SystemTime,
    pub width: u32,
    pub height: u32,
}

impl GameWindow {
    pub fn new() -> Result<GameWindow> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video()?;
        init(InitFlag::JPG | InitFlag::PNG)?;
        let canvas = video_subsystem
            .window("rust Tetris", WIN_WIDTH, WIN_HEIGHT)
            .position_centered()
            .opengl()
            .build()?
            .into_canvas()
            .build()?;

        Ok(GameWindow {
            canvas,
            texture_creator: &canvas.texture_creator(),
            event_pump: sdl_context.event_pump()?,
            timer: SystemTime::now(),
            width: WIN_WIDTH,
            height: WIN_HEIGHT,
        })
    }

    fn create_texture_rect<'a>(
        &mut self,

        col: Color,
        width: u32,
        height: u32,
    ) -> Result<Texture<'a>> {
        let mut rect_texture = self
            .texture_creator
            .create_texture_target(None, width, height)?;
        self.canvas
            .with_texture_canvas(&mut rect_texture, |texture| {
                texture.set_draw_color(col);
                texture.clear();
            })?;

        Ok(rect_texture)
    }

    pub fn draw_background(&mut self) -> Result<()> {
        macro_rules! rgb_tex {
            ($r:expr, $g:expr, $b:expr) => {
                self.create_texture_rect(
                    &self.texture_creator,
                    Color::RGB($r, $g, $b),
                    PIECE_SIZE as u32,
                    PIECE_SIZE as u32,
                )
                .unwrap()
            };
        }
        let grid_color = self.rgb_tex!(0, 0, 0);
        let border_color = rgb_tex!(255, 255, 255);
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
