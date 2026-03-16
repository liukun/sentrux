//! Individual rule-check helpers for the architecture rules engine.
//!
//! Extracted from rules.rs to keep each file under 500 lines.
//! Each function checks one constraint and returns violations (if any).

use crate::metrics::arch::ArchReport;
use crate::metrics::types::HealthReport;
use serde::Deserialize;

/// Structural constraints parsed from `.sentrux/rules.toml`.
/// Each field is optional — only set constraints are checked.
///
/// Two levels of gating:
///   1. Root cause gates — minimum per-dimension scores (0.0-1.0)
///   2. Specific limits — hard engineering constraints on individual metrics
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Constraints {
    // ── Root cause gates (minimum scores, 0.0-1.0) ──
    /// Required minimum quality signal (geometric mean of 5 root causes)
    pub min_quality: Option<f64>,
    /// Required minimum modularity score (Newman's Q normalized)
    pub min_modularity: Option<f64>,
    /// Required minimum acyclicity score (1.0 = zero cycles)
    pub min_acyclicity: Option<f64>,
    /// Required minimum depth score (1.0 = shallow)
    pub min_depth: Option<f64>,
    /// Required minimum equality score (1.0 = evenly distributed complexity)
    pub min_equality: Option<f64>,
    /// Required minimum redundancy score (1.0 = no dead/duplicate code)
    pub min_redundancy: Option<f64>,

    // ── Specific engineering limits ──
    /// Required maximum coupling score (e.g., 0.35)
    pub max_coupling_score: Option<f64>,
    /// Maximum allowed circular dependency cycles
    pub max_cycles: Option<usize>,
    /// Maximum cyclomatic complexity per function
    pub max_cc: Option<u32>,
    /// Maximum lines per file
    pub max_file_lines: Option<u32>,
    /// Maximum lines per function
    pub max_fn_lines: Option<u32>,
    /// Whether god files (fan-out > threshold) are forbidden
    #[serde(default)]
    pub no_god_files: bool,
    /// Maximum allowed upward dependency violations
    pub max_upward_violations: Option<usize>,
}

impl Constraints {
    /// Count how many constraint thresholds are actively set.
    pub fn count_active(&self) -> usize {
        let mut n = 0;
        if self.min_quality.is_some() { n += 1; }
        if self.min_modularity.is_some() { n += 1; }
        if self.min_acyclicity.is_some() { n += 1; }
        if self.min_depth.is_some() { n += 1; }
        if self.min_equality.is_some() { n += 1; }
        if self.min_redundancy.is_some() { n += 1; }
        if self.max_coupling_score.is_some() { n += 1; }
        if self.max_cycles.is_some() { n += 1; }
        if self.max_cc.is_some() { n += 1; }
        if self.max_file_lines.is_some() { n += 1; }
        if self.max_fn_lines.is_some() { n += 1; }
        if self.no_god_files { n += 1; }
        if self.max_upward_violations.is_some() { n += 1; }
        n
    }

    /// Merge language-specific overrides into this constraint set.
    pub fn merge(&self, override_with: &Constraints) -> Constraints {
        Constraints {
            min_quality: override_with.min_quality.or(self.min_quality),
            min_modularity: override_with.min_modularity.or(self.min_modularity),
            min_acyclicity: override_with.min_acyclicity.or(self.min_acyclicity),
            min_depth: override_with.min_depth.or(self.min_depth),
            min_equality: override_with.min_equality.or(self.min_equality),
            min_redundancy: override_with.min_redundancy.or(self.min_redundancy),
            max_coupling_score: override_with.max_coupling_score.or(self.max_coupling_score),
            max_cycles: override_with.max_cycles.or(self.max_cycles),
            max_cc: override_with.max_cc.or(self.max_cc),
            max_file_lines: override_with.max_file_lines.or(self.max_file_lines),
            max_fn_lines: override_with.max_fn_lines.or(self.max_fn_lines),
            no_god_files: override_with.no_god_files || self.no_god_files,
            max_upward_violations: override_with.max_upward_violations.or(self.max_upward_violations),
        }
    }
}

/// Result of running all architecture rules against a codebase.
#[derive(Debug, Clone)]
pub struct RuleCheckResult {
    /// Whether all checked rules passed
    pub passed: bool,
    /// List of rule violations found
    pub violations: Vec<RuleViolation>,
    /// Number of rules that were checked
    pub rules_checked: usize,
}

/// A single rule violation with context about what failed.
#[derive(Debug, Clone)]
pub struct RuleViolation {
    /// Rule name (e.g., "max_cycles", "no_god_files")
    pub rule: String,
    /// Severity level (error or warning)
    pub severity: Severity,
    /// Human-readable violation description
    pub message: String,
    /// Files involved in the violation
    pub files: Vec<String>,
}

/// Severity level for rule violations.
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    /// Hard failure — the gate should reject
    Error,
    /// Soft failure — informational, does not block the gate
    Warning,
}

/// Check minimum quality signal.
pub fn check_min_quality(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    let min_quality = c.min_quality?;
    if health.quality_signal < min_quality {
        Some(RuleViolation {
            rule: "min_quality".into(),
            severity: Severity::Error,
            message: format!("Quality signal {:.2} below minimum required {:.2}", health.quality_signal, min_quality),
            files: vec![],
        })
    } else {
        None
    }
}

