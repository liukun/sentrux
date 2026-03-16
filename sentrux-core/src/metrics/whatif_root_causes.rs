//! What-if prediction using root causes (PRO).
//!
//! Predicts how the quality_signal would change if a specific refactoring
//! action is taken — BEFORE actually doing it. Like rocket trajectory
//! prediction: "if I apply thrust X, position changes by Y."
//!
//! This enables the AI agent to pick the BEST action from multiple options
//! instead of trial-and-error. Faster convergence = less compute = less cost.
//!
//! Theory: 钱学森 predictive control — model the system, predict outcomes,
//! choose optimal action. The "model" is: recompute root causes on a
//! hypothetical modified graph.
//!
//! STATUS: Skeleton — to be implemented.
//! TIER: Pro (gated by tier.is_pro())

// TODO: Implement what-if prediction with root causes
//
// Approach:
//   1. Take current snapshot + proposed action (move file, extract module, etc.)
//   2. Apply action to a CLONE of the dependency graph (don't modify real data)
//   3. Recompute all 5 root causes on the modified graph
//   4. Return delta for each root cause and predicted new quality_signal
//
// Actions to support:
//   - move_file(path, new_module)     → recompute Q, may fix cycles
//   - extract_module(files, new_name) → recompute Q, depth
//   - delete_file(path)               → recompute redundancy, Q
//   - merge_modules(a, b)             → recompute Q, depth
//
// Output:
//   struct WhatIfPrediction {
//       action: String,
//       signal_before: f64,
//       signal_after: f64,
//       signal_delta: f64,
//       root_cause_deltas: RootCauseDeltas,
//       recommended: bool,  // signal_after > signal_before
//   }
//
// MCP tool:
//   name: "whatif_root"
//   input: { "action": "move_file", "file": "src/utils.rs", "to_module": "src/helpers" }
//   output: { "signal_delta": +0.03, "modularity_delta": +0.05, ... }
//   tier: Pro
//
// Integration:
//   - Registered as MCP tool in tools.rs
//   - Uses root_causes::compute_modularity_q etc. on modified graph
//   - Existing whatif module can be refactored to use this
