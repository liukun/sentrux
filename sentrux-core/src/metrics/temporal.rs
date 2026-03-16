//! Temporal quality signal — dH/dt from git history (PRO).
//!
//! Measures the RATE of quality change over time. This is the deepest
//! possible signal: not "is the code good?" but "is it getting better
//! or worse, and how fast?"
//!
//! Theory: Thermodynamic entropy production rate. The second law says
//! entropy increases in closed systems. A codebase with good architecture
//! RESISTS entropy increase — dH/dt ≈ 0 or negative. A codebase with
//! poor architecture ACCUMULATES entropy — dH/dt >> 0.
//!
//! This answers the question the AI agent really needs:
//!   "Are my changes helping or hurting, and at what rate?"
//!
//! STATUS: Skeleton — to be implemented.
//! TIER: Pro (gated by tier.is_pro())

// TODO: Implement temporal quality signal
//
// Approach:
//   1. Walk git history (last N commits, or since baseline)
//   2. At each sampled commit, checkout and compute root causes
//      (or use cached values from previous scans)
//   3. Compute dQ/dt for each root cause:
//      - dQ/dt = (Q_now - Q_baseline) / num_commits
//      - Positive = improving, Negative = degrading
//   4. Compute overall dS/dt = d(quality_signal)/dt
//   5. Predict: "at current rate, signal reaches X in Y commits"
//
// Optimizations:
//   - Don't re-parse every commit — use git diff to incrementally
//     update the dependency graph
//   - Cache root cause scores per commit hash
//   - Sample commits (every 5th) for large histories
//
// Output:
//   struct TemporalSignal {
//       signal_velocity: f64,        // dS/dt per commit (positive = improving)
//       signal_acceleration: f64,    // d²S/dt² (positive = improvement accelerating)
//       commits_analyzed: usize,
//       prediction: Option<String>,  // "reaches 0.80 in ~15 commits" or "plateaus at 0.75"
//       per_root_cause: TemporalPerCause,
//   }
//
//   struct TemporalPerCause {
//       modularity_velocity: f64,
//       acyclicity_velocity: f64,
//       depth_velocity: f64,
//       equality_velocity: f64,
//       redundancy_velocity: f64,
//   }
//
// MCP response (added to health):
//   "temporal": {
//       "velocity": 0.005,           // +0.5% per commit
//       "acceleration": -0.001,      // slowing down
//       "prediction": "plateaus at 82% in ~20 commits",
//       "per_cause": {
//           "modularity": +0.008,
//           "redundancy": -0.002     // getting worse
//       }
//   }
//
// Integration:
//   - Computed after evolution report (needs git history)
//   - Added to MCP health response under "temporal" key
//   - GUI shows velocity arrow next to quality signal: ↑ ↓ →
//   - Pro only — requires git history walking (expensive)
