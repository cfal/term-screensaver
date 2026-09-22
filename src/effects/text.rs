use std::f64::consts::{PI, TAU};

use chrono::Timelike;
use rand::Rng;

use crate::{
    canvas::{Canvas, Rgb, Style},
    effects::{Animation, FrameContext, lerp_color, seeded_rng},
    font,
    math::{Vec3, fitting_scale, project},
};

const GLYPH_SEQUENCE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const GLYPH_SOURCE_EXTENT: Vec3 = Vec3::new(2.18, 3.18, 0.12);
const GLYPH_RADIUS: f64 = 1.25;
const CLOCK_HALF_WIDTH: f64 = 3.0;

fn glyph_points(glyph: char) -> Vec<(usize, Vec3)> {
    const OFFSETS: &[Vec3] = &[
        Vec3::new(-0.18, -0.18, -0.12),
        Vec3::new(0.18, -0.18, -0.12),
        Vec3::new(-0.18, 0.18, 0.12),
        Vec3::new(0.18, 0.18, 0.12),
    ];
    let normalization = GLYPH_RADIUS / GLYPH_SOURCE_EXTENT.length();
    font::points(&glyph.to_string())
        .into_iter()
        .enumerate()
        .flat_map(|(index, base)| {
            OFFSETS
                .iter()
                .map(move |&offset| (index, (base + offset) * normalization))
        })
        .collect()
}

fn clock_points(text: &str) -> Vec<Vec3> {
    let mut points = font::points(text);
    let half_width = points.iter().map(|point| point.x.abs()).fold(0.0, f64::max);
    if half_width > f64::EPSILON {
        let normalization = CLOCK_HALF_WIDTH / half_width;
        for point in &mut points {
            *point = *point * normalization;
        }
    }
    points
}

pub struct GlyphSpin {
    offset: usize,
    speed: f64,
    tilt: f64,
    colors: [Rgb; 2],
}

impl GlyphSpin {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        Self {
            offset: rng.gen_range(0..GLYPH_SEQUENCE.len()),
            speed: rng.gen_range(0.65..1.05),
            tilt: rng.gen_range(-0.22..0.22),
            colors: [Rgb::new(64, 220, 192), Rgb::new(255, 190, 88)],
        }
    }
}

impl Animation for GlyphSpin {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        let cycle = frame.scene_seconds * self.speed / 1.7;
        let cycle_index = cycle.floor() as usize;
        let local = cycle.fract();
        let glyph = GLYPH_SEQUENCE[(self.offset + cycle_index) % GLYPH_SEQUENCE.len()] as char;
        let smooth = local * local * (3.0 - 2.0 * local);
        let angle_y = smooth * TAU;
        let angle_x = self.tilt + (local * PI).sin() * 0.18;
        let scale = fitting_scale(canvas.width(), canvas.height(), GLYPH_RADIUS, 0.86);
        let points = glyph_points(glyph);

        for (index, point) in points {
            let point = point.rotate_x(angle_x).rotate_y(angle_y);
            let Some(projected) = project(point, canvas.width(), canvas.height(), scale) else {
                continue;
            };
            let color_position = (index as f64 / 35.0 + point.z * 0.08).clamp(0.0, 1.0);
            canvas.plot(
                projected.x,
                projected.y,
                projected.inverse_depth,
                glyph,
                Style {
                    foreground: frame.color(lerp_color(
                        self.colors[0],
                        self.colors[1],
                        color_position,
                    )),
                    bold: point.z > 0.0,
                    dim: point.z < -0.1,
                },
            );
        }
    }
}

#[derive(Clone, Copy)]
enum ClockVariant {
    Spin,
    Orbit,
    Ribbon,
}

pub struct Clock {
    variant: ClockVariant,
    speed: f64,
    phase: f64,
    colors: [Rgb; 3],
}

impl Clock {
    pub fn new(seed: u64) -> Self {
        let mut rng = seeded_rng(seed);
        let variant = match rng.gen_range(0..3) {
            0 => ClockVariant::Spin,
            1 => ClockVariant::Orbit,
            _ => ClockVariant::Ribbon,
        };
        Self {
            variant,
            speed: rng.gen_range(0.35..0.72),
            phase: rng.gen_range(0.0..TAU),
            colors: [
                Rgb::new(36, 47, 112),
                Rgb::new(73, 218, 186),
                Rgb::new(255, 223, 126),
            ],
        }
    }

