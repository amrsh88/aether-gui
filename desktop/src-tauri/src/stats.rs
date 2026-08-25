//! Throughput sampler.
//!
//! Counters are cumulative totals; the UI wants a rate. Sampling once a second and
//! dividing by the real elapsed time (not the nominal interval) keeps the reported
//! speed honest even when the machine is loaded and the timer slips.

use std::sync::Arc;
use std::time::Instant;

use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::model::{events, StatsEvent};
use crate::tun::adapter::Counters;

const INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

/// Emit a `StatsEvent` every second until cancelled.
pub fn spawn(
    app: AppHandle,
    counters: Arc<Counters>,
    cancel: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut last = counters.snapshot();
        let mut last_at = Instant::now();

        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => break,
                _ = tokio::time::sleep(INTERVAL) => {}
            }

            let now = counters.snapshot();
            let at = Instant::now();
            let elapsed = at.duration_since(last_at).as_secs_f64();

            // A zero or negative interval would divide by ~0 and spike the graph.
            if elapsed <= f64::EPSILON {
                continue;
            }

            // `saturating_sub` guards against a counter reset between samples,
            // which would otherwise wrap and report an absurd rate.
            let down_delta = now.0.saturating_sub(last.0);
            let up_delta = now.1.saturating_sub(last.1);

            let event = StatsEvent {
                down_bps: down_delta as f64 / elapsed,
                up_bps: up_delta as f64 / elapsed,
                total_down: now.0,
                total_up: now.1,
            };

            if app.emit(events::STATS, event).is_err() {
                // The window is gone; nothing left to report to.
                break;
            }

            last = now;
            last_at = at;
        }
    })
}
