#[derive(Debug)]
pub struct Color(pub f32, pub f32, pub f32, pub u8);

impl Color {
    pub fn from_hex(hex: &str) -> Option<Color> {
        let palette_index = match hex {
            "7EED56" => 0,
            "BE0039" => 1,
            "FF3881" => 2,
            "515252" => 3,
            "00CCC0" => 4,
            "FFD635" => 5,
            "811E9F" => 6,
            "FFA800" => 7,
            "D4D7D9" => 8,
            "DE107F" => 9,
            "FFB470" => 10,
            "94B3FF" => 11,
            "FFF8B8" => 12,
            "6D001A" => 13,
            "00756F" => 14,
            "3690EA" => 15,
            "B44AC0" => 16,
            "FF99AA" => 17,
            "FFFFFF" => 18,
            "6A5CFF" => 19,
            "898D90" => 20,
            "00A368" => 21,
            "9C6926" => 22,
            "6D482F" => 23,
            "000000" => 24,
            "FF4500" => 25,
            "51E9F4" => 26,
            "493AC1" => 27,
            "009EAA" => 28,
            "E4ABFF" => 29,
            "2450A4" => 30,
            "00CC78" => 31,
            _ => return None,
        };
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(Color(r, g, b, palette_index))
    }
}
