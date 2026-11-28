use core::fmt::Write;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::MonoTextStyle,
    pixelcolor::{Rgb565, RgbColor},
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::{Baseline, Text, TextStyleBuilder},
};
use heapless::String;

use super::Display;
use crate::{
    config::defaults::{PANEL_H, PANEL_W},
    display::{color565, textdraw::TextSize},
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

    pub fn draw_ambient(&mut self, display: &mut Display, hour: u8, min: u8, sec: u8) {
        const TOP: i32 = 22;
        const BOT: i32 = 45;
        let mins = (hour as i32) * 60 + (min as i32);
        let day = mins >= 360 && mins < 1110;
        let night = hour >= 19 || hour < 5;

        const SUN: Rgb565 = color565(255, 214, 90);
        const SUNST: Rgb565 = color565(255, 150, 110); // sunset orange
        const MOON: Rgb565 = color565(225, 228, 240);
        const STAR: Rgb565 = color565(190, 200, 230);
        const CATC: Rgb565 = color565(255, 170, 190);
        const ZZZ: Rgb565 = color565(150, 160, 200);

        let mut cel_x = 0;
        if day {
            let p: f32 = (mins as f32 - 360.0) / 750.0;
            let sx: i32 = 3 + ((p * 57.0) as i32);
            let sy: i32 = BOT - 3 - ((p * std::f32::consts::PI).sin() * 14.0) as i32;
            cel_x = sx;

            let color = if mins > 1020 { SUNST } else { SUN };
            let _ = Circle::with_center(Point::new(sx, sy), 7)
                .into_styled(PrimitiveStyle::with_fill(color))
                .draw(display);

            let ray = (self.frame_no / 8) % 2; // gentle ray shimmer
            if ray == 1 {
                let sun_raw = SUN.into_storage();
                display.draw_pixel(sx as i16, (sy - 5) as i16, sun_raw);
                display.draw_pixel((sx - 5) as i16, sy as i16, sun_raw);
                display.draw_pixel((sx + 5) as i16, sy as i16, sun_raw);
                display.draw_pixel(sx as i16, (sy + 5) as i16, sun_raw);
            }
        } else {
            let nm: i32 = if mins > 1140 { mins - 1140 } else { mins + 300 };
            let p: f32 = nm as f32 / 660.0;
            let mx: i32 = 3 + ((p * 57.0) as i32);
            let my: i32 = BOT - 3 - ((p * std::f32::consts::PI).sin() * 14.0) as i32;
            cel_x = mx;
            let _ = Circle::with_center(Point::new(mx, my), 7)
                .into_styled(PrimitiveStyle::with_fill(MOON))
                .draw(display);

            let _ = Circle::with_center(Point::new(mx + 2, my - 1), 7)
                .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
                .draw(display);
        }

        if night {
            const SXS: [i16; 12] = [5, 12, 20, 27, 34, 41, 48, 56, 60, 16, 37, 52];
            const SYS: [i16; 12] = [23, 28, 22, 32, 25, 36, 24, 30, 38, 40, 41, 35];
            let star_raw = STAR.into_storage();

            for i in 0..12 {
                let n = ((self.frame_no / 5) + (i as u32) * 3) % 7;
                if n < 4 {
                    display.draw_pixel(SXS[i], SYS[i], star_raw);
                }
            }
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
        let colon_w = 5;
        let mw = 24;
        let xoff = ((PANEL_W as i32 - (hw + colon_w + mw)) / 2).max(0);
        let yoff = 2;

        let char_style = MonoTextStyle::new(TextSize::LargeX.font(), Rgb565::GREEN);
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
            Rgb565::GREEN
        } else {
            Rgb565::new(0, 18, 0)
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
        let comet_y = yoff + 18;
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
        self.draw_ambient(display, hour, min, sec);
        self.frame_no = self.frame_no.wrapping_add(1);
    }
}
