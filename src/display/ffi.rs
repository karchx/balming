#[repr(C)]
pub struct Hub75Config {
    pub r1: i8,
    pub g1: i8,
    pub b1: i8,
    pub r2: i8,
    pub g2: i8,
    pub b2: i8,
    pub a: i8,
    pub b: i8,
    pub c: i8,
    pub d: i8,
    pub e: i8,
    pub lat: i8,
    pub oe: i8,
    pub clk: i8,
    pub width: u16,
    pub height: u16,
    pub double_buff: bool,
}

unsafe extern "C" {
    pub fn begin(cfg: *const Hub75Config) -> bool;
}
