use core::fmt::Write;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::MonoTextStyle,
    pixelcolor::{Rgb565, RgbColor},
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::{Baseline, Text, TextStyleBuilder},
};
use heapless::String;

use super::Display;
use crate::{
    config::defaults::{PANEL_H, PANEL_W},
    display::textdraw::TextSize,
};

pub struct ClockView {
    pub is_24h: bool,
    frame_no: u32,
}

impl ClockView {
    pub fn new(is_24h: bool) -> Self {
        Self {
            is_24h,
            frame_no: 0,
        }
    }

    pub fn render(&mut self, display: &mut Display, hour: u8, min: u8, sec: u8) {
        let _ = display.clear(Rgb565::BLACK);

        let hv = if self.is_24h {
            hour
        } else {
            let h = hour % 12;
            if h == 0 {
                12
            } else {
                h
            }
        };

        let mut hs = String::<4>::new();
        let mut ms = String::<4>::new();
        if self.is_24h {
            let _ = write!(hs, "{:02}", hv);
        } else {
            let _ = write!(hs, "{}", hv);
        }

        let _ = write!(ms, "{:02}", min);

        let hw = (hs.len() as i32) * 10;
        let colon_w = 6;
        let mw = 20;
        let xoff = ((PANEL_W as i32 - (hw + colon_w + mw)) / 2).max(0);
        let yoff = (PANEL_H as i32 - 20) / 2 - 2;

        let char_style = MonoTextStyle::new(TextSize::LargeX.font(), Rgb565::WHITE);
        let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

        // Hour and minutes
        let _ = Text::with_text_style(&hs, Point::new(xoff, yoff), char_style, text_style)
            .draw(display);
        let _ = Text::with_text_style(
            &ms,
            Point::new(xoff + hw + colon_w, yoff),
            char_style,
            text_style,
        )
        .draw(display);

        let colon_color = if sec % 2 == 1 {
            Rgb565::WHITE
        } else {
            Rgb565::new(15, 17, 12)
        };
        let colon_style = PrimitiveStyle::with_fill(colon_color);
        let cx = xoff + hw + 2;
        let _ = Rectangle::new(Point::new(cx, yoff + 5), Size::new(2, 2))
            .into_styled(colon_style)
            .draw(display);
        let _ = Rectangle::new(Point::new(cx, yoff + 13), Size::new(2, 2))
            .into_styled(colon_style)
            .draw(display);

        // Seconds
        let comet_y = yoff + 24;
        if sec > 0 {
            let trail_color = Rgb565::new(13, 13, 10);
            let _ = Line::new(Point::new(2, comet_y), Point::new(2 + sec as i32, comet_y))
                .into_styled(PrimitiveStyle::with_stroke(trail_color, 1))
                .draw(display);
        }

        let head_color = if (self.frame_no / 3) % 2 == 1 {
            Rgb565::CYAN
        } else {
            Rgb565::WHITE
        };
        let _ = Pixel(Point::new(2 + sec as i32, comet_y), head_color).draw(display);
        self.frame_no = self.frame_no.wrapping_add(1);
    }
}
