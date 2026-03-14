//! Central application state — single source of truth for the UI.
//!
//! `AppState` is owned exclusively by the main thread. Worker threads
//! (scanner, layout) communicate via channels and never touch this directly.
//! All fields are public for UI code simplicity; access is serialized by
//! the single-threaded egui event loop.

use crate::metrics::arch::ArchReport;
use crate::metrics::dsm::{DesignStructureMatrix, DsmStats};
use crate::metrics::evo::EvolutionReport;
use crate::metrics::testgap::TestGapReport;
use crate::metrics::rules::checks::RuleCheckResult;
use crate::layout::types::{EdgeFilter, FocusMode, LayoutMode, RenderData, ScaleMode, SizeMode};
use crate::metrics::HealthReport;
use crate::layout::types::ColorMode;
use crate::core::heat::HeatTracker;
use crate::layout::spatial_index::SpatialIndex;
use crate::core::settings::{Theme, ThemeConfig};
use crate::layout::viewport::ViewportTransform;
use crate::core::settings::Settings;
use crate::core::snapshot::Snapshot;
use crate::core::types::FileIndexEntry;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

/// All mutable UI state — owned exclusively by the main thread.
/// Fields are grouped by concern. Worker threads never touch this directly;
/// they communicate via the typed channels in `channels.rs`.
pub struct AppState {
    // ── Scan state ──
    /// Absolute path of the currently scanned root directory
    pub root_path: Option<String>,
    /// Current scan step description (shown in progress UI)
    pub scan_step: String,
    /// Scan progress percentage (0-100)
    pub scan_pct: u8,
    /// Whether a scan is currently in progress
    pub scanning: bool,

    // ── Data ──
    /// Latest complete scan snapshot (file tree + graphs)
    pub snapshot: Option<Arc<Snapshot>>,
    /// Pre-computed layout data (rects + edges) ready for rendering
    pub render_data: Option<RenderData>,
    /// Per-file metadata index for O(1) lookup by path
    pub file_index: HashMap<String, FileIndexEntry>,

    // ── Viewport ──
    /// Current pan/zoom state for world→screen coordinate transform
    pub viewport: ViewportTransform,
    /// Grid-based spatial index for O(1) hit testing
    pub spatial_index: Option<SpatialIndex>,

    // ── Interaction ──
    /// File path currently under the mouse cursor
    pub hovered_path: Option<String>,
    /// File path currently selected (clicked)
    pub selected_path: Option<String>,
    /// Drill-down navigation stack (directory path prefixes)
    pub drill_stack: Vec<String>,

    // ── Pan state ──
    /// Whether the user is currently dragging to pan
    pub dragging: bool,
    /// Screen position where the drag started
    pub drag_start_screen: Option<egui::Pos2>,
    /// Viewport offset at drag start (for delta computation)
    pub drag_start_offset: Option<(f64, f64)>,
    /// When the last interaction occurred (for idle detection)
    pub last_interaction: Instant,
    /// Whether the user is actively interacting (reduces LOD)
    pub interacting: bool,

    // ── Settings ──
    /// Active size metric for file block area
    pub size_mode: SizeMode,
    /// Active scaling transform for size compression
    pub scale_mode: ScaleMode,
    /// Active spatial layout algorithm
    pub layout_mode: LayoutMode,
    /// Active color mode for file blocks
    pub color_mode: ColorMode,
    /// Active visual theme
    pub theme: Theme,
    /// Resolved theme colors for the active theme
    pub theme_config: ThemeConfig,
    /// Active edge type filter
    pub edge_filter: EdgeFilter,
    /// Whether to show all edges or only spotlight edges
    pub show_all_edges: bool,
    /// Active focus/filter mode (all files, directory, language, etc.)
    pub focus_mode: FocusMode,
    /// User-tunable rendering parameters
    pub settings: Settings,
    /// Whether the settings panel is currently open
    pub settings_open: bool,

    // ── Layout pending ──
    /// Whether a layout recomputation is needed
    pub layout_pending: bool,
    /// A layout request was dropped (channel Full) and the data needs re-layout.
    /// Unlike `layout_pending`, this is NOT cleared by the result handler —
    /// it's only cleared when a retry succeeds. Prevents edges from being
    /// permanently lost when Complete's layout request is dropped.
    pub layout_request_dropped: bool,
    /// Monotonically increasing layout version counter
    pub layout_version: u64,
    /// Version of the most recently rendered layout
    pub rendered_version: u64,
    /// Throttle layout retry to avoid hot-looping when channel is full
    pub layout_retry_at: Option<Instant>,
    /// Throttle scan retry to avoid 60fps hot-loop when scanner channel is full
    pub scan_retry_at: Option<Instant>,

    // ── Heat / live updates ──
    /// Tracks per-file edit heat with exponential decay
    pub heat: HeatTracker,

    // ── Animation ──
    /// Monotonic animation time in seconds (updated each frame)
    pub anim_time: f64,
    /// Instant when animation started (for anim_time computation)
    pub anim_start: Instant,

