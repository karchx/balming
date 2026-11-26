use std::time::Instant;

pub struct Capture {
    width: usize,
    height: usize,
    armed: bool,
    shadow_back: Vec<u16>,
    shadow_front: Vec<u16>,
    pub frame_count: u32,
    last_read_at: Instant,
}

impl Capture {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width: width as usize,
            height: height as usize,
            armed: false,
            shadow_back: Vec::new(),
            shadow_front: Vec::new(),
            frame_count: 0,
            last_read_at: Instant::now(),
        }
    }

    #[inline]
    pub fn write_pixel(&mut self, x: i16, y: i16, color: u16) {
        if !self.armed || x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x < self.width && y < self.height {
            self.shadow_back[y * self.width + x] = color;
        }
    }
}
