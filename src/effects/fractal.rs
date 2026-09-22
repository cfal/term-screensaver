use std::f64::consts::PI;

use rand::Rng;

use crate::{
    canvas::{Canvas, Rgb, Style},
    effects::{Animation, FrameContext, lerp_color, ramp, seeded_rng},
};

const MANDELBROT_CENTERS: &[(f64, f64, f64)] = &[
    (-0.743_643_887, 0.131_825_904, 1.8),
    (-0.101_096_363, 0.956_286_51, 1.35),
    (-1.250_66, 0.020_12, 1.6),
    (-0.16, 1.0405, 1.25),
];

const JULIA_CONSTANTS: &[(f64, f64)] = &[
    (-0.8, 0.156),
    (-0.4, 0.6),
    (0.285, 0.01),
    (-0.701_76, -0.3842),
];

pub struct Fractal {
    julia: bool,
    center: (f64, f64),
    base_zoom: f64,
    julia_constant: (f64, f64),
    phase: f64,
    palette: [Rgb; 3],
}

impl Fractal {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        let julia = rng.gen_bool(0.45);
        let &(center_x, center_y, zoom) =
            &MANDELBROT_CENTERS[rng.gen_range(0..MANDELBROT_CENTERS.len())];
        let julia_constant = JULIA_CONSTANTS[rng.gen_range(0..JULIA_CONSTANTS.len())];
        let palettes = [
            [
                Rgb::new(11, 19, 48),
                Rgb::new(38, 182, 192),
                Rgb::new(255, 221, 120),
            ],
            [
                Rgb::new(45, 5, 73),
                Rgb::new(207, 49, 128),
                Rgb::new(255, 188, 73),
            ],
        ];
        Self {
            julia,
            center: if julia {
                (0.0, 0.0)
            } else {
                (center_x, center_y)
            },
            base_zoom: if julia {
                rng.gen_range(0.78..1.15)
            } else {
                zoom
            },
            julia_constant,
            phase: rng.gen_range(0.0..PI * 2.0),
            palette: palettes[rng.gen_range(0..palettes.len())],
        }
    }

    fn color(&self, value: f64) -> Rgb {
        if value < 0.55 {
            lerp_color(self.palette[0], self.palette[1], value / 0.55)
        } else {
            lerp_color(self.palette[1], self.palette[2], (value - 0.55) / 0.45)
        }
    }
}

impl Animation for Fractal {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        const ASCII: &[char] = &[' ', '.', ',', ':', ';', 'i', 'x', 'X', '#', '@'];
        const UNICODE: &[char] = &[' ', '·', ':', '•', '░', '▒', '▓', 'o', 'O', '●'];
        let glyphs = if frame.ascii { ASCII } else { UNICODE };
        let width = f64::from(canvas.width().max(1));
        let height = f64::from(canvas.height().max(1));
        let aspect = width / (height * 2.0);
        let zoom = self.base_zoom
            * 1.7_f64.powf(
                frame.scene_seconds * 0.14 + (frame.scene_seconds * 0.23 + self.phase).sin() * 0.22,
            );
        let drift = if self.julia { 0.0 } else { 0.015 / zoom };
        let center_x = self.center.0 + (frame.scene_seconds * 0.21 + self.phase).cos() * drift;
        let center_y = self.center.1 + (frame.scene_seconds * 0.17 + self.phase).sin() * drift;
        let max_iterations = 72;

        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                let real = center_x + ((f64::from(x) + 0.5) / width - 0.5) * 3.2 * aspect / zoom;
                let imaginary = center_y + ((f64::from(y) + 0.5) / height - 0.5) * 2.0 / zoom;
                let (mut zr, mut zi, cr, ci) = if self.julia {
                    (
                        real,
                        imaginary,
                        self.julia_constant.0,
                        self.julia_constant.1,
                    )
                } else {
                    (0.0, 0.0, real, imaginary)
                };
                let mut escaped_at = None;
                for iteration in 0..max_iterations {
                    let zr_squared = zr * zr;
                    let zi_squared = zi * zi;
                    if zr_squared + zi_squared > 4.0 {
                        let magnitude = (zr_squared + zi_squared).sqrt();
                        let smooth = iteration as f64 + 1.0 - magnitude.ln().ln() / 2.0_f64.ln();
                        escaped_at = Some(smooth);
                        break;
                    }
                    zi = 2.0 * zr * zi + ci;
                    zr = zr_squared - zi_squared + cr;
                }

                let value = escaped_at
                    .map(|iteration| {
                        let normalized = (iteration / max_iterations as f64).clamp(0.0, 1.0);
                        (normalized * 3.4 + frame.scene_seconds * 0.035 + self.phase).fract()
                    })
                    .unwrap_or(0.98);
                canvas.put(
                    i32::from(x),
                    i32::from(y),
                    ramp(value, glyphs),
                    Style {
                        foreground: frame.color(self.color(value)),
                        bold: value > 0.86,
                        dim: value < 0.14,
                    },
                );
            }
        }
    }
}
