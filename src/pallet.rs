
use smart_leds::RGB8;

#[allow(dead_code)]
pub enum Colors {
    Black,
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
    White,
}

impl Colors {
    pub fn as_rgb(&self) -> RGB8 {
        match *self {
            Colors::Black => RGB8::new(0x00, 0x00, 0x00),
            Colors::Red => RGB8::new(0x3f, 0x00, 0x00),
            Colors::Orange => RGB8::new(0x3f, 0x1f, 0x00),
            Colors::Yellow => RGB8::new(0x3f, 0x3f, 0x00),
            Colors::Green => RGB8::new(0x00, 0x3f, 0x00),
            Colors::Cyan => RGB8::new(0x00, 0x3f, 0x3f),
            Colors::Blue => RGB8::new(0x00, 0x00, 0x3f),
            Colors::Magenta => RGB8::new(0x3f, 0x00, 0x3f),
            Colors::White => RGB8::new(0x3f, 0x3f, 0x3f),
        }
    }
}
