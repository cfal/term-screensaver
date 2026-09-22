mod fractal;
mod orb;
mod plasma;
mod solids;
mod starfield;
mod text;

use chrono::{DateTime, Local};
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::canvas::{Canvas, Rgb};

pub use plasma::Plasma;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum EffectKind {
    Orb,
    Fractal,
    Donut,
    Wireframe,
    Glyphs,
    Clock,
    Starfield,
    Plasma,
}

impl EffectKind {
    pub const ALL: &'static [Self] = &[
        Self::Orb,
        Self::Fractal,
        Self::Donut,
        Self::Wireframe,
        Self::Glyphs,
        Self::Clock,
        Self::Starfield,
        Self::Plasma,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Orb => "orb",
            Self::Fractal => "fractal",
            Self::Donut => "donut",
            Self::Wireframe => "wireframe",
            Self::Glyphs => "glyphs",
            Self::Clock => "clock",
            Self::Starfield => "starfield",
            Self::Plasma => "plasma",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|effect| effect.name() == name)
    }

    pub fn create(self, seed: u64) -> Box<dyn Animation> {
        match self {
            Self::Orb => Box::new(orb::Orb::new(seed)),
            Self::Fractal => Box::new(fractal::Fractal::new(seed)),
            Self::Donut => Box::new(solids::Donut::new(seed)),
            Self::Wireframe => Box::new(solids::Wireframe::new(seed)),
            Self::Glyphs => Box::new(text::GlyphSpin::new(seed)),
            Self::Clock => Box::new(text::Clock::new(seed)),
            Self::Starfield => Box::new(starfield::Starfield::new(seed)),
            Self::Plasma => Box::new(Plasma::new(seed)),
        }
    }
}

pub struct FrameContext {
    pub scene_seconds: f64,
    pub wall_time: DateTime<Local>,
    pub ascii: bool,
    pub colored: bool,
}

impl FrameContext {
    pub fn color(&self, color: Rgb) -> Option<Rgb> {
        self.colored.then_some(color)
    }
}

pub trait Animation {
    fn render(&mut self, frame: &FrameContext, canvas: &mut Canvas);
}

pub fn random_seed() -> u64 {
    rand::thread_rng().r#gen()
}

pub fn seeded_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

pub fn lerp_color(a: Rgb, b: Rgb, amount: f64) -> Rgb {
    let amount = amount.clamp(0.0, 1.0);
    let channel = |a: u8, b: u8| (f64::from(a) + f64::from(b as i16 - a as i16) * amount) as u8;
    Rgb::new(channel(a.r, b.r), channel(a.g, b.g), channel(a.b, b.b))
}

pub fn ramp(value: f64, glyphs: &[char]) -> char {
    let index = (value.clamp(0.0, 0.999_999) * glyphs.len() as f64) as usize;
    glyphs[index]
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};

    use super::*;

    fn frame(seconds: f64, ascii: bool, colored: bool) -> FrameContext {
        FrameContext {
            scene_seconds: seconds,
            wall_time: Local.with_ymd_and_hms(2026, 9, 22, 12, 34, 0).unwrap(),
            ascii,
            colored,
        }
    }

    #[test]
    fn every_effect_is_deterministic_and_handles_tiny_sizes() {
        for kind in EffectKind::ALL {
            for (width, height) in [(0, 0), (1, 1), (7, 3), (48, 18)] {
                let mut first = Canvas::new(width, height);
                let mut second = Canvas::new(width, height);
                kind.create(91).render(&frame(2.5, true, false), &mut first);
                kind.create(91)
                    .render(&frame(2.5, true, false), &mut second);
                assert_eq!(first, second, "{kind:?} differed at {width}x{height}");
                assert!(
                    first
                        .cells()
                        .iter()
                        .all(|cell| { cell.glyph.is_ascii() && cell.style.foreground.is_none() })
                );
                if width >= 40 && height >= 12 {
                    assert!(first.visible_cells() > 0, "{kind:?} rendered blank");
                }
            }
        }
    }
}
