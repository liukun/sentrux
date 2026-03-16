//! File heat tracking — exponential-decay activity heatmap.
//!
//! Tracks per-file "heat" that spikes on each file save and decays
//! exponentially over time. Used for the Heat color mode, ripple animations,
//! and the activity trail display. All timing is based on `Instant` (monotonic).

use egui::Color32;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

/// Maximum heat value a file can accumulate (caps exponential stacking).
const MAX_HEAT: f64 = 5.0;

/// Maximum number of trail entries retained (prevents unbounded growth).
const MAX_TRAIL: usize = 500;

/// Per-file heat entry
struct HeatEntry {
    /// Initial heat level at last_change time (accumulated from rapid edits)
    initial_heat: f64,
    /// Current decayed heat level
    heat: f64,
    /// When the last change was recorded
    last_change: Instant,
    /// When the ripple animation started (None = no active ripple)
    ripple_start: Option<Instant>,
}

/// Tracks per-file heat (recent edit activity) with exponential decay.
pub struct HeatTracker {
    entries: HashMap<String, HeatEntry>,
    /// Recent change trail: (path, time) — kept for 30s. Uses VecDeque for O(1) front removal.
    pub trail: VecDeque<(String, Instant)>,
}

/// Heat config passed from Settings at call sites.
/// Avoids coupling HeatTracker to the Settings struct directly.
pub struct HeatConfig {
    /// Exponential decay half-life in seconds for heat values
    pub half_life: f64,
    /// Duration in seconds for the ripple border animation
    pub ripple_duration: f64,
    /// Maximum age in seconds before trail entries are pruned
    pub trail_max_age: f64,
}

