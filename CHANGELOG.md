# Changelog

## v1.0.5 (2025-12-01)
- Added a first-run quick-start card (steps to add fonts, pick install scope, render) and simplified empty preview copy for clearer onboarding.
- Render action now disables itself until there is at least one renderable font, with hints when filters or selection state block rendering.

## v1.0.4 (2025-12-01)
- Fixed Python module import to use `typfpy` instead of `typf` in testypf-core for GUI runtime.
- Added pyo3 `prepare_freethreaded_python()` initialization at GUI startup.
- Updated error messages and tests to reference correct `typfpy` module name.

## v1.0.3 (2025-12-01)
- Added typg-core integration for font discovery via new `DiscoveryManager` and `SearchCriteria` APIs.
- DiscoveryManager auto-detects platform font directories (macOS/Windows/Linux).
- SearchCriteria supports name pattern (regex), OpenType features/scripts/axes, and variable-only filtering.

## v1.0.2 (2025-12-01)
- Fixed `build.sh --verify` to use correct module name `typfpy` instead of `typf`.

## v1.0.1 (2025-12-01)
- GUI now surfaces friendlier error guidance for font installs/uninstalls (permissions, platform features) and typf render failures (PYTHONPATH/build.sh hints); added unit tests for the messaging helpers.
- build.sh now calls the fontlift builder with the supported `--core-only` flag, respects `.venv` or `venv` for typf bindings, and defaults typf feature flags to `render-opixa` (docs/examples updated accordingly).
- Added command/ctrl shortcuts for add fonts, render, export PNGs, and opening the render window, with in-app shortcut hints.
- Added font list filter and a render-selected-only toggle so renders can be scoped to visible fonts or the active selection for large font sets; previews respect the filter while still showing the scoped selection.
- Added font metadata panel with per-font "Details" toggle showing name/family/style, PostScript name, install state, path, and file size.
- Captured per-preview render durations and surface them in preview metadata plus total render time status messages.
- Added helper utilities and unit tests for preview metadata text and human-readable file size formatting.
- Added install scope picker (user/system) wired through FontLift with core tests to ensure the selected scope is honored for install/uninstall actions.
- Added `build.sh --fresh-check` to simulate a fresh macOS setup by verifying font directories and bootstrapping a disposable uv virtualenv before building.
- Added preview layout toggle (single column or side-by-side) in both main and transparent windows for faster font comparison, with layout-aware grouping tests.
- Added foreground/background hex color controls (with optional background toggle) so renders respect transparency and custom colors from the GUI.
- Implemented render preview caching keyed by font set and render settings, skipping redundant renders when nothing changed and reusing previews for the transparent window.
- Added PNG export for render previews with folder picker, filename sanitization, and a guard unit test on the generated PNG signature.
- Added unit tests for color parsing and render cache decisions to guard new controls.
- Added multi-window support with a dedicated always-on-top transparent render window; render previews now open in a separate overlay while controls stay in the main window.
- Enabled iced `multi-window` feature and added a render window settings test to guard transparency/borderless defaults.
- `build.sh --verify` now exercises a real `typf.render_text` call using a known system font (override with `TYPF_VERIFY_FONT`) for quicker fresh-env validation.
- GUI disables install/uninstall when platform fontlift features are absent and surfaces a clear notice; added a guard test to keep the UX safe when platform support is missing.
- Render previews now display Typf RGBA8 output via Iced image handles with metadata.
- Fixed backend mapping to Typf Opixa and hid the debug-only JSON backend from the picker.
- Enabled iced file-drop subscription with hover feedback; added unit tests for render-to-image conversion.
- Backend picker now reflects detected renderer backends, persists last selection to a config file, and filters out non-visual options.
- Drag/drop scanning now reports folder/file counts with sample filenames for quick verification.
- Drag-and-drop now batches multiple files per drop, supports broader font formats (ttf/otf/ttc/otc/woff/woff2/dfont/eot/svg/pfa/pfb/otb), and shows extension breakdowns in status messages.
- `build.sh` now enforces Python 3.12+, adds preflight `--diagnose`, optional `--verify` import check for typf, installs `maturin` automatically, and provides clearer hints when fontlift/typf sources are missing.
- `build.sh` picks up `TYPF_FEATURES` for typf binding builds, and a new `examples/render_once.rs` demonstrates headless rendering with documented setup steps in `examples/README.md` plus cross-links in README/USAGE.
