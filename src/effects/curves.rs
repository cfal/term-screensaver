use std::f64::consts::{PI, TAU};

use rand::Rng;

use crate::{
    canvas::{Canvas, Rgb, Style},
    effects::{Animation, FrameContext, lerp_color, ramp, seeded_rng},
    math::{Projected, Vec3, draw_projected_line, fitting_scale, project},
};

const KNOT_BOUNDING_RADIUS: f64 = 1.75;
const KNOT_PARAMS: &[(f64, f64)] = &[(2.0, 3.0), (2.0, 5.0), (3.0, 4.0)];
const HELIX_BOUNDING_RADIUS: f64 = 1.66;
const HELIX_RADIUS: f64 = 0.95;
const HELIX_HALF_HEIGHT: f64 = 1.35;
const HELIX_SAMPLES: usize = 160;

pub struct Knot {
    samples: Vec<Vec3>,
    speed: Vec3,
    phase: Vec3,
    palette: [Rgb; 2],
}

impl Knot {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        let (p, q) = KNOT_PARAMS[rng.gen_range(0..KNOT_PARAMS.len())];
        let samples = (0..720)
            .map(|step| {
                let t = step as f64 / 720.0 * TAU;
                let radial = 2.0 + (q * t).cos();
                Vec3::new(
                    radial * (p * t).cos(),
                    radial * (p * t).sin(),
                    (q * t).sin(),
                ) * 0.55
            })
            .collect();
        Self {
            samples,
            speed: Vec3::new(
                rng.gen_range(0.22..0.4),
                rng.gen_range(0.3..0.5),
                rng.gen_range(-0.2..0.2),
            ),
            phase: Vec3::new(
                rng.r#gen::<f64>() * TAU,
                rng.r#gen::<f64>() * TAU,
                rng.r#gen::<f64>() * TAU,
            ),
            palette: [Rgb::new(255, 179, 71), Rgb::new(255, 89, 143)],
        }
    }
}

impl Animation for Knot {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        const ASCII: &[char] = &['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];
        const UNICODE: &[char] = &['·', '·', ':', '•', '+', '*', 'o', 'O', '0', '●', '▓', '@'];
        let glyphs = if frame.ascii { ASCII } else { UNICODE };
        let scale = fitting_scale(canvas.width(), canvas.height(), KNOT_BOUNDING_RADIUS, 0.85);
        let angles = self.speed * frame.scene_seconds + self.phase;

        for (index, &point) in self.samples.iter().enumerate() {
            let rotated = point.rotate(angles.x, angles.y, angles.z);
            let luminance = (rotated.z / KNOT_BOUNDING_RADIUS * 0.5 + 0.5).clamp(0.0, 1.0);
            let Some(projected) = project(rotated, canvas.width(), canvas.height(), scale) else {
                continue;
            };
            let along = index as f64 / self.samples.len() as f64;
            canvas.plot(
                projected.x,
                projected.y,
                projected.inverse_depth,
                ramp(luminance, glyphs),
                Style {
                    foreground: frame.color(lerp_color(self.palette[0], self.palette[1], along)),
                    bold: luminance > 0.72,
                    dim: luminance < 0.2,
                },
            );
        }
    }
}

pub struct Helix {
    strands: [Vec<Vec3>; 2],
    speed_y: f64,
    tilt: f64,
    colors: [Rgb; 2],
}

impl Helix {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        let turns = rng.gen_range(2.5..3.5);
        let phase = rng.r#gen::<f64>() * TAU;
        let strand = |offset: f64| {
            (0..HELIX_SAMPLES)
                .map(|step| {
                    let t = step as f64 / (HELIX_SAMPLES - 1) as f64;
                    let angle = t * turns * TAU + phase + offset;
                    Vec3::new(
                        angle.cos() * HELIX_RADIUS,
                        (t - 0.5) * 2.0 * HELIX_HALF_HEIGHT,
                        angle.sin() * HELIX_RADIUS,
                    )
                })
                .collect()
        };
        Self {
            strands: [strand(0.0), strand(PI)],
            speed_y: rng.gen_range(0.45..0.8),
            tilt: rng.gen_range(0.35..0.6),
            colors: [Rgb::new(94, 234, 140), Rgb::new(92, 200, 255)],
        }
    }
}

