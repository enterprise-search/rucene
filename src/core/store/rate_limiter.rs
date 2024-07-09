use std::time::Duration;
use crate::Result;
use std::sync::Arc;

/// Trait base class to rate limit IO.
///
/// Typically implementations are shared across multiple IndexInputs
/// or IndexOutputs (for example those involved all merging).  Those IndexInputs and
/// IndexOutputs would call {@link #pause} whenever the have read
/// or written more than {@link #getMinPauseCheckBytes} bytes.

pub trait RateLimiter: Sync + Send {
    /// Sets an updated MB per second rate limit.
    fn set_mb_per_sec(&self, mb_per_sec: f64);

    /// The current MB per second rate limit.
    fn mb_per_sec(&self) -> f64;

    /// Pauses, if necessary, to keep the instantaneous IO rate
    /// at or below the target
    ///
    /// Note: the implementation is thread-safe
    fn pause(&self, bytes: u64) -> Result<Duration>;

    /// how many bytes caller should add up isself before invoking `#pause`
    fn min_pause_check_bytes(&self) -> u64;
}

impl RateLimiter for Arc<dyn RateLimiter> {
    fn set_mb_per_sec(&self, mb_per_sec: f64) {
        (**self).set_mb_per_sec(mb_per_sec);
    }

    fn mb_per_sec(&self) -> f64 {
        (**self).mb_per_sec()
    }

    fn pause(&self, bytes: u64) -> Result<Duration> {
        (**self).pause(bytes)
    }

    fn min_pause_check_bytes(&self) -> u64 {
        (**self).min_pause_check_bytes()
    }
}
