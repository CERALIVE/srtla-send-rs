use network_sim::scenarios::{self, Profile};

pub struct Settle {
    target: f64,
    previous_second: u64,
    consecutive: u32,
}

impl Settle {
    pub fn for_profile(profile: &Profile) -> Self {
        Self::new(profile.warmup_offered_bps)
    }

    pub fn new(warmup_bps: u64) -> Self {
        Self {
            target: f64::from(u32::try_from(warmup_bps).expect("scenario rates fit u32"))
                * scenarios::WARMUP_SETTLE_RATIO,
            previous_second: 0,
            consecutive: 0,
        }
    }

    pub fn observe(&mut self, second: u64, bps: f64) -> bool {
        if second != self.previous_second + 1 {
            self.consecutive = 0;
        }
        self.previous_second = second;
        self.consecutive = if bps.is_finite() && bps >= self.target {
            self.consecutive + 1
        } else {
            0
        };
        self.consecutive >= scenarios::WARMUP_SETTLE_SECONDS
    }
}
