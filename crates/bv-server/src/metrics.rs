use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tokio::time::MissedTickBehavior;

use crate::logger;
use crate::shared::Shared;

pub struct Metrics {
    started: Instant,
    pub connections: AtomicU64,
    pub login_failures: AtomicU64,
    pub packets_in: AtomicU64,
    pub chunks_sent: AtomicU64,
    lag_last_us: AtomicU64,
    lag_max_us: AtomicU64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            connections: AtomicU64::new(0),
            login_failures: AtomicU64::new(0),
            packets_in: AtomicU64::new(0),
            chunks_sent: AtomicU64::new(0),
            lag_last_us: AtomicU64::new(0),
            lag_max_us: AtomicU64::new(0),
        }
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started.elapsed().as_secs()
    }

    pub fn add_packets(&self, n: u64) {
        self.packets_in.fetch_add(n, Ordering::Relaxed);
    }

    pub fn add_chunks(&self, n: u64) {
        self.chunks_sent.fetch_add(n, Ordering::Relaxed);
    }

    pub fn record_lag(&self, micros: u64) {
        self.lag_last_us.store(micros, Ordering::Relaxed);
        self.lag_max_us.fetch_max(micros, Ordering::Relaxed);
    }

    pub fn lag_last_ms(&self) -> f64 {
        self.lag_last_us.load(Ordering::Relaxed) as f64 / 1000.0
    }

    pub fn lag_max_ms(&self) -> f64 {
        self.lag_max_us.load(Ordering::Relaxed) as f64 / 1000.0
    }
}

pub fn format_uptime(secs: u64) -> String {
    let (d, h, m, s) = (secs / 86_400, (secs / 3600) % 24, (secs / 60) % 60, secs % 60);
    if d > 0 {
        format!("{d}d {h:02}:{m:02}:{s:02}")
    } else {
        format!("{h:02}:{m:02}:{s:02}")
    }
}

pub async fn monitor_loop_lag(shared: Arc<Shared>) {
    let period = Duration::from_millis(50);
    let mut interval = tokio::time::interval(period);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    interval.tick().await;
    let mut last = Instant::now();
    let mut last_warning: Option<Instant> = None;
    loop {
        interval.tick().await;
        let now = Instant::now();
        let lag = now.duration_since(last).saturating_sub(period);
        last = now;
        shared.metrics.record_lag(lag.as_micros() as u64);
        let warn_due = last_warning.is_none_or(|t| t.elapsed() >= Duration::from_secs(10));
        if lag >= Duration::from_millis(500) && warn_due {
            logger::warn(&format!("Server loop is lagging: {} ms", lag.as_millis()));
            last_warning = Some(now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uptime_format() {
        assert_eq!(format_uptime(0), "00:00:00");
        assert_eq!(format_uptime(3725), "01:02:05");
        assert_eq!(format_uptime(90_061), "1d 01:01:01");
    }

    #[test]
    fn lag_keeps_the_maximum() {
        let m = Metrics::new();
        m.record_lag(2_000);
        m.record_lag(9_000);
        m.record_lag(1_000);
        assert_eq!(m.lag_last_ms(), 1.0);
        assert_eq!(m.lag_max_ms(), 9.0);
    }

    #[test]
    fn chunk_counter_adds_up() {
        let m = Metrics::new();
        m.add_chunks(81);
        m.add_chunks(1);
        assert_eq!(m.chunks_sent.load(Ordering::Relaxed), 82);
    }

    #[test]
    fn packet_counter_adds_up() {
        let m = Metrics::new();
        m.add_packets(3);
        m.add_packets(4);
        assert_eq!(m.packets_in.load(Ordering::Relaxed), 7);
    }
}