    /// BUG 4 fix: current UNIX epoch time in seconds, computed once per frame
    /// instead of per-file in file_color(). Eliminates ~120k syscalls/sec at 60fps.
    pub frame_now_secs: f64,

    /// Monotonic frame instant — computed once per frame for heat/ripple queries.
    /// Avoids calling Instant::now() per-file (~2000 syscalls/frame for 1000 files).
    pub frame_instant: Instant,

    // ── Rescan accumulator ──
    /// Paths changed since last rescan (accumulated from watcher events) — HashSet for O(1) dedup
    pub pending_changes: HashSet<String>,
    /// When the first pending change arrived
    pub pending_since: Option<Instant>,

    // ── Derived data for focus/context dropdowns ──
    /// Top-level directories found in snapshot (for focus dropdown)
    pub top_dirs: Vec<String>,
    /// Languages found in snapshot (for focus dropdown)
    pub languages: Vec<String>,
    /// Entry-point file paths (for focus mode) — Arc for O(1) clone into layout requests
    pub entry_point_files: Arc<HashSet<String>>,

    // ── Activity panel ──
    /// Recent file events from watcher (newest first, capped)
    pub recent_activity: Vec<ActivityEntry>,
    /// Whether the activity panel is visible
    pub activity_panel_open: bool,
    /// Whether the DSM panel is visible
    pub dsm_panel_open: bool,
    /// Cached DSM matrix + stats, keyed by rendered_version to avoid O(N^2) per-frame rebuild
    pub dsm_cache: Option<(u64, DesignStructureMatrix, DsmStats)>,
    /// Cached top connected files, keyed by (rendered_version, edge_filter) to avoid O(E) per-frame rebuild
    pub top_connections_cache: Option<(u64, u8, Vec<(String, usize)>)>,

    // ── Language stats (cached per scan) ──
    pub lang_stats: Vec<(String, crate::app::panels::language_summary::LangStat)>,

    // ── Health metrics ──
    /// Code health report — recomputed on each ScanMsg::Complete
    pub health_report: Option<HealthReport>,
    /// Architecture report — recomputed on each ScanMsg::Complete
    pub arch_report: Option<ArchReport>,
    /// Evolution report — churn, bus factor, hotspots, change coupling
    pub evolution_report: Option<EvolutionReport>,
    /// Test gap report — coverage ratio, riskiest untested files
    pub test_gap_report: Option<TestGapReport>,
    /// Architecture rules check result
    pub rule_check_result: Option<RuleCheckResult>,
    /// Cached what-if simulation result for the selected file
    pub(crate) whatif_cache: Option<super::panels::whatif_display::WhatIfCache>,
    /// Pre-computed impact files for ImpactRadius focus mode (transitive dependents).
    pub impact_files: Option<Arc<HashSet<String>>>,

    /// BUG 2 fix: flag set by toolbar when "Open Folder" is clicked.
    /// The app handles the actual dialog on a background thread to avoid
    /// blocking the UI (especially on Linux where rfd blocks the event loop).
    pub folder_picker_requested: bool,
    /// Flag set by toolbar Rescan button — triggers re-scan of current project.
    pub rescan_requested: bool,
    /// Search query for file filtering.
    pub search_query: String,

    // ── Context menu / hide ──
    /// Paths hidden by the user (files or directory prefixes). Files whose path
    /// matches or starts with a hidden prefix get weight 0 in layout.
    /// Wrapped in Arc for O(1) clone into layout requests.
    pub hidden_paths: Arc<HashSet<String>>,
    /// Path under the pointer when context menu was opened (file or section).
    pub context_menu_target: Option<ContextMenuTarget>,

}

/// A recent file change event for the activity panel.
pub struct ActivityEntry {
    /// Relative file path of the changed file
    pub path: String,
    /// Event kind: "create", "modify", or "remove"
    pub kind: String,
    /// When the event occurred (monotonic clock)
    pub time: Instant,
}

/// Target of a right-click context menu.
#[derive(Debug, Clone)]
pub struct ContextMenuTarget {
    /// File or directory path that was right-clicked
    pub path: String,
    /// True if this is a directory/section, false if a file
    pub is_dir: bool,
}

// FileIndexEntry moved to core::types (re-exported via use above)

/// Compute current UNIX epoch seconds, with graceful fallback.
fn now_epoch_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|e| {
            eprintln!("[state] system clock before epoch: {}", e);
            std::time::Duration::ZERO
        })
        .as_secs_f64()
}

