#[derive(Copy, Clone, Debug, trans::Trans)]
pub struct ColorF32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorF32 {
    pub fn alpha(&self, new_a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: new_a,
        }
    }
}
