use crate::color::Color;

#[derive(Debug)]
pub struct Entry {
    pub color: Color,
    pub pos: (u16, u16),
}

impl Entry {
    pub fn from_line(line: &str) -> Option<Self> {
        let mut parts = line.split(',').skip(2);
        let color = parts.next()?;
        let pos1 = parts.next()?;
        let pos2 = parts.next()?;

        let color = Color::from_hex(&color[1..])?;

        let x = pos1[1..].parse().ok()?;
        let y = pos2[..pos2.len() - 1].parse().ok()?;
        let pos = (x, y);

        if parts.next().is_some() {
            return None;
        }

        Some(Self { color, pos })
    }
}
