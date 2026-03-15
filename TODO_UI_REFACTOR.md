# UI Refactor — Next Session

## 1. Layout Reorganization
- LEFT panel: grades + language stats (always visible)
- RIGHT panel: context-sensitive
  - No file selected → show Recent Activity
  - File selected → show File Detail
- Remove the separate activity panel toggle

## 2. Git Status Colors (professional, low-saturation)
Use VS Code-style dark theme colors:
- Added:    #73C991 (muted green)
- Modified: #6796E6 (muted blue)  
- Deleted:  #E06C75 (muted red)
- Renamed:  #D19A66 (muted amber)
- Untracked: #73C991 with dotted border

Apply to: treemap rect borders, activity panel indicators

## 3. Activity Panel Enhancement
Currently: just blink on change
Should show per-entry:
- Change type indicator (+ / ~ / - with color)
- File name
- Time ago
- Delta: +12 -3 lines (green/red)
- Function changes: +2 fn, -1 fn
- Complexity change: CC 5→8

## 4. Breadcrumb bug
Fixed in this session (commit 39ff1da).

## Data Available
- `gs` field: "A", "M", "D", "R", "MM", "?"
- HeatTracker: per-file change times, trail
- StructuralAnalysis: can diff old vs new for +/- functions/lines
- Line counts: lines, logic, comments, blanks

## Files to Modify
- sentrux-core/src/app/panels/metrics_panel.rs (left panel)
- sentrux-core/src/app/panels/activity_panel.rs (right panel)  
- sentrux-core/src/app/panels/file_detail.rs (right panel, conditional)
- sentrux-core/src/app/draw_panels.rs (layout orchestration)
- sentrux-core/src/renderer/rects.rs (git status border colors)
- sentrux-core/src/core/heat.rs (store change type in trail)
