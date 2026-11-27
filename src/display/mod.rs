use crate::display::{
    capture::Capture,
    lang_data::{tk, TextKey},
    textdraw::TextSize,
};
use anyhow::bail;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{IntoStorage, Rgb565, RgbColor},
    Pixel,
};
use esp_idf_hal::cpu::Core;
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use std::{
    convert::Infallible,
    sync::mpsc::{self, Receiver, Sender},
};

pub mod capture;
pub mod ffi;
pub mod lang_data;
pub mod scene;
pub mod textdraw;

// helpers
pub const fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    (((r as u16) & 0xF8) << 8) | (((g as u16) & 0xFC) << 3) | ((b as u16) >> 3)
}

pub const C_BLACK: u16 = 0x0000;
pub const C_RED: u16 = rgb565(255, 0, 0);
// end helpers

pub enum DisplayCmd {
    TestRed,
    Clock,
}

pub struct Display {
    double_buff: bool,
    pub capture: Capture,
}

impl Display {
    pub fn new(double_buff: bool) -> anyhow::Result<Self> {
        let width = 64;
        let height = 64;
        let cfg = ffi::Hub75Config {
            r1: 4,
            g1: 5,
            b1: 6,
            r2: 7,
            g2: 15,
            b2: 16,
            a: 17,
            b: 18,
            c: 8,
            d: 9,
            e: 10,
            clk: 11,
            lat: 12,
            oe: 13,
            width: width,
            height: height,
            double_buff,
            shift_driver: 0, // PANEL_DRIVERS
            latch_blanking: 2,
            clkphase: false,
            color_depth: 8,
            i2s_hz: 16000000, // 16MHz
            min_refresh_rate: 100,
        };

        let ok = unsafe { ffi::begin(&cfg) };
        if !ok {
            bail!("Failed init HUB75 DMA");
        }

        Ok(Self {
            double_buff,
            capture: Capture::new(width, height),
        })
    }

    pub fn draw_pixel(&mut self, x: i16, y: i16, color: u16) {
        unsafe { ffi::draw_pixel(x, y, color) };
        self.capture.write_pixel(x, y, color);
    }

    pub fn fill_screen(&mut self, color: u16) {
        unsafe {
            ffi::fill_screen(color);
        }
    }

    pub fn set_brightness8(&mut self, level: u8) {
        unsafe {
            ffi::set_brightness8(level);
        }
    }

    pub fn present(&mut self) {
        if self.double_buff {
            unsafe { ffi::flip_dma_buffer() };
        }
    }

    pub fn refresh_rate(&self) -> i32 {
        unsafe { ffi::get_refresh_rate() }
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size::new(64, 64)
    }
}

impl DrawTarget for Display {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0 && coord.y >= 0 && coord.x < 64 && coord.y < 64 {
                self.draw_pixel(coord.x as i16, coord.y as i16, color.into_storage());
            }
        }
        Ok(())
    }
}

pub fn spawn_display_worker(double_buff: bool) -> anyhow::Result<Sender<DisplayCmd>> {
    let (tx, rx): (Sender<DisplayCmd>, Receiver<DisplayCmd>) = mpsc::channel();

    ThreadSpawnConfiguration {
        name: Some(c"hub75_worker"),
        stack_size: 8192,
        priority: 5,
        pin_to_core: Some(Core::Core1),
        ..Default::default()
    }
    .set()?;

    std::thread::spawn(move || {
        let mut display = match Display::new(double_buff) {
            Ok(d) => d,
            Err(e) => {
                log::error!("Error initialize HUB75: {:?}", e);
                return;
            }
        };

        log::info!("[panel] {} Hz scan", display.refresh_rate());
        display.set_brightness8(100);
        display.fill_screen(C_BLACK);
        textdraw::draw_text(
            &mut display,
            1,
            12,
            tk(TextKey::PanellHello),
            TextSize::Small,
            Rgb565::WHITE,
        );
        display.present();

        while let Ok(cmd) = rx.recv() {
            match cmd {
                DisplayCmd::TestRed => {
                    log::info!("Display: Initialize test_red_screen");
                    display.fill_screen(C_RED);
                    display.present();
                }
                DisplayCmd::Clock => {
                    log::info!("Display: Stopped");
                }
            }
        }
    });

    ThreadSpawnConfiguration::default().set()?;
    Ok(tx)
}
