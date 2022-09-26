use ai_cup_22::model::*;
use crate::MyStrategy;

pub fn calculate_circle_line(center: Vec2, radius: f64, start: Vec2, end: Vec2) -> Option<Vec2> {
    closest_intersection_point(radius, start - center, end - center)
        .iter()
        .map(|p| *p + center)
        .filter(|p| p.x >= start.x && p.x <= end.x)
        .filter(|p| p.y >= start.y && p.y <= end.y)
        .next()
}

// https://e-maxx.ru/algo/segment_to_line
// https://e-maxx.ru/algo/circle_line_intersection
pub fn closest_intersection_point(radius: f64, start: Vec2, end: Vec2) -> Vec<Vec2> {
    let a = start.y - end.y;
    let b = end.x - start.x;
    let c = -a * start.x - b * start.y;

    let r = radius;

    let a2b2 = a * a + b * b;
    let r2 = r * r;
    let c2 = c * c;

    let x0 = -a * c / (a2b2);
    let y0 = -b * c / (a2b2);

    if c2 > r2 * (a2b2) {
        vec![]
    } else if (c2 - r2 * (a2b2)).abs() < 0.000000001 {
        vec![Vec2::from_xy(x0, y0)]
    } else {
        let d = r2 - c2 / (a2b2);
        let mult = (d / (a2b2)).sqrt();
        let ax = x0 + b * mult;
        let bx = x0 - b * mult;
        let ay = y0 - a * mult;
        let by = y0 + a * mult;
        vec![Vec2::from_xy(ax, ay), Vec2::from_xy(bx, by)]
    }
}