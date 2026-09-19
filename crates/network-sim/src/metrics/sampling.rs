use std::time::{Duration, Instant};

use super::{MetricError, SAMPLE_INTERVAL};

/// Blocking 1 Hz collector lane. The caller owns its thread and joins it before teardown.
/// Callback timestamps are actual local elapsed ms; align them to the run clock at launch.
pub fn sample_for<T>(
    duration: Duration,
    mut sample: impl FnMut(i64) -> Result<T, MetricError>,
) -> Result<Vec<T>, MetricError> {
    let origin = Instant::now();
    let mut deadline = Duration::ZERO;
    let mut samples = Vec::new();
    loop {
        std::thread::sleep(deadline.saturating_sub(origin.elapsed()));
        let elapsed = origin.elapsed();
        samples.push(sample(
            i64::try_from(elapsed.as_millis()).map_err(|_| MetricError::InvalidWindow)?,
        )?);
        if deadline >= duration {
            break;
        }
        let next_second = origin
            .elapsed()
            .as_secs()
            .checked_add(1)
            .ok_or(MetricError::InvalidWindow)?;
        deadline = Duration::from_secs(next_second).min(duration);
    }
    Ok(samples)
}

const _: () = assert!(SAMPLE_INTERVAL.as_millis() == 1000);
