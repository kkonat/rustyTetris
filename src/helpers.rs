use std::error::Error;

use sdl2::pixels::Color;

// helper type for error propagation
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub trait ColorFromU32 {
    fn fromu32(val: u32) -> Color;
}
impl ColorFromU32 for Color {
    fn fromu32(val: u32) -> Color {
        Color::RGB(
            (val >> 16) as u8,
            ((val & 0xff00) >> 8) as u8,
            (val & 0xff) as u8,
        )
    }
}
