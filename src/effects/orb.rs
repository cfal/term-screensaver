use std::f64::consts::PI;

use rand::Rng;

use crate::{
    canvas::{Canvas, Rgb, Style},
    effects::{Animation, FrameContext, lerp_color, ramp, seeded_rng},
    math::Vec3,
};

const PALETTES: &[[Rgb; 3]] = &[
    [
        Rgb::new(22, 8, 76),
        Rgb::new(79, 91, 255),
        Rgb::new(100, 255, 194),
    ],
    [
        Rgb::new(92, 12, 18),
        Rgb::new(255, 82, 48),
        Rgb::new(255, 226, 112),
    ],
    [
        Rgb::new(5, 45, 38),
        Rgb::new(16, 184, 157),
        Rgb::new(218, 246, 116),
    ],
];

pub struct Orb {
    tilt: f64,
    yaw_speed: f64,
    roll_speed: f64,
    phase: f64,
    band_frequency: f64,
    palette: [Rgb; 3],
    spark_phase: f64,
}

impl Orb {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        Self {
            tilt: rng.gen_range(-0.7..0.7),
            yaw_speed: rng.gen_range(0.28..0.62),
            roll_speed: rng.gen_range(-0.24..0.24),
            phase: rng.gen_range(0.0..PI * 2.0),
            band_frequency: rng.gen_range(5.0..9.0),
            palette: PALETTES[rng.gen_range(0..PALETTES.len())],
            spark_phase: rng.gen_range(0.0..PI * 2.0),
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

impl Animation for Orb {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        const ASCII: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
        const UNICODE: &[char] = &[' ', '·', '·', ':', '•', '•', 'o', 'O', '●', '@'];
        let glyphs = if frame.ascii { ASCII } else { UNICODE };
        let width = f64::from(canvas.width());
        let height = f64::from(canvas.height());
        if width < 2.0 || height < 2.0 {
            return;
        }

        let center_x = width * 0.5;
        let center_y = height * 0.5;
        let radius_x = (width * 0.38).min(height * 0.92).max(1.0);
        let breath = 1.0 + (frame.scene_seconds * 1.4 + self.phase).sin() * 0.035;
        let radius_x = radius_x * breath;
        let radius_y = radius_x * 0.5;
        let light = Vec3::new(-0.45, -0.35, 0.82).normalized();
        let yaw = frame.scene_seconds * self.yaw_speed + self.phase;
        let roll = frame.scene_seconds * self.roll_speed;

        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                let nx = (f64::from(x) + 0.5 - center_x) / radius_x;
                let ny = (f64::from(y) + 0.5 - center_y) / radius_y;
                let radius_squared = nx * nx + ny * ny;

                if radius_squared > 1.0 {
                    let radius = radius_squared.sqrt();
                    if radius < 1.22 {
                        let halo = ((1.22 - radius) / 0.22).powi(2)
                            * (0.45 + 0.2 * (ny * 11.0 + frame.scene_seconds).sin());
                        if halo > 0.12 {
                            canvas.put(
                                i32::from(x),
                                i32::from(y),
                                ramp(halo * 0.48, glyphs),
                                Style {
                                    foreground: frame.color(self.color(halo * 0.55)),
                                    bold: false,
                                    dim: true,
                                },
                            );
                        }
                    }
                    continue;
                }

                let z = (1.0 - radius_squared).sqrt();
                let normal = Vec3::new(nx, ny, z);
                let surface = normal.rotate_z(-roll).rotate_x(-self.tilt).rotate_y(-yaw);
                let longitude = surface.z.atan2(surface.x);
                let latitude = surface.y.clamp(-1.0, 1.0).asin();
                let filament = (self.band_frequency * latitude
                    + 3.0 * longitude
                    + 2.0 * (longitude * 2.0 + frame.scene_seconds * 0.7).sin())
                .sin();
                let bands = (1.0 - filament.abs()).powf(5.0);
                let fine = (surface.x * 17.0 - surface.z * 11.0 + frame.scene_seconds * 1.8).sin()
                    * 0.5
                    + 0.5;
                let diffuse = normal.dot(light).max(0.0);
                let rim = (1.0 - z).powf(2.0);
                let specular = normal.dot(light).max(0.0).powf(16.0);
                let intensity = (0.08
                    + diffuse * 0.54
                    + rim * 0.25
                    + specular * 0.55
                    + bands * (0.18 + fine * 0.22))
                    .clamp(0.0, 1.0);
                canvas.put(
                    i32::from(x),
                    i32::from(y),
                    ramp(intensity, glyphs),
                    Style {
                        foreground: frame
                            .color(self.color((intensity * 0.65 + bands * 0.35).clamp(0.0, 1.0))),
                        bold: intensity > 0.74,
                        dim: intensity < 0.2,
                    },
                );
            }
        }

        let spark_count = usize::from(canvas.width().min(80) / 8).max(4);
        for spark in 0..spark_count {
            let offset = spark as f64 * 2.399_963 + self.spark_phase;
            let orbit = frame.scene_seconds * (0.18 + spark as f64 * 0.007) + offset;
            let distance = radius_x * (1.12 + (offset * 3.1).sin().abs() * 0.32);
            let x = center_x + orbit.cos() * distance;
            let y = center_y + orbit.sin() * distance * 0.46;
            let pulse = (frame.scene_seconds * 2.4 + offset).sin() * 0.5 + 0.5;
            canvas.put(
                x.round() as i32,
                y.round() as i32,
                if frame.ascii { '*' } else { '·' },
                Style {
                    foreground: frame.color(self.color(0.55 + pulse * 0.45)),
                    bold: pulse > 0.75,
                    dim: pulse < 0.35,
                },
            );
        }
    }
}
