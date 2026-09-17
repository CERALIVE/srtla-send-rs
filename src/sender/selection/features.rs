use std::ops::{BitOr, Sub};

/// Independent mechanism switches; construction cannot contain unknown bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchedulerFeatures(u16);

impl SchedulerFeatures {
    pub const STALL: Self = Self(1);
    pub const LOSS: Self = Self(1 << 1);
    pub const QUEUE: Self = Self(1 << 2);
    pub const DEADLINE: Self = Self(1 << 3);
    pub const REJOIN: Self = Self(1 << 4);
    pub const SOLE: Self = Self(1 << 5);
    pub const PREF: Self = Self(1 << 6);
    pub const RATECAP: Self = Self(1 << 7);
    pub const QUALITY: Self = Self(1 << 8);
    pub const ALL: Self = Self((1 << 9) - 1);
    pub const NONE: Self = Self(0);

    pub const fn bits(self) -> u16 {
        self.0
    }
    pub const fn contains(self, feature: Self) -> bool {
        self.0 & feature.0 == feature.0
    }
}

impl Default for SchedulerFeatures {
    fn default() -> Self {
        Self::ALL
    }
}

impl Sub for SchedulerFeatures {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }
}

impl BitOr for SchedulerFeatures {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
