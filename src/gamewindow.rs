use std::time::SystemTime;

use crate::{
    game::{
        Game, GameState::Playing, GAMEMAP_COLS, GAMEMAP_ROWS, LEVEL_TIMES, PIECE_SIZE, WIN_HEIGHT,
        WIN_MARGIN, WIN_WIDTH,
    },
    helpers::ColorFromU32,
    Result,
};

use sdl2::{
    image::{init, InitFlag},
    pixels::Color,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    ttf::Font,
    video::{Window, WindowContext},
    EventPump,
};

pub struct GameWindow<'a> {
    pub canvas: Canvas<Window>,
    pub tc: Option<&'a TextureCreator<WindowContext>>,
    pub event_pump: EventPump,
    pub step_timer: SystemTime,
    pub global_timer: SystemTime,
    pub width: u32,
    pub height: u32,
}

impl<'a> GameWindow<'a> {
    pub fn new() -> Result<GameWindow<'a>> {
        let sdl_context = sdl2::init()?;
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
            event_pump: sdl_context.event_pump()?,
            step_timer: SystemTime::now(),
            global_timer: SystemTime::now(),
            width: WIN_WIDTH,
            height: WIN_HEIGHT,
            tc: None,
        })
    }

    // creates sdl texture
    pub fn create_tex(&mut self, col: Color) -> Result<Texture<'a>> {
        let mut tex =
            self.tc
                .unwrap()
                .create_texture_target(None, PIECE_SIZE as u32, PIECE_SIZE as u32)?;

        self.canvas.with_texture_canvas(&mut tex, |texture| {
            texture.set_draw_color(col);
            texture.clear();
        })?;

        Ok(tex)
    }
    pub fn draw_background(&mut self) -> Result<()> {
        let grid_color = self.create_tex(Color::RGB(0, 0, 0))?;

        let border_color = self.create_tex(Color::RGB(255, 255, 255))?;

        static mut BKG_COLOR_R: i16 = 0;
        static mut BKG_COLOR_STEP: i16 = 1;

        unsafe {
            if BKG_COLOR_R == 216 || BKG_COLOR_R == -1 {
                BKG_COLOR_STEP = -BKG_COLOR_STEP;
                BKG_COLOR_R += BKG_COLOR_STEP;
            }

            self.canvas
                .set_draw_color(Color::RGB(BKG_COLOR_R as u8, 64, 255 - BKG_COLOR_R as u8));
            self.canvas.clear();
            BKG_COLOR_R += BKG_COLOR_STEP;
        }
        let (x, y, w, h) = (
            ((self.width - PIECE_SIZE * GAMEMAP_COLS as u32) / 2) as i32,
            ((self.height - PIECE_SIZE * GAMEMAP_ROWS as u32) / 2) as i32,
            PIECE_SIZE * GAMEMAP_COLS as u32,
            PIECE_SIZE * GAMEMAP_ROWS as u32,
        );

        self.draw_rect(
            x - WIN_MARGIN as i32,
            y - WIN_MARGIN as i32,
            w + WIN_MARGIN * 2,
            h + WIN_MARGIN * 2,
            &border_color,
        )?;

        self.draw_rect(x, y, w, h, &grid_color)?;

        Ok(())
    }
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: &Texture) -> Result<()> {
        self.canvas.copy(color, None, Rect::new(x, y, w, h))?;
        Ok(())
    }
    pub fn draw_tile(&mut self, x: i32, y: i32, color: &Texture) -> Result<()> {
        self.draw_rect(x, y, PIECE_SIZE as u32, PIECE_SIZE as u32, color)?;
        Ok(())
    }
    pub fn timer_tick(&self, level: u32) -> bool {
        match self.step_timer.elapsed() {
            Ok(elapsed) => {
                let millis = elapsed.as_secs() as u32 * 1000 + elapsed.subsec_millis();
                millis > LEVEL_TIMES[level as usize - 1]
            }
            Err(_) => false,
        }
    }

    pub fn display_text_line(
        &mut self,
        font: &Font,
        color: &Color,
        text: String,
        x: u32,
        y: u32,
    ) -> Result<Rect> {
        let (w, h) = font.size_of(text.as_str()).unwrap();

        let surface = font.render(text.as_str()).blended(*color)?;
        let tex = self.tc.unwrap().create_texture_from_surface(&surface)?;
        let r = Rect::new(x as i32, y as i32, w, h);
        self.canvas.copy(&tex, None, r)?;
        Ok(r)
    }

    pub fn display_game_information(
        &mut self,
        game: &Game,
        font: &Font,
        color_palette: [u32; 8],
    ) -> Result<()> {
        macro_rules! col {
            ($i: expr) => {
                &Color::fromu32(color_palette[$i])
            };
        }

        let mut y = 0;
        let h = self
            .display_text_line(font, col!(0), format!("Score: {}", game.score), 10, y)?
            .height()
            / 2;
        y += h;

        self.display_text_line(
            font,
            col!(1),
            format!("Lines cleared: {}", game.lines_cleared),
            10,
            y,
        )?;
        y += h;

        self.display_text_line(font, col!(2), format!("Level: {}", game.level), 10, y)?;
        y += h;

        let mut el = game.total_time_played;
        if matches!(game.current_state, Playing) {
            el += SystemTime::now()
                .duration_since(game.time_measure_start)
                .unwrap()
                .as_millis();
        }

        let millis = el % 1000;
        el /= 1000;
        let (secs, mins) = (el % 60, el / 60);
        let elapsed_text = format!("Time: {}:{:0>2}.{:0>3}", mins, secs, millis);

        self.display_text_line(font, col!(3), elapsed_text, 10, y)?;

        Ok(())
    }
}