    fn time_text(frame: &FrameContext) -> String {
        format!(
            "{:02}:{:02}",
            frame.wall_time.hour(),
            frame.wall_time.minute()
        )
    }

    fn style(&self, frame: &FrameContext, amount: f64, bold: bool) -> Style {
        let color = if amount < 0.55 {
            lerp_color(self.colors[0], self.colors[1], amount / 0.55)
        } else {
            lerp_color(self.colors[1], self.colors[2], (amount - 0.55) / 0.45)
        };
        Style {
            foreground: frame.color(color),
            bold,
            dim: amount < 0.18,
        }
    }

    fn render_spin(&self, frame: &FrameContext, canvas: &mut Canvas, points: &[Vec3]) {
        let scale = (f64::from(canvas.width()) * 0.22)
            .min(f64::from(canvas.height()) * 2.2)
            .max(0.5);
        let angle_y = (frame.scene_seconds * self.speed + self.phase).sin() * 0.42;
        let angle_x = (frame.scene_seconds * self.speed * 0.63).sin() * 0.16;
        for (index, &point) in points.iter().enumerate() {
            let point = point.rotate_x(angle_x).rotate_y(angle_y);
            let Some(projected) = project(point, canvas.width(), canvas.height(), scale) else {
                continue;
            };
            let amount = (index as f64 / points.len().max(1) as f64 * 0.55
                + projected.inverse_depth * 1.7)
                .clamp(0.0, 1.0);
            canvas.plot(
                projected.x,
                projected.y,
                projected.inverse_depth,
                if frame.ascii { '#' } else { '●' },
                self.style(frame, amount, point.z > 0.0),
            );
        }
    }

    fn render_orbit(&self, frame: &FrameContext, canvas: &mut Canvas, points: &[Vec3]) {
        let scale = (f64::from(canvas.width()) * 0.11)
            .min(f64::from(canvas.height()) * 1.1)
            .max(0.35);
        for panel in 0..3 {
            let angle = frame.scene_seconds * self.speed * (0.75 + panel as f64 * 0.08)
                + self.phase
                + panel as f64 * TAU / 3.0;
            let offset_x = (angle.cos() * f64::from(canvas.width()) * 0.28).round() as i32;
            let offset_y = (angle.sin() * f64::from(canvas.height()) * 0.24).round() as i32;
            for (index, &point) in points.iter().enumerate() {
                let point = point.rotate_y(angle.sin() * 0.32).rotate_x(-0.08);
                let Some(projected) = project(point, canvas.width(), canvas.height(), scale) else {
                    continue;
                };
                let amount = (0.25 + panel as f64 * 0.22 + index as f64 / 500.0).min(1.0);
                canvas.plot(
                    projected.x + offset_x,
                    projected.y + offset_y,
                    projected.inverse_depth + panel as f64 * 0.001,
                    if frame.ascii { '*' } else { '•' },
                    self.style(frame, amount, panel == 1),
                );
            }
        }
    }

    fn render_ribbon(&self, frame: &FrameContext, canvas: &mut Canvas, points: &[Vec3]) {
        let scale = (f64::from(canvas.width()) * 0.24)
            .min(f64::from(canvas.height()) * 2.2)
            .max(0.5);
        for (index, &point) in points.iter().enumerate() {
            let twist = frame.scene_seconds * self.speed + point.x * 0.12 + self.phase;
            let twisted = Vec3::new(point.x, point.y * twist.cos(), point.y * twist.sin() * 0.72)
                .rotate_x(0.08);
            let Some(projected) = project(twisted, canvas.width(), canvas.height(), scale) else {
                continue;
            };
            let amount = (twisted.z * 0.1 + 0.52 + index as f64 / 800.0).clamp(0.0, 1.0);
            canvas.plot(
                projected.x,
                projected.y,
                projected.inverse_depth,
                if frame.ascii { '@' } else { '●' },
                self.style(frame, amount, twisted.z > 0.0),
            );
        }
    }
}

