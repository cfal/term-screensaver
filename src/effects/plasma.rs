use rand::Rng;

use crate::{
    canvas::{Canvas, Rgb, Style},
    effects::{Animation, FrameContext, lerp_color, ramp, seeded_rng},
};

pub struct Plasma {
    phase: [f64; 4],
    speed: f64,
    colors: [Rgb; 3],
}

impl Plasma {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        Self {
            phase: [rng.r#gen(), rng.r#gen(), rng.r#gen(), rng.r#gen()],
            speed: rng.gen_range(0.45..1.05),
            colors: [
                Rgb::new(18, 22, 58),
                Rgb::new(20, 210, 176),
                Rgb::new(255, 188, 62),
            ],
        }
    }
}

impl Animation for Plasma {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        const ASCII: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
        const UNICODE: &[char] = &[' ', '·', ':', '•', '+', '*', 'o', 'O', '0', '@'];
        let glyphs = if frame.ascii { ASCII } else { UNICODE };
        let width = f64::from(canvas.width().max(1));
        let height = f64::from(canvas.height().max(1));
        let time = frame.scene_seconds * self.speed;

        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                let nx = f64::from(x) / width * 2.0 - 1.0;
                let ny = f64::from(y) / height * 2.0 - 1.0;
                let radial = (nx * nx + ny * ny).sqrt();
                let wave = (nx * 8.0 + time + self.phase[0] * 6.0).sin()
                    + (ny * 9.0 - time * 1.3 + self.phase[1] * 6.0).sin()
                    + ((nx + ny) * 6.0 + time * 0.7 + self.phase[2] * 6.0).sin()
                    + (radial * 14.0 - time * 1.8 + self.phase[3] * 6.0).sin();
                let value = (wave / 4.0 * 0.5 + 0.5).clamp(0.0, 1.0);
                let color = if value < 0.5 {
                    lerp_color(self.colors[0], self.colors[1], value * 2.0)
                } else {
                    lerp_color(self.colors[1], self.colors[2], (value - 0.5) * 2.0)
                };
                canvas.put(
                    i32::from(x),
                    i32::from(y),
                    ramp(value, glyphs),
                    Style {
                        foreground: frame.color(color),
                        bold: value > 0.82,
                        dim: value < 0.2,
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};

    use super::*;

    #[test]
    fn deterministic_and_visible() {
        let frame = FrameContext {
            scene_seconds: 1.25,
            wall_time: Local.with_ymd_and_hms(2026, 9, 22, 12, 34, 0).unwrap(),
            ascii: true,
            colored: false,
        };
        let mut first = Canvas::new(40, 12);
        let mut second = Canvas::new(40, 12);
        Plasma::new(7).render(&frame, &mut first);
        Plasma::new(7).render(&frame, &mut second);
        assert_eq!(first, second);
        assert!(first.visible_cells() > first.cells().len() / 2);
        assert!(first.cells().iter().all(|cell| cell.glyph.is_ascii()));
    }
}
