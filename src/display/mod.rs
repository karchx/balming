use esp_idf_hal::{
    delay::Ets,
    gpio::{PinDriver, Pins},
};

pub fn test_red_screen(pins: Pins) -> anyhow::Result<()> {
    let mut r1 = PinDriver::output(pins.gpio4)?;
    let mut g1 = PinDriver::output(pins.gpio5)?;
    let mut b1 = PinDriver::output(pins.gpio6)?;
    let mut r2 = PinDriver::output(pins.gpio7)?;
    let mut g2 = PinDriver::output(pins.gpio15)?;
    let mut b2 = PinDriver::output(pins.gpio16)?;

    let mut a = PinDriver::output(pins.gpio17)?;
    let mut b = PinDriver::output(pins.gpio18)?;
    let mut c = PinDriver::output(pins.gpio8)?;
    let mut d = PinDriver::output(pins.gpio9)?;
    let mut e = PinDriver::output(pins.gpio10)?;

    let mut clk = PinDriver::output(pins.gpio11)?;
    let mut lat = PinDriver::output(pins.gpio12)?;
    let mut oe = PinDriver::output(pins.gpio13)?;

    r1.set_high()?;
    r2.set_high()?;
    g1.set_low()?;
    g2.set_low()?;
    b1.set_low()?;
    b2.set_low()?;
    oe.set_high()?;

    loop {
        for row in 0u8..32 {
            for _ in 0..64 {
                clk.set_high()?;
                clk.set_low()?;
            }

            oe.set_high()?;

            if (row & 0x01) != 0 {
                a.set_high()?;
            } else {
                a.set_low()?;
            }
            if (row & 0x02) != 0 {
                b.set_high()?;
            } else {
                b.set_low()?;
            }
            if (row & 0x04) != 0 {
                c.set_high()?;
            } else {
                c.set_low()?;
            }
            if (row & 0x08) != 0 {
                d.set_high()?;
            } else {
                d.set_low()?;
            }
            if (row & 0x10) != 0 {
                e.set_high()?;
            } else {
                e.set_low()?;
            }

            lat.set_high()?;
            lat.set_low()?;
            oe.set_low()?;

            Ets::delay_us(50);
        }
    }
}