impl AppState {
    /// Create a new AppState with default settings and no active scan.
    pub fn new() -> Self {
        let theme = Theme::Calm;
        let now = Instant::now();
        Self {
            root_path: None,
            scan_step: String::new(),
            scan_pct: 0,
            scanning: false,
            snapshot: None,
            render_data: None,
            file_index: HashMap::new(),
            viewport: ViewportTransform::new(),
            spatial_index: None,
            hovered_path: None,
            selected_path: None,
            drill_stack: Vec::new(),
            dragging: false,
            drag_start_screen: None,
            drag_start_offset: None,
            last_interaction: now,
            interacting: false,
            size_mode: SizeMode::Lines,
            scale_mode: ScaleMode::Smooth,
            layout_mode: LayoutMode::Treemap,
            color_mode: ColorMode::Monochrome,
            theme,
            theme_config: ThemeConfig::from_theme(theme),
            edge_filter: EdgeFilter::All,
            show_all_edges: false,
            focus_mode: FocusMode::All,
            settings: Settings::default(),
            settings_open: false,
            layout_pending: false,
            layout_request_dropped: false,
            layout_version: 0,
            rendered_version: 0,
            layout_retry_at: None,
            scan_retry_at: None,
            heat: HeatTracker::new(),
            anim_time: 0.0,
            anim_start: now,
            frame_now_secs: now_epoch_secs(),
            frame_instant: now,
            pending_changes: HashSet::new(),
            pending_since: None,
            top_dirs: Vec::new(),
            languages: Vec::new(),
            entry_point_files: Arc::new(HashSet::new()),
            recent_activity: Vec::new(),
            activity_panel_open: false,
            dsm_panel_open: false,
            dsm_cache: None,
            top_connections_cache: None,
            lang_stats: Vec::new(),
            health_report: None,
            arch_report: None,
            evolution_report: None,
            test_gap_report: None,
            rule_check_result: None,
            whatif_cache: None,
            impact_files: None,
            folder_picker_requested: false,
            rescan_requested: false,
            search_query: String::new(),
            hidden_paths: Arc::new(HashSet::new()),
            context_menu_target: None,
        }
    }

    /// Record a file event in the activity panel (newest first, capped at 50).
    /// Deduplicates: if the same path already exists, removes old entry first.
    pub fn record_activity(&mut self, path: String, kind: String) {
        const MAX_ACTIVITY: usize = 50;
        // Dedup: find and remove existing entry for this path
        if let Some(pos) = self.recent_activity.iter().position(|e| e.path == path) {
            // Use remove() not swap_remove() to preserve newest-first ordering. [H7 fix]
            self.recent_activity.remove(pos);
        }
        // Insert at front (newest first)
        self.recent_activity.insert(0, ActivityEntry {
            path,
            kind,
            time: Instant::now(),
        });
        // Cap size
        self.recent_activity.truncate(MAX_ACTIVITY);
    }

    /// Check if a path is hidden (exact match or starts with a hidden directory prefix).
    #[allow(dead_code)] // Called from canvas interaction; kept for hide/show feature
    pub fn is_hidden(&self, path: &str) -> bool {
        if self.hidden_paths.contains(path) {
            return true;
        }
        // Check directory prefixes: "src" hides "src/foo.rs"
        for hp in self.hidden_paths.iter() {
            if path.starts_with(hp.as_str()) && path.as_bytes().get(hp.len()) == Some(&b'/') {
                return true;
            }
        }
        false
    }

    /// Apply a new theme — updates theme_config.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.theme_config = ThemeConfig::from_theme(theme);
    }

    /// Build a FileIndexEntry from a FileNode.
    fn file_to_index_entry(f: &crate::core::types::FileNode) -> FileIndexEntry {
        FileIndexEntry {
            lines: f.lines,
            logic: f.logic,
            funcs: f.funcs,
            lang: f.lang.clone(),
            gs: f.gs.clone(),
            mtime: f.mtime,
            stats_line: format!("{}ln {}fn {}cd", f.lines, f.funcs, f.logic),
        }
    }

    /// Build file_index from snapshot for O(1) lookup.
    /// Also rebuilds derived data: top_dirs, languages, entry_point_files.
    pub fn rebuild_file_index(&mut self) {
        self.file_index.clear();
        self.top_dirs.clear();
        self.languages.clear();
        let mut ep_files = HashSet::new();

        let snap = match &self.snapshot {
            Some(s) => s,
            None => return,
        };

        let files = crate::core::snapshot::flatten_files_ref(&snap.root);
        self.file_index.reserve(files.len());

        let mut dir_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut lang_set: std::collections::HashSet<String> = std::collections::HashSet::new();

        for f in &files {
            self.file_index.insert(f.path.clone(), Self::file_to_index_entry(f));
            if let Some(slash) = f.path.find('/') {
                dir_set.insert(f.path[..slash].to_string());
            }
            if !f.lang.is_empty() && f.lang != "unknown" {
                lang_set.insert(f.lang.clone());
            }
        }

        for ep in &snap.entry_points {
            ep_files.insert(ep.file.clone());
        }
        self.entry_point_files = Arc::new(ep_files);

        self.top_dirs = dir_set.into_iter().collect();
        self.top_dirs.sort_unstable();
        self.languages = lang_set.into_iter().collect();
        self.languages.sort_unstable();
    }
}
