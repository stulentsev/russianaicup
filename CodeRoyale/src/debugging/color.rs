use super::*;

/// RGBA Color
#[derive(Clone, Debug)]
pub struct Color {
    /// Red component
    pub r: f64,
    /// Green component
    pub g: f64,
    /// Blue component
    pub b: f64,
    /// Alpha (opacity) component
    pub a: f64,
}

impl trans::Trans for Color {
    fn write_to(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.r.write_to(writer)?;
        self.g.write_to(writer)?;
        self.b.write_to(writer)?;
        self.a.write_to(writer)?;
        Ok(())
    }
    fn read_from(reader: &mut dyn std::io::Read) -> std::io::Result<Self> {
        let r: f64 = trans::Trans::read_from(reader)?;
        let g: f64 = trans::Trans::read_from(reader)?;
        let b: f64 = trans::Trans::read_from(reader)?;
        let a: f64 = trans::Trans::read_from(reader)?;
        Ok(Self { r, g, b, a })
    }
}

impl Color {
    pub fn red() -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    pub fn green() -> Self {
        Self {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }

    pub fn blue() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }

    pub fn a(&self, val: f64) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: val,
        }
    }
}