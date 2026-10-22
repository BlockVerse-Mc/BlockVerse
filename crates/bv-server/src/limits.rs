use std::time::{Duration, Instant};

pub struct RateLimiter {
    window_start: Instant,
    count: u64,
    max_per_second: u64,
}

impl RateLimiter {
    pub fn new(max_per_second: u64, now: Instant) -> Self {
        Self {
            window_start: now,
            count: 0,
            max_per_second,
        }
    }

    pub fn allow(&mut self, amount: u64, now: Instant) -> bool {
        if now.duration_since(self.window_start) >= Duration::from_secs(1) {
            self.window_start = now;
            self.count = 0;
        }
        self.count = self.count.saturating_add(amount);
        self.count <= self.max_per_second
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traffic_within_the_limit_is_allowed() {
        let start = Instant::now();
        let mut limiter = RateLimiter::new(10, start);
        assert!(limiter.allow(4, start));
        assert!(limiter.allow(6, start));
    }

    #[test]
    fn traffic_over_the_limit_is_rejected() {
        let start = Instant::now();
        let mut limiter = RateLimiter::new(10, start);
        assert!(limiter.allow(10, start));
        assert!(!limiter.allow(1, start));
    }

    #[test]
    fn the_window_resets_after_one_second() {
        let start = Instant::now();
        let mut limiter = RateLimiter::new(5, start);
        assert!(!limiter.allow(50, start));
        let later = start + Duration::from_millis(1500);
        assert!(limiter.allow(5, later));
        assert!(!limiter.allow(1, later));
    }

    #[test]
    fn huge_amounts_do_not_overflow() {
        let start = Instant::now();
        let mut limiter = RateLimiter::new(5, start);
        assert!(!limiter.allow(u64::MAX, start));
        assert!(!limiter.allow(u64::MAX, start));
    }
}
