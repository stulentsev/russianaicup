use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, trans::Trans)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

pub type Vec2F32 = Vec2<f32>;
pub type Vec2F64 = Vec2<f64>;

impl Vec2F64 {
    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn dot_product(&self, other: &Vec2F64) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn angle_with(&self, other: &Vec2F64) -> f64 {
        let cos_angle = self.dot_product(other) / (self.length() * other.length());

        cos_angle.acos()
    }

    pub fn angle(&self) -> f64 {
        (self.x / self.length()).acos()
    }

    pub fn rotate(&self, new_angle: f64) -> Self {
        Self::from_length_and_angle(self.length(), self.angle() - new_angle)
    }

    pub fn from_length_and_angle(length: f64, angle: f64) -> Self {
        Self {
            x: length * angle.cos(),
            y: length * angle.sin(),
        }
    }


    pub fn add_x(&self, delta_x: f64) -> Self {
        Self {
            x: self.x + delta_x,
            y: self.y,
        }
    }

    pub fn add_y(&self, delta_y: f64) -> Self {
        Self {
            x: self.x,
            y: self.y + delta_y,
        }
    }
}

impl From<Vec2F32> for Vec2F64 {
    fn from(v: Vec2F32) -> Vec2F64 {
        Vec2F64 {
            x: v.x as f64,
            y: v.y as f64,
        }
    }
}
impl From<Vec2F64> for Vec2F32 {
    fn from(v: Vec2F64) -> Vec2F32 {
        Vec2F32 {
            x: v.x as f32,
            y: v.y as f32,
        }
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
