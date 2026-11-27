use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::{Rgb565, RgbColor},
};

use crate::display::{textdraw::TextSize, Display, C_BLACK};

pub struct ClockView {
    pub is_24h: bool,
    frame_no: u32,
}

pub fn render_clock(display: &mut Display) {
    display.fill_screen(C_BLACK);
    let size = TextSize::Large;
    let char_style = MonoTextStyle::new(size.font(), Rgb565::WHITE);
    display.present();
}
