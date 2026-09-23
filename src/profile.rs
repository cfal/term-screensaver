use crate::effects::EffectKind;

/// A named group of effects that scenes rotate through.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    All,
    ThreeD,
    Time,
}

impl Profile {
    pub const ALL: &'static [Self] = &[Self::All, Self::ThreeD, Self::Time];

    pub const fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::ThreeD => "3d",
            Self::Time => "time",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|profile| profile.name() == name)
    }

    pub fn effects(self) -> &'static [EffectKind] {
        match self {
            Self::All => EffectKind::ALL,
            Self::ThreeD => &[EffectKind::Orb, EffectKind::Donut, EffectKind::Wireframe],
            Self::Time => &[EffectKind::Clock],
        }
    }

    pub fn names() -> impl Iterator<Item = &'static str> {
        Self::ALL.iter().map(|profile| profile.name())
    }
}