/// Check maximum coupling score.
pub fn check_max_coupling(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    let max_coupling = c.max_coupling_score?;
    if health.coupling_score > max_coupling {
        Some(RuleViolation {
            rule: "max_coupling_score".into(),
            severity: Severity::Error,
            message: format!("Coupling score {:.2} exceeds maximum allowed {:.2}", health.coupling_score, max_coupling),
            files: vec![],
        })
    } else {
        None
    }
}

/// Check circular dependency count.
pub fn check_max_cycles(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    let max_cycles = c.max_cycles?;
    if health.circular_dep_count > max_cycles {
        Some(RuleViolation {
            rule: "max_cycles".into(),
            severity: Severity::Error,
            message: format!("Found {} circular dependencies, maximum allowed is {}", health.circular_dep_count, max_cycles),
            files: health.circular_dep_files.iter().flatten().cloned().collect(),
        })
    } else {
        None
    }
}

/// Check maximum cyclomatic complexity per function.
/// Uses `all_function_ccs` (unfiltered) so user thresholds stricter than the
/// hardcoded CC_THRESHOLD_HIGH (15) are correctly enforced.
pub fn check_max_cc(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    let max_cc = c.max_cc?;
    let over: Vec<_> = health.all_function_ccs.iter().filter(|f| f.value > max_cc).collect();
    if !over.is_empty() {
        Some(RuleViolation {
            rule: "max_cc".into(),
            severity: Severity::Error,
            message: format!("{} function(s) exceed max cyclomatic complexity of {}", over.len(), max_cc),
            files: over.iter().map(|f| format!("{}:{} (cc={})", f.file, f.func, f.value)).collect(),
        })
    } else {
        None
    }
}

/// Check maximum file length in lines.
/// Uses `all_file_lines` (unfiltered) so user thresholds stricter than the
/// hardcoded LARGE_FILE_THRESHOLD (500) are correctly enforced.
pub fn check_max_file_lines(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    let max_file_lines = c.max_file_lines?;
    let over: Vec<_> = health.all_file_lines.iter().filter(|f| f.value > max_file_lines as usize).collect();
    if !over.is_empty() {
        Some(RuleViolation {
            rule: "max_file_lines".into(),
            severity: Severity::Error,
            message: format!("{} file(s) exceed max length of {} lines", over.len(), max_file_lines),
            files: over.iter().map(|f| format!("{} ({} lines)", f.path, f.value)).collect(),
        })
    } else {
        None
    }
}

/// Check maximum function length in lines.
/// Uses `all_function_lines` (unfiltered) so user thresholds stricter than the
/// hardcoded FUNC_LENGTH_THRESHOLD (50) are correctly enforced.
pub fn check_max_fn_lines(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    let max_fn_lines = c.max_fn_lines?;
    let over: Vec<_> = health.all_function_lines.iter().filter(|f| f.value > max_fn_lines).collect();
    if !over.is_empty() {
        Some(RuleViolation {
            rule: "max_fn_lines".into(),
            severity: Severity::Error,
            message: format!("{} function(s) exceed max length of {} lines", over.len(), max_fn_lines),
            files: over.iter().map(|f| format!("{}:{} ({} lines)", f.file, f.func, f.value)).collect(),
        })
    } else {
        None
    }
}

/// Check for god files (high fan-out).
pub fn check_no_god_files(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    if !c.no_god_files {
        return None;
    }
    if !health.god_files.is_empty() {
        Some(RuleViolation {
            rule: "no_god_files".into(),
            severity: Severity::Error,
            message: format!("{} god file(s) found (fan-out > 15)", health.god_files.len()),
            files: health.god_files.iter().map(|f| format!("{} (fan-out={})", f.path, f.value)).collect(),
        })
    } else {
        None
    }
}

// ── Root cause gates ──

fn check_root_cause(name: &str, score: f64, min: Option<f64>) -> Option<RuleViolation> {
    let min = min?;
    if score < min {
        Some(RuleViolation {
            rule: format!("min_{}", name),
            severity: Severity::Error,
            message: format!("{} score {:.4} below minimum required {:.4}", name, score, min),
            files: vec![],
        })
    } else {
        None
    }
}

/// Check minimum modularity score.
pub fn check_min_modularity(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    check_root_cause("modularity", health.root_cause_scores.modularity, c.min_modularity)
}

/// Check minimum acyclicity score.
pub fn check_min_acyclicity(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    check_root_cause("acyclicity", health.root_cause_scores.acyclicity, c.min_acyclicity)
}

/// Check minimum depth score.
pub fn check_min_depth(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    check_root_cause("depth", health.root_cause_scores.depth, c.min_depth)
}

/// Check minimum equality score.
pub fn check_min_equality(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    check_root_cause("equality", health.root_cause_scores.equality, c.min_equality)
}

/// Check minimum redundancy score.
pub fn check_min_redundancy(c: &Constraints, health: &HealthReport) -> Option<RuleViolation> {
    check_root_cause("redundancy", health.root_cause_scores.redundancy, c.min_redundancy)
}

/// Check maximum upward dependency violations.
pub fn check_max_upward(c: &Constraints, arch: &ArchReport) -> Option<RuleViolation> {
    let max_upward = c.max_upward_violations?;
    if arch.upward_violations.len() > max_upward {
        Some(RuleViolation {
            rule: "max_upward_violations".into(),
            severity: Severity::Error,
            message: format!("{} upward dependency violations, maximum allowed is {}", arch.upward_violations.len(), max_upward),
            files: arch.upward_violations.iter().take(5)
                .map(|v| format!("{} (L{}) \u{2192} {} (L{})", v.from_file, v.from_level, v.to_file, v.to_level))
                .collect(),
        })
    } else {
        None
    }
}
