use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::fmt;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, trans::Trans)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

pub type Vec2I32 = Vec2<i32>;
pub type Vec2F32 = Vec2<f32>;

impl Vec2I32 {
    pub fn origin() -> Self {
        Self::from_i32(0, 0)
    }

    pub fn top_right_corner() -> Self {
        Self::from_i32(79, 79)
    }

    pub fn from_flat(idx: i32) -> Self {
        Self {
            x: idx / 80,
            y: idx % 80,
        }
    }
    pub fn from_i32(x: i32, y: i32) -> Self {
        Self{x, y}
    }

    pub fn add_xy(&self, delta: i32) -> Self {
        Self {
            x: self.x + delta,
            y: self.y + delta,
        }
    }

    pub fn add_x(&self, delta_x: i32) -> Self {
        Self {
            x: self.x + delta_x,
            y: self.y
        }
    }

    pub fn add_y(&self, delta_y: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + delta_y,
        }
    }

    pub fn mdist(&self, other: &Vec2I32) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl Vec2F32 {
    pub fn from_i32(x: i32, y: i32) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }

    pub fn from_f32(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
        }
    }

    pub fn add_xy(&self, delta: f32) -> Self {
        Self {
            x: self.x + delta,
            y: self.y + delta,
        }
    }

    pub fn length(&self) -> f32 {
        ((self.x.powi(2) + self.y.powi(2)) as f32).sqrt()
    }

    pub fn dot_product(&self, other: &Vec2F32) -> f32 {
        self.x * other.x + self.y * other.y
    }
    pub fn angle_with(&self, other: &Vec2F32) -> f32 {
        let cos_angle = self.dot_product(other) / (self.length() * other.length());

        cos_angle.acos()
    }
}


impl From<Vec2F32> for Vec2I32 {
    fn from(other: Vec2F32) -> Self {
        Self {
            x: other.x as i32,
            y: other.y as i32,
        }
    }
}

impl From<Vec2I32> for Vec2F32 {
    fn from(other: Vec2I32) -> Self {
        Self {
            x: other.x as f32,
            y: other.y as f32,
        }
    }
}

impl fmt::Display for Vec2I32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}


impl<T: Add<Output = T>> Add for Vec2<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl<T: AddAssign> AddAssign for Vec2<T> {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}
impl<T: Sub<Output = T>> Sub for Vec2<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl<T: SubAssign> SubAssign for Vec2<T> {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}
impl<T: Mul<Output = T> + Copy> Mul<T> for Vec2<T> {
    type Output = Self;
    fn mul(self, factor: T) -> Self::Output {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}
impl<T: MulAssign + Copy> MulAssign<T> for Vec2<T> {
    fn mul_assign(&mut self, factor: T) {
        self.x *= factor;
        self.y *= factor;
    }
}
impl<T: Div<Output = T> + Copy> Div<T> for Vec2<T> {
    type Output = Self;
    fn div(self, divisor: T) -> Self::Output {
        Self {
            x: self.x / divisor,
            y: self.y / divisor,
        }
    }
}
impl<T: DivAssign + Copy> DivAssign<T> for Vec2<T> {
    fn div_assign(&mut self, divisor: T) {
        self.x /= divisor;
        self.y /= divisor;
    }
}
