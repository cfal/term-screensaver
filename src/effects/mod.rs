mod plasma;

use chrono::{DateTime, Local};
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::canvas::{Canvas, Rgb};

pub use plasma::Plasma;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum EffectKind {
    Plasma,
}

impl EffectKind {
    pub const ALL: &'static [Self] = &[Self::Plasma];

    pub const fn name(self) -> &'static str {
        match self {
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
