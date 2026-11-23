use esp_idf_hal::cpu::Core;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::{Output, PinDriver, Pins};
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use std::sync::mpsc::{self, Receiver, Sender};

pub enum DisplayCmd {
    TestRed,
    Stop,
}

pub struct Hub75Pins<'d> {
    r1: PinDriver<'d, Output>,
    g1: PinDriver<'d, Output>,
    b1: PinDriver<'d, Output>,
    r2: PinDriver<'d, Output>,
    g2: PinDriver<'d, Output>,
    b2: PinDriver<'d, Output>,
    a: PinDriver<'d, Output>,
    b: PinDriver<'d, Output>,
    c: PinDriver<'d, Output>,
    d: PinDriver<'d, Output>,
    e: PinDriver<'d, Output>,
    clk: PinDriver<'d, Output>,
    lat: PinDriver<'d, Output>,
    oe: PinDriver<'d, Output>,
}

impl<'d> Hub75Pins<'d> {
    pub fn new(pins: Pins) -> anyhow::Result<Self> {
        let mut oe = PinDriver::output(pins.gpio13)?;
        oe.set_high()?; // Panel apagado al inicio (OE es activo en bajo)

        Ok(Self {
            r1: PinDriver::output(pins.gpio4)?,
            g1: PinDriver::output(pins.gpio5)?,
            b1: PinDriver::output(pins.gpio6)?,
            r2: PinDriver::output(pins.gpio7)?,
            g2: PinDriver::output(pins.gpio15)?,
            b2: PinDriver::output(pins.gpio16)?,
            a: PinDriver::output(pins.gpio17)?,
            b: PinDriver::output(pins.gpio18)?,
            c: PinDriver::output(pins.gpio8)?,
            d: PinDriver::output(pins.gpio9)?,
            e: PinDriver::output(pins.gpio10)?,
            clk: PinDriver::output(pins.gpio11)?,
            lat: PinDriver::output(pins.gpio12)?,
            oe,
        })
    }

    fn set_red_channels(&mut self) -> anyhow::Result<()> {
        self.r1.set_high()?;
        self.r2.set_high()?;
        self.g1.set_low()?;
        self.g2.set_low()?;
        self.b1.set_low()?;
        self.b2.set_low()?;
        Ok(())
    }

    fn render_red_frame(&mut self) -> anyhow::Result<()> {
        for row in 0u8..32 {
            for _ in 0..64 {
                self.clk.set_high()?;
                self.clk.set_low()?;
            }
            self.oe.set_high()?;

            self.a.set_level(((row & 0x01) != 0).into())?;
            self.b.set_level(((row & 0x02) != 0).into())?;
            self.c.set_level(((row & 0x04) != 0).into())?;
            self.d.set_level(((row & 0x08) != 0).into())?;
            self.e.set_level(((row & 0x10) != 0).into())?;

            self.lat.set_high()?;
            self.lat.set_low()?;
            self.oe.set_low()?;

            Ets::delay_us(50);
        }
        Ok(())
    }

    fn blank(&mut self) -> anyhow::Result<()> {
        self.oe.set_high()?;
        Ok(())
    }
}

pub fn spawn_display_worker(pins: Pins) -> anyhow::Result<Sender<DisplayCmd>> {
    let (tx, rx): (Sender<DisplayCmd>, Receiver<DisplayCmd>) = mpsc::channel();

    ThreadSpawnConfiguration {
        name: Some(c"hub75_worker"),
        stack_size: 4096,
        priority: 5,
        pin_to_core: Some(Core::Core1),
        ..Default::default()
    }
    .set()?;

    std::thread::spawn(move || {
        let mut display = match Hub75Pins::new(pins) {
            Ok(d) => d,
            Err(e) => {
                log::error!("Error initialize HUB75: {:?}", e);
                return;
            }
        };

        let mut running_test = false;
        let mut frame_counter: u32 = 0;

        loop {
            let cmd = if running_test {
                rx.try_recv().ok()
            } else {
                rx.recv().ok()
            };

            if let Some(c) = cmd {
                match c {
                    DisplayCmd::TestRed => {
                        log::info!("Display: Initialize test_red_screen");
                        let _ = display.set_red_channels();
                        running_test = true;
                    }
                    DisplayCmd::Stop => {
                        log::info!("Display: Stopped");
                        let _ = display.blank();
                        running_test = false;
                    }
                }
            }

            if running_test {
                if let Err(e) = display.render_red_frame() {
                    log::error!("Error render frame: {:?}", e);
                    running_test = false;
                    continue;
                }

                frame_counter = frame_counter.wrapping_add(1);
                if frame_counter % 60 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        }
    });

    ThreadSpawnConfiguration::default().set()?;
    Ok(tx)
}
