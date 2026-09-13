use std::ops::{BitOr, Sub};

/// Independent mechanism switches; construction cannot contain unknown bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdaptiveFeatures(u8);

impl AdaptiveFeatures {
    pub const STALL: Self = Self(1);
    pub const LOSS: Self = Self(1 << 1);
    pub const QUEUE: Self = Self(1 << 2);
    pub const DEADLINE: Self = Self(1 << 3);
    pub const REJOIN: Self = Self(1 << 4);
    pub const SOLE: Self = Self(1 << 5);
    pub const PREF: Self = Self(1 << 6);
    pub const RATECAP: Self = Self(1 << 7);
    pub const ALL: Self = Self(u8::MAX);
    pub const NONE: Self = Self(0);

    pub const fn bits(self) -> u8 {
        self.0
    }
    pub const fn contains(self, feature: Self) -> bool {
        self.0 & feature.0 == feature.0
    }
}

impl Default for AdaptiveFeatures {
    fn default() -> Self {
        Self::ALL
    }
}

impl Sub for AdaptiveFeatures {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }
}

impl BitOr for AdaptiveFeatures {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
