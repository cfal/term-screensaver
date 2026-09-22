use rand::Rng;

use crate::{
    canvas::{Canvas, Rgb, Style},
    effects::{Animation, FrameContext, lerp_color, seeded_rng},
};

#[derive(Clone, Copy)]
struct Star {
    x: f64,
    y: f64,
    depth: f64,
    speed: f64,
}

pub struct Starfield {
    stars: Vec<Star>,
    drift_x: f64,
    drift_y: f64,
    colors: [Rgb; 2],
}

impl Starfield {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        let stars = (0..260)
            .map(|_| Star {
                x: rng.gen_range(-1.0..1.0),
                y: rng.gen_range(-0.62..0.62),
                depth: rng.gen_range(0.08..1.0),
                speed: rng.gen_range(0.12..0.32),
            })
            .collect();
        Self {
            stars,
            drift_x: rng.gen_range(-0.08..0.08),
            drift_y: rng.gen_range(-0.04..0.04),
            colors: [Rgb::new(84, 151, 255), Rgb::new(255, 244, 199)],
        }
    }
}

impl Animation for Starfield {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        let width = f64::from(canvas.width());
        let height = f64::from(canvas.height());
        if width < 2.0 || height < 2.0 {
            return;
        }
        let center_x = width * (0.5 + self.drift_x);
        let center_y = height * (0.5 + self.drift_y);
        let scale_x = width * 0.42;
        let scale_y = height * 0.72;

        for star in &self.stars {
            let depth = (star.depth - frame.scene_seconds * star.speed).rem_euclid(0.94) + 0.06;
            let previous_depth = (depth + star.speed * 0.12).min(1.0);
            let x = center_x + star.x / depth * scale_x;
            let y = center_y + star.y / depth * scale_y;
            let previous_x = center_x + star.x / previous_depth * scale_x;
            let previous_y = center_y + star.y / previous_depth * scale_y;
            let brightness = (1.0 - depth).clamp(0.0, 1.0);
            let style = Style {
                foreground: frame.color(lerp_color(self.colors[0], self.colors[1], brightness)),
                bold: brightness > 0.72,
                dim: brightness < 0.25,
            };
            let dx = x - previous_x;
            let dy = y - previous_y;
            let steps = (dx.abs().max(dy.abs()).round() as i32).clamp(1, 4);
            for step in 0..=steps {
                let amount = f64::from(step) / f64::from(steps);
                let trail_x = previous_x + dx * amount;
                let trail_y = previous_y + dy * amount;
                let glyph = if step == steps {
                    if brightness > 0.78 {
                        if frame.ascii { '*' } else { '●' }
                    } else if brightness > 0.42 {
                        '+'
                    } else if frame.ascii {
                        '.'
                    } else {
                        '·'
                    }
                } else if frame.ascii {
                    '.'
                } else {
                    '·'
                };
                canvas.plot(
                    trail_x.round() as i32,
                    trail_y.round() as i32,
                    1.0 / depth + amount * 0.001,
                    glyph,
                    style,
                );
            }
        }
    }
}
