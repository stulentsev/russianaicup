use core::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use super::*;

/// 2 dimensional vector.
#[derive(Clone, Debug)]
pub struct Vec2 {
    /// `x` coordinate of the vector
    pub x: f64,
    /// `y` coordinate of the vector
    pub y: f64,
}

impl Vec2 {
    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        self.sub(other).length()
    }
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
    pub fn mul(&self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        }
    }
}

impl trans::Trans for Vec2 {
    fn write_to(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.x.write_to(writer)?;
        self.y.write_to(writer)?;
        Ok(())
    }
    fn read_from(reader: &mut dyn std::io::Read) -> std::io::Result<Self> {
        let x: f64 = trans::Trans::read_from(reader)?;
        let y: f64 = trans::Trans::read_from(reader)?;
        Ok(Self {
            x,
            y,
        })
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}


impl Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}