use std::time::{Duration, Instant};

use anyhow::{Result, ensure};

use super::{Profile, TimedEvent};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EventRecord {
    #[serde(rename = "t_ms")]
    pub t_actual_ms: u64,
    pub event: TimedEvent,
}

#[derive(Debug, Default)]
pub struct EventLog {
    pub entries: Vec<EventRecord>,
}

pub struct Scheduler {
    events: Vec<TimedEvent>,
    next: usize,
    duration: Duration,
    start: Option<Instant>,
    log: EventLog,
    ready: bool,
}

impl Scheduler {
    pub fn new(profile: &Profile) -> Result<Self> {
        Ok(Self {
            events: profile.expanded_events()?,
            next: 0,
            duration: profile.duration,
            start: None,
            log: EventLog::default(),
            ready: true,
        })
    }

    pub const fn log(&self) -> &EventLog {
        &self.log
    }

    /// Blocking harness runner. The first call anchors the monotonic run clock.
    /// An application error poisons this scheduler: partial side effects are never retried.
    pub fn run(&mut self, apply: &mut impl FnMut(&TimedEvent) -> Result<()>) -> Result<()> {
        ensure!(
            self.ready,
            "scheduler stopped after an event application failure"
        );
        let start = *self.start.get_or_insert_with(Instant::now);
        while let Some(event) = self.events.get(self.next) {
            std::thread::sleep(event.at.saturating_sub(start.elapsed()));
            self.ready = false;
            apply(event)?;
            self.log.entries.push(EventRecord {
                t_actual_ms: u64::try_from(start.elapsed().as_millis())?,
                event: event.clone(),
            });
            self.ready = true;
            self.next += 1;
        }
        std::thread::sleep(self.duration.saturating_sub(start.elapsed()));
        Ok(())
    }
}