impl Animation for Helix {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        const ASCII: &[char] = &['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];
        const UNICODE: &[char] = &['·', '·', ':', '•', '+', '*', 'o', 'O', '0', '●', '▓', '@'];
        let glyphs = if frame.ascii { ASCII } else { UNICODE };
        let scale = fitting_scale(canvas.width(), canvas.height(), HELIX_BOUNDING_RADIUS, 0.85);
        let angle_y = frame.scene_seconds * self.speed_y;
        let angle_x = self.tilt + 0.25 * (frame.scene_seconds * 0.35).sin();

        let projected: [Vec<(Option<Projected>, f64)>; 2] = self.strands.each_ref().map(|strand| {
            strand
                .iter()
                .map(|point| {
                    let rotated = point.rotate_x(angle_x).rotate_y(angle_y);
                    let luminance = (rotated.z / HELIX_BOUNDING_RADIUS * 0.5 + 0.5).clamp(0.0, 1.0);
                    (
                        project(rotated, canvas.width(), canvas.height(), scale),
                        luminance,
                    )
                })
                .collect()
        });

        let rung_style = Style {
            foreground: frame.color(lerp_color(self.colors[0], self.colors[1], 0.5)),
            bold: false,
            dim: true,
        };
        for step in (0..HELIX_SAMPLES).step_by(8) {
            let (Some(start), Some(end)) = (projected[0][step].0, projected[1][step].0) else {
                continue;
            };
            draw_projected_line(
                canvas,
                start,
                end,
                if frame.ascii { '-' } else { '·' },
                rung_style,
            );
        }

        for (strand_index, strand) in projected.iter().enumerate() {
            for &(point, luminance) in strand {
                let Some(projected) = point else {
                    continue;
                };
                canvas.plot(
                    projected.x,
                    projected.y,
                    projected.inverse_depth,
                    ramp(luminance, glyphs),
                    Style {
                        foreground: frame.color(self.colors[strand_index]),
                        bold: luminance > 0.72,
                        dim: luminance < 0.2,
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_on_screen(point: Vec3, width: u16, height: u16, scale: f64) {
        let projected = project(point, width, height, scale).expect("point crossed the camera");
        assert!((0..i32::from(width)).contains(&projected.x));
        assert!((0..i32::from(height)).contains(&projected.y));
    }

    #[test]
    fn knot_samples_fit_common_terminal_sizes() {
        let knot = Knot::new(91);
        for (width, height) in [(79, 24), (119, 30), (39, 12)] {
            let scale = fitting_scale(width, height, KNOT_BOUNDING_RADIUS, 0.85);
            for seconds in [0.0, 2.5, 8.884, 31.0] {
                let angles = knot.speed * seconds + knot.phase;
                for &point in &knot.samples {
                    assert_on_screen(
                        point.rotate(angles.x, angles.y, angles.z),
                        width,
                        height,
                        scale,
                    );
                }
            }
        }
    }

    #[test]
    fn helix_points_fit_common_terminal_sizes() {
        let helix = Helix::new(91);
        for (width, height) in [(79, 24), (119, 30), (39, 12)] {
            let scale = fitting_scale(width, height, HELIX_BOUNDING_RADIUS, 0.85);
            for seconds in [0.0, 2.5, 8.884, 31.0] {
                let angle_y = seconds * helix.speed_y;
                let angle_x = helix.tilt + 0.25 * (seconds * 0.35).sin();
                for strand in &helix.strands {
                    for &point in strand {
                        assert_on_screen(
                            point.rotate_x(angle_x).rotate_y(angle_y),
                            width,
                            height,
                            scale,
                        );
                    }
                }
            }
        }
    }
}
