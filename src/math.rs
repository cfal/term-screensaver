use std::ops::{Add, Mul, Sub};

use crate::canvas::{Canvas, Style};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();
        if length <= f64::EPSILON {
            self
        } else {
            self * (1.0 / length)
        }
    }

    pub fn rotate_x(self, angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(
            self.x,
            self.y * cos - self.z * sin,
            self.y * sin + self.z * cos,
        )
    }

    pub fn rotate_y(self, angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(
            self.x * cos + self.z * sin,
            self.y,
            -self.x * sin + self.z * cos,
        )
    }

    pub fn rotate_z(self, angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(
            self.x * cos - self.y * sin,
            self.x * sin + self.y * cos,
            self.z,
        )
    }

    pub fn rotate(self, x: f64, y: f64, z: f64) -> Self {
        self.rotate_x(x).rotate_y(y).rotate_z(z)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Projected {
    pub x: i32,
    pub y: i32,
    pub inverse_depth: f64,
}

pub fn project(point: Vec3, width: u16, height: u16, scale: f64) -> Option<Projected> {
    let denominator = 4.5 - point.z;
    if !denominator.is_finite() || denominator <= 0.2 {
        return None;
    }
    let inverse_depth = 1.0 / denominator;
    let x = f64::from(width) * 0.5 + point.x * scale * inverse_depth * 2.0;
    let y = f64::from(height) * 0.5 - point.y * scale * inverse_depth;
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    Some(Projected {
        x: x.round() as i32,
        y: y.round() as i32,
        inverse_depth,
    })
}

pub fn draw_projected_line(
    canvas: &mut Canvas,
    start: Projected,
    end: Projected,
    glyph: char,
    style: Style,
) {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let steps = dx.abs().max(dy.abs());
    if steps == 0 {
        canvas.plot(start.x, start.y, start.inverse_depth, glyph, style);
        return;
    }
    for step in 0..=steps {
        let amount = f64::from(step) / f64::from(steps);
        let x = (f64::from(start.x) + f64::from(dx) * amount).round() as i32;
        let y = (f64::from(start.y) + f64::from(dy) * amount).round() as i32;
        let depth = start.inverse_depth + (end.inverse_depth - start.inverse_depth) * amount;
        canvas.plot(x, y, depth, glyph, style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotations_preserve_length() {
        let point = Vec3::new(1.0, 2.0, 3.0);
        let rotated = point.rotate(0.7, 1.1, -0.4);
        assert!((point.length() - rotated.length()).abs() < 1e-10);
    }

    #[test]
    fn projection_rejects_points_behind_camera() {
        assert!(project(Vec3::new(0.0, 0.0, 5.0), 80, 24, 20.0).is_none());
        assert!(project(Vec3::new(0.0, 0.0, 0.0), 80, 24, 20.0).is_some());
    }
}