impl HeatTracker {
    /// Create an empty heat tracker with no entries.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            trail: VecDeque::new(),
        }
    }

    /// Record a file change — boosts heat to 1.0 and starts a ripple.
    pub fn record_change(&mut self, path: &str, cfg: &HeatConfig) {
        let now = Instant::now();
        let entry = self.entries.entry(path.to_string()).or_insert(HeatEntry {
            initial_heat: 0.0,
            heat: 0.0,
            last_change: now,
            ripple_start: None,
        });
        // Compute live-decayed heat before accumulating, to avoid using the
        // stale cached value from the last tick(). [ref:4f5a9de5]
        let dt = now.duration_since(entry.last_change).as_secs_f64();
        let hl = cfg.half_life.max(0.001);
        let live_heat = entry.initial_heat * (-std::f64::consts::LN_2 * dt / hl).exp();
        entry.initial_heat = (live_heat + 1.0).min(MAX_HEAT);
        entry.heat = entry.initial_heat;
        entry.last_change = now;
        entry.ripple_start = Some(now);

        self.trail.push_back((path.to_string(), now));
        while self.trail.len() > MAX_TRAIL {
            self.trail.pop_front();
        }
    }

    /// Tick all entries — apply exponential decay, prune cold entries, trim trail.
    pub fn tick(&mut self, cfg: &HeatConfig) {
        let now = Instant::now();
        let ln2 = std::f64::consts::LN_2;

        self.entries.retain(|_, e| {
            let dt = now.duration_since(e.last_change).as_secs_f64();
            let hl = cfg.half_life.max(0.001);
            e.heat = e.initial_heat * (-ln2 * dt / hl).exp();

            // Clear expired ripple
            if let Some(rs) = e.ripple_start {
                if now.duration_since(rs).as_secs_f64() > cfg.ripple_duration {
                    e.ripple_start = None;
                }
            }

            e.heat > 0.01 // prune near-zero
        });

        // Trim old trail entries — pop from front since trail is time-ordered
        // (push_back with monotonically increasing timestamps). O(k) where k is
        // the number of expired entries, instead of O(n) full scan. [M5 fix]
        while let Some((_p, t)) = self.trail.front() {
            if now.duration_since(*t).as_secs_f64() >= cfg.trail_max_age {
                self.trail.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get current heat for a file (0.0–5.0 raw, 0.0 if not tracked).
    /// Computes live decay at query time for smooth animation (not stale
    /// cached value from last tick). [ref:4f5a9de5]
    /// Takes `now` to avoid calling Instant::now() per-file per-frame
    /// (previously ~2000 syscalls/frame for 1000 visible files).
    #[inline]
    pub fn get_heat(&self, path: &str, now: Instant, half_life: f64) -> f64 {
        self.entries.get(path).map_or(0.0, |e| {
            let dt = now.duration_since(e.last_change).as_secs_f64();
            let hl = half_life.max(0.001);
            e.initial_heat * (-std::f64::consts::LN_2 * dt / hl).exp()
        })
    }

    /// Get ripple progress for a file (0.0–1.0, None if no active ripple).
    /// Takes `now` for frame-consistent rendering (same timestamp for all files).
    #[inline]
    pub fn get_ripple(&self, path: &str, now: Instant, ripple_duration: f64) -> Option<f64> {
        self.entries.get(path).and_then(|e| {
            e.ripple_start.and_then(|rs| {
                let t = now.duration_since(rs).as_secs_f64() / ripple_duration.max(0.001);
                if t >= 1.0 {
                    None // Expired — don't issue draw calls for phantom shapes
                } else {
                    Some(t)
                }
            })
        })
    }

    /// True if any file has visually significant heat (above rendering threshold).
    /// Prevents infinite repaint loops from near-zero heat values.
    pub fn is_active(&self) -> bool {
        self.entries.values().any(|e| e.heat > 0.02)
    }

    /// True if any file has an active (unexpired) ripple animation.
    pub fn has_any_ripples(&self) -> bool {
        self.entries.values().any(|e| e.ripple_start.is_some())
    }

    /// Get all files with heat > threshold, sorted by heat descending.
    /// Uses live exponential decay (same as get_heat) for consistency. [ref:4f5a9de5]
    pub fn hot_files(&self, threshold: f64, now: Instant, half_life: f64) -> Vec<(&str, f64)> {
        let ln2 = std::f64::consts::LN_2;
        let mut v: Vec<(&str, f64)> = self
            .entries
            .iter()
            .map(|(k, e)| {
                let dt = now.duration_since(e.last_change).as_secs_f64();
                let hl = half_life.max(0.001);
                let live_heat = e.initial_heat * (-ln2 * dt / hl).exp();
                (k.as_str(), live_heat)
            })
            .filter(|(_, h)| *h > threshold)
            .collect();
        v.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v
    }
}

/// Heat value → color (cool blue → warm yellow → hot red).
/// BUG 12 fix: use 3-stop gradient across the full [0,1] range instead of
/// saturating at t=0.5. Previously heat values > 2.5 all mapped to nearly
/// the same red, compressing half the color space into identical output.
/// New stops: blue(0) → orange(0.33) → yellow(0.66) → red(1.0). [ref:93cf32d4]
pub fn heat_color(heat: f64) -> Color32 {
    let t = (heat / MAX_HEAT).clamp(0.0, 1.0) as f32;
    if t < 0.33 {
        // blue → orange
        let u = t / 0.33;
        let r = (40.0 + u * 215.0) as u8;
        let g = (60.0 + u * 100.0) as u8;
        let b = (180.0 - u * 180.0) as u8;
        Color32::from_rgb(r, g, b)
    } else if t < 0.66 {
        // orange → yellow
        let u = (t - 0.33) / 0.33;
        let r = 255;
        let g = (160.0 + u * 60.0) as u8; // 160 → 220
        let b = (0.0 + u * 20.0) as u8;   // 0 → 20
        Color32::from_rgb(r, g, b)
    } else {
        // yellow → deep red
        let u = (t - 0.66) / 0.34;
        let r = 255;
        let g = (220.0 - u * 190.0) as u8; // 220 → 30
        let b = (20.0 - u * 20.0) as u8;   // 20 → 0
        Color32::from_rgb(r, g, b)
    }
}

/// Ripple glow color with alpha based on ripple progress (fades out).
pub fn ripple_color(progress: f64) -> Color32 {
    let alpha = ((1.0 - progress) * 180.0) as u8;
    Color32::from_rgba_unmultiplied(255, 200, 80, alpha)
}
