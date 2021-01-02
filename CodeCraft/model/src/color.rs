#[derive(Clone, Copy, Debug, Default, trans::Trans)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[allow(clippy::many_single_char_names)]
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let c = v * s;
        let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = if (0.0..60.0).contains(&h) {
            (c, x, 0.0)
        } else if (60.0..120.0).contains(&h) {
            (x, c, 0.0)
        } else if (120.0..180.0).contains(&h) {
            (0.0, c, x)
        } else if (180.0..240.0).contains(&h) {
            (0.0, x, c)
        } else if (240.0..300.0).contains(&h) {
            (x, 0.0, c)
        } else if (300.0..360.0).contains(&h) {
            (c, 0.0, x)
        } else {
            unreachable!("invalid value of h: {}", h)
        };


        // println!("h={}, s={}, v={}, c={}, x={}, m={}, r1={}, g1={}, b1={}", h, s, v, c, x, m, r1, g1, b1);
        Self {
            r: r1 + m,
            g: g1 + m,
            b: b1 + m,
            a: 1.0,
        }
    }

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

    pub fn yellow() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }

    pub fn purple() -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }

    pub fn set_a(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }
}
