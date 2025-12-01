# TestYPF Development Work Log

**Current Session:** 2025-12-01 (hotkeys + filtered rendering scope)  
**Focus:** Keyboard shortcuts and scoped rendering for large font sets  
**Status:** Tests passing (typf-dependent GUI tests still ignored)

## Session Overview
- Added command/ctrl hotkeys for add fonts, render, export, and opening the render window with an in-app shortcut hint.
- Added font filter input (name/family/style/path) and a render-selected-only toggle to limit renders to the active font for large collections; previews respect the filter while still showing the selected font when scoped renders are on.
- Surfaced filtered counts in the font list header to clarify visibility when filters hide entries.

## Tests
- `cargo fmt --all`
- `cargo test --workspace` (pass; typf-dependent GUI tests ignored)

## Follow-ups
- Consider honoring the filter when persisting config or adding a quick-clear action for the filter field.
- Evaluate whether to expose a “render first N fonts” guard for extremely large drops as an additional safety valve.

---

**Current Session:** 2025-12-01 (metadata + render timing)
**Focus:** Font metadata surfacing and basic render profiling
**Status:** Tests passing (typf-dependent GUI tests still ignored)

## Session Overview
- Added per-font "Details" toggle and metadata panel showing name, family/style, PostScript name, install state, path, and file size when available.
- Captured per-preview render durations and surfaced them in metadata strings plus overall render status timing.
- Introduced helpers for human-readable file sizes and preview metadata text, with unit tests guarding formatting.

## Tests
- `cargo fmt --all`
- `cargo test --workspace` (pass; typf-dependent tests ignored)

## Follow-ups
- Extend profiling to memory/throughput metrics and High-DPI optimization.

---

**Previous Session:** 2025-12-01  
**Focus:** PNG export of render previews  
**Status:** Tests passing (typf-dependent tests ignored)

## Session Overview
- Added PNG export pipeline by retaining RGBA buffers in render previews and wiring a new “Export PNGs” action with a folder picker.
- Sanitized filenames, avoid overwrites with numeric suffixes, and surface clear status messages for success/failure.
- Added a unit test that writes a preview to disk and asserts the PNG signature to guard regressions.

## Tests
- `cargo test -p testypf-gui --tests` (pass; typf-dependent GUI tests ignored)

## Follow-ups
- Consider optional SVG export and variable font axis sliders.

---

**Previous Session:** 2025-12-01  
**Focus:** Font install scope selection + fresh macOS build check  
**Status:** Tests passing (GUI test warnings unchanged; typf-dependent tests still ignored)

## Session Overview
- Added user/system install scope plumbing through `FontListManager` and GUI picker; installation status text now reflects chosen scope.
- Added core tests with a mock FontLift manager to guard scope selection and state updates.
- Added `build.sh --fresh-check` to verify font directory permissions and uv venv bootstrap for fresh macOS setups.

## Tests
- `cargo test --workspace` (pass; 2 typf-dependent GUI tests ignored)
- `./build.sh --fresh-check`

## Follow-ups
- System-scope installs still require elevated privileges at runtime; consider prompting users when scope is System and install fails with permission errors.
- Persist install scope in config alongside backend selection if we want stickier UX.

---

**Earlier Session:** 2025-12-01  
**Focus:** Render caching + color controls  
**Status:** Tests passing (GUI-only warnings about unused helper views/lifetime hints)

## Session Overview
- Added RGBA hex foreground input plus optional background toggle/input so Typf renders respect custom colors and transparency from the GUI.
- Introduced render preview caching keyed by render settings and loaded font paths; reused cached previews when nothing changed and surfaced a status message.
- Added unit tests covering color parsing and cache hit decisions to guard the new controls.

## Tests
- `cargo test -p testypf-gui --tests` (pass; two typf-dependent tests still ignored; warnings about unused `ui` helpers and lifetime suggestion)

## Follow-ups
- Consider persisting color settings alongside backend in config file.
- Profile cache behavior with large font sets and incremental renders once Typf bindings run in CI.

---

**Earlier Session:** 2025-12-01  
**Focus:** Multi-window render preview with transparent overlay  
**Status:** Tests passing (typf-dependent tests ignored)

## Session Overview
- Migrated GUI to `iced::multi_window::Application` and added a dedicated transparent render window spawned via `window::spawn`, always-on-top and borderless for overlay previews.
- Added render-window button and auto-open after renders; refactored preview rendering to pipe results into the secondary window while keeping main window controls.
- Introduced helper to centralize render window settings plus a unit test asserting transparency/decorations/size; enabled iced `multi-window` feature workspace-wide.

## Tests
- `cargo test -p testypf-gui --tests` (pass; typf-dependent tests still ignored)

## Follow-ups
- Consider syncing preview scroll position between windows and making the render window optionally hidden until first render.
- Evaluate rendering performance impact across multiple windows once typf bindings available in CI.

---

**Earlier Session:** 2025-12-01 (cont.)  
**Focus:** FontLift status surfacing in GUI install flow  
**Status:** Tests passing (one env-dependent test ignored)

## Session Overview
- Added install-state cache helpers and wired GUI buttons to refresh status via FontLift `is_font_installed`, disabling install/uninstall when not applicable.
- Surfaced install state in the font list with clear installed/not-installed badges to avoid no-op clicks.

## Tests
- `cargo test -p testypf-gui --tests`

## Follow-ups
- Validate FontLift install/uninstall on macOS and Windows feature builds to ensure status checks agree with system directories.
- Harden permission/conflict error messages once real platform runs are available.

---

**Earlier Session:** 2025-12-01 (cont.)  
**Focus:** Multi-format drag/drop batching; build.sh diagnostics & verification hooks  
**Status:** Tests passing (one env-dependent test ignored)

## Session Overview (2025-12-01 later)
- Batched drag-and-drop processing to capture multiple files/formats per drop and surface extension mix in status messages; added support for dfont/eot/svg/pfa/pfb/otb.
- Added extension statistics helper and refreshed drop-area copy to reflect supported formats; updated file dialog path to reuse the same validation.
- Hardened build.sh with strict shell flags, Python 3.12 check, dependency directory hints, maturin install, new `--diagnose` and `--verify` modes, and typf import verification.

## Tests
- `cargo test -p testypf-gui --tests` (pass; `tests::test_app_creation` still ignored pending typf on PYTHONPATH)

## Follow-ups
- Consider reducing drop aggregation delay or making it configurable once we profile UX.
- Fresh-mac build validation still outstanding; typf build robustness across feature combos remains to verify.
- Add examples/ directory with runnable samples.

---

**Earlier Session:** 2025-12-01 (late)  
**Focus:** Build script reliability + runnable example  
**Status:** Tests passing

## Session Overview
- Fixed `build.sh` prerequisite check regression (missing `command_exists`) and made typf feature set overridable via `TYPF_FEATURES`, keeping uv/maturin installs inside the venv.
- Added `examples/render_once.rs` plus `examples/README.md` showing how to run a one-shot typf render; cross-referenced in README/USAGE with env hint for feature overrides.

## Tests
- `cargo fmt --all`
- `cargo test -p testypf-core --tests`

## Follow-ups
- Still need real fresh-mac validation of `build.sh` and typf binding build matrix.
- Document or script discovery of typf site-packages path to reduce PYTHONPATH friction.
