# testypf TODO

**Scope:** Build `testypf`, a minimal-yet-fast cross-platform GUI app showcasing typf rendering, typg discovery, and fontlift install flows. Requirements taken from `/Users/adam/Developer/vcs/TODO2.md` (Nov 16, 2025).

## Phase 0 – Foundations
- [ ] Audit `../typf`, `../fontlift`, and `../typg` APIs to understand hooks we can call from the GUI
- [ ] Evaluate GUI toolkits (iced, Slint, Dioxus, winit/tao + custom rendering) for required features: drag/drop folders, floating panels, transparent borderless window, multi-backend rendering
- [ ] Spike minimal prototypes for top two toolkit candidates; document timing + platform caveats
- [ ] Select toolkit + state architecture (Unidirectional data flow preferred) and record decision in `PLAN.md`

## Phase 1 – Spec & Planning
- [ ] Author `PLAN.md` describing overall UX, data flow, rendering pipeline, and integration points (typf/fontlift/typg)
- [ ] Define windowing strategy (floating list panel, floating control panel, borderless render window) + communication channels between them
- [ ] Outline performance targets (load/render latency, UI responsiveness) and instrumentation plan
- [ ] Capture color/typography guidelines to keep UI clean/minimal

## Phase 2 – Font Source Panel
- [ ] Implement drag/drop + “Add” button to ingest files/folders recursively (dedupe + path elision)
- [ ] Display fonts in sortable list (path, family, style) with multi-select + removal
- [ ] Persist list in session (temporary workspace) and expose actions to send selected fonts to fontlift install/uninstall
- [ ] Write tests covering ingestion edge cases (invalid files, duplicates, nested directories)

## Phase 3 – Control Panel
- [ ] Build floating control surface with inputs: sample text, font size, OpenType feature dropdown, backend selector (allow 2), foreground/background color pickers supporting transparency
- [ ] Implement dynamic variation sliders based on font axes metadata (auto-generated from typf data)
- [ ] Add toggles for backend pairing/comparison layout + caching behavior
- [ ] Guarantee controls are keyboard accessible and movable (drag-handle) without overlapping other panels permanently
- [ ] Add state synchronization tests to ensure control inputs propagate to renderer predictably

## Phase 4 – Render Window
- [ ] Create borderless, optionally transparent window that tiles render rows per font (scrollable, high-DPI aware)
- [ ] Pipe typf rendering outputs for the selected backend(s); when two backends chosen, show side-by-side comparison per font
- [ ] Support background/foreground transparency so underlying apps show through
- [ ] Add GPU/CPU profiling hooks to measure render throughput (per update + scroll)
- [ ] Implement screenshot/export function for QA comparisons

## Phase 5 – Integration Workflows
- [ ] Hook typg search results into font source panel (allow importing matches)
- [ ] Surface fontlift install/uninstall/list commands within UI (context menu or buttons)
- [ ] Provide status notifications/log pane for long-running operations
- [ ] Add telemetry hooks (local only) for debugging performance; keep optional per privacy requirements

## Phase 6 – Testing & Packaging
- [ ] Write integration tests/smoke tests for each major workflow (drag/drop, control adjustments, dual backend rendering)
- [ ] Automate snapshot tests for UI layout + color correctness (per toolkit support)
- [ ] Package app for macOS + Windows initially; document Linux blockers/workarounds
- [ ] Update `README.md` with install instructions, screenshots/gifs, troubleshooting
- [ ] Maintain `WORK.md` log + `CHANGELOG.md` entries as features land

**Performance First:** Keep allocations/render passes minimal. Profile before adding features, and prefer typf-provided caches over custom ones.