impl Animation for Clock {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas) {
        let points = clock_points(&Self::time_text(frame));
        match self.variant {
            ClockVariant::Spin => self.render_spin(frame, canvas, &points),
            ClockVariant::Orbit => self.render_orbit(frame, canvas, &points),
            ClockVariant::Ribbon => self.render_ribbon(frame, canvas, &points),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};

    use super::*;

    #[test]
    fn formats_clock_rollovers() {
        let frame = |hour, minute| FrameContext {
            scene_seconds: 0.0,
            wall_time: Local
                .with_ymd_and_hms(2026, 9, 22, hour, minute, 0)
                .unwrap(),
            ascii: true,
            colored: false,
        };
        assert_eq!(Clock::time_text(&frame(9, 59)), "09:59");
        assert_eq!(Clock::time_text(&frame(10, 0)), "10:00");
        assert_eq!(Clock::time_text(&frame(23, 59)), "23:59");
        assert_eq!(Clock::time_text(&frame(0, 0)), "00:00");
    }

    fn assert_on_screen(
        point: Vec3,
        width: u16,
        height: u16,
        scale: f64,
        offset_x: i32,
        offset_y: i32,
    ) {
        let projected = project(point, width, height, scale).expect("point crossed the camera");
        assert!((0..i32::from(width)).contains(&(projected.x + offset_x)));
        assert!((0..i32::from(height)).contains(&(projected.y + offset_y)));
    }

    #[test]
    fn normalized_clock_stays_visible_during_spin() {
        let clock = Clock::new(2);
        assert!(matches!(clock.variant, ClockVariant::Spin));
        let seconds = 8.884;
        let width = 79;
        let height = 24;
        let points = clock_points("12:34");
        let scale = (f64::from(width) * 0.22)
            .min(f64::from(height) * 2.2)
            .max(0.5);
        let angle_y = (seconds * clock.speed + clock.phase).sin() * 0.42;
        let angle_x = (seconds * clock.speed * 0.63).sin() * 0.16;

        assert!(points.iter().all(|point| point.x.abs() <= CLOCK_HALF_WIDTH));
        for point in points {
            assert_on_screen(
                point.rotate_x(angle_x).rotate_y(angle_y),
                width,
                height,
                scale,
                0,
                0,
            );
        }
    }

    #[test]
    fn rotating_glyphs_fit_common_terminal_sizes() {
        for glyph in GLYPH_SEQUENCE.iter().map(|&glyph| char::from(glyph)) {
            let points = glyph_points(glyph);
            assert!(
                points
                    .iter()
                    .all(|(_, point)| point.length() <= GLYPH_RADIUS)
            );
            for (width, height) in [(79, 24), (39, 12)] {
                let scale = fitting_scale(width, height, GLYPH_RADIUS, 0.86);
                for step in 0..32 {
                    let angle_y = f64::from(step) / 32.0 * TAU;
                    for angle_x in [-0.4, 0.0, 0.4] {
                        for &(_, point) in &points {
                            assert_on_screen(
                                point.rotate_x(angle_x).rotate_y(angle_y),
                                width,
                                height,
                                scale,
                                0,
                                0,
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn orbit_and_ribbon_fit_a_common_terminal() {
        let width = 79;
        let height = 24;
        let points = clock_points("23:59");
        let orbit_scale = (f64::from(width) * 0.11)
            .min(f64::from(height) * 1.1)
            .max(0.35);
        let ribbon_scale = (f64::from(width) * 0.24)
            .min(f64::from(height) * 2.2)
            .max(0.5);

        for step in 0..72 {
            let phase = f64::from(step) / 72.0 * TAU;
            let offset_x = (phase.cos() * f64::from(width) * 0.28).round() as i32;
            let offset_y = (phase.sin() * f64::from(height) * 0.24).round() as i32;
            for &point in &points {
                assert_on_screen(
                    point.rotate_y(phase.sin() * 0.32).rotate_x(-0.08),
                    width,
                    height,
                    orbit_scale,
                    offset_x,
                    offset_y,
                );

                let twist = phase + point.x * 0.12;
                let twisted =
                    Vec3::new(point.x, point.y * twist.cos(), point.y * twist.sin() * 0.72)
                        .rotate_x(0.08);
                assert_on_screen(twisted, width, height, ribbon_scale, 0, 0);
            }
        }
    }
}
