//! Metrics/display panel sub-module — UI panels for health, architecture,
//! evolution, test gaps, rules, what-if, DSM, and activity display.
//!
//! All files in this module were extracted from `src/app/` to improve
//! module cohesion. They form a natural cluster: `metrics_panel.rs`
//! orchestrates the others, and most use `ui_helpers::dim_grade_color`.

pub(crate) mod activity_panel;
pub(crate) mod arch_display;
pub(crate) mod dsm_panel;
pub(crate) mod evolution_display;
pub(crate) mod file_detail;
pub(crate) mod health_display;
pub(crate) mod language_summary;
pub(crate) mod metrics_panel;
pub(crate) mod rules_display;
pub(crate) mod testgap_display;
pub(crate) mod ui_helpers;
pub(crate) mod whatif_display;
