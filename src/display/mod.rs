use anyhow::bail;

pub mod capture;
pub mod ffi;

pub struct Display {
    double_buff: bool,
}

impl Display {
    pub fn new(double_buff: bool) -> anyhow::Result<Self> {
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
            width: 64,
            height: 64,
            double_buff,
        };

        let ok = unsafe { ffi::begin(&cfg) };
        if !ok {
            bail!("Failed init HUB75 DMA");
        }

        Ok(Self { double_buff })
    }
}
