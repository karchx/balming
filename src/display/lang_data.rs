#[derive(Clone, Copy)]
pub enum TextKey {
    PanellHello,
}

pub const fn tk(key: TextKey) -> &'static str {
    match key {
        TextKey::PanellHello => "HELLO",
    }
}
