use super::*;
use core::fmt;
use std::f64::consts::PI;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// 2 dimensional vector.
#[derive(Clone, Debug, Copy)]
pub struct Vec2 {
    /// `x` coordinate of the vector
    pub x: f64,
    /// `y` coordinate of the vector
    pub y: f64,
}

impl Vec2 {
    pub fn approx_equal(&self, other: Vec2) -> bool {
        let factor = 10.0f64.powi(9);
        let x1 = (self.x * factor).trunc();
        let y1 = (self.y * factor).trunc();

        let x2 = (other.x * factor).trunc();
        let y2 = (other.y * factor).trunc();

        x1 == x2 && y1 == y2
    }

    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }


    // if( cross == -0.0f )
    // turn around
    //
    // if( cross == 0.0f )
    // already traveling the right direction
    //
    // if( cross < 0.0f)
    // turn left
    //
    // if( cross > 0.0f)
    // turn right
    pub fn cross_product(&self, other: &Self) -> f64 {
        return self.x * other.y - self.y * other.x;
    }

    pub fn dot_product(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        Self {
            x: self.x / len,
            y: self.y / len,
        }
    }

    pub fn angle_with(&self, other: &Self) -> f64 {
        let cos_angle = self.dot_product(other) / (self.length() * other.length());

        cos_angle.acos()
    }

    pub fn angle(&self) -> f64 {
        // const angle = Math.atan2(vec.y, vec.x);
        // return angle < 0 ? Math.PI * 2 + angle : angle;
        let res = (self.x / self.length()).acos();
        if self.y >= 0.0 {
            res
        } else {
            PI * 2.0 - res
        }
    }

    pub fn rotate(&self, angle: f64) -> Self {
        let (sin, cos) = f64::sin_cos(angle);
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    pub fn clamp(self, max_len: f64) -> Self {
        let len = self.length();
        if len > max_len {
            self * max_len / len
        } else {
            self
        }
    }

    pub fn clamp_min(self, min_len: f64) -> Self {
        let len = self.length();
        if len < min_len {
            self * min_len / len
        } else {
            self
        }
    }

    pub fn arg(self) -> f64 {
        f64::atan2(self.y, self.x)
    }

    pub fn from_length_and_angle(length: f64, angle: f64) -> Self {
        Self {
            x: length * angle.cos(),
            y: length * angle.sin(),
        }
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        (*other - *self).length()
    }
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn add_x(&self, val: f64) -> Self {
        Self {
            x: self.x + val,
            y: self.y,
        }
    }

    pub fn add_y(&self, val: f64) -> Self {
        Self {
            x: self.x,
            y: self.y + val,
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
        Self { x: 0.0, y: 0.0 }
    }

    pub fn from_xy(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
        }
    }

    pub fn max_speed(&self) -> Self {
        *self * 10.0
    }

    pub fn to_short_string(&self) -> String {
        format!("({:.2},{:.2})", self.x, self.y)
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
        Ok(Self { x, y })
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

impl Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, factor: f64) -> Self::Output {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, factor: f64) {
        self.x *= factor;
        self.y *= factor;
    }
}

impl Div<f64> for Vec2 {
    type Output = Self;
    fn div(self, factor: f64) -> Self::Output {
        Self {
            x: self.x / factor,
            y: self.y / factor,
        }
    }
}

impl DivAssign<f64> for Vec2 {
    fn div_assign(&mut self, factor: f64) {
        self.x /= factor;
        self.y /= factor;
    }
}