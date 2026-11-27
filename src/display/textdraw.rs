use embedded_graphics::{
    geometry::Point,
    mono_font::{
        ascii::{FONT_10X20, FONT_5X7, FONT_6X10, FONT_9X15},
        MonoFont, MonoTextStyle,
    },
    pixelcolor::Rgb565,
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};

use super::Display;

#[derive(Clone, Copy)]
pub enum TextSize {
    Small, // 1 in GFX
    Medium,
    Large, // 2 in GFX
    LargeX,
}

impl TextSize {
    pub fn font(self) -> &'static MonoFont<'static> {
        match self {
            TextSize::Small => &FONT_5X7,
            TextSize::Medium => &FONT_6X10,
            TextSize::Large => &FONT_9X15,
            TextSize::LargeX => &FONT_10X20,
        }
    }
}

pub fn draw_text(
    display: &mut Display,
    x: i32,
    y: i32,
    text: &str,
    size: TextSize,
    color: Rgb565,
) -> i32 {
    let char_style = MonoTextStyle::new(size.font(), color);
    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let next_point = Text::with_text_style(text, Point::new(x, y), char_style, text_style)
        .draw(display)
        .unwrap();

    next_point.x
}
