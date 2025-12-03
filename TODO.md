# TestYPF TODO

- [x] Integrate @./linked/typg for font finding (DiscoveryManager + SearchCriteria API in testypf-core)

**Scope:** Build `testypf`, a minimal-yet-fast cross-platform GUI app showcasing typf rendering, typg discovery, and fontlift install flows.

**Current Status:** MVP Critical Path - Core functionality exists but needs essential visual and integration improvements for a working product.

## MVP CRITICAL PATH - 5 Essential Improvements for Working Product

### [x] 1. Real Rendering Display in GUI (COMPLETE for MVP)
**Current State:** Typf RGBA8 results are converted to Iced image handles; preview area now renders real images with metadata.
- [x] Convert Typf RGBA data to Iced image handles for display
- [x] Replace placeholder text in [`preview_area`](crates/testypf-gui/src/main.rs:414) with actual image widgets
- [x] Add scrollable preview area that shows rendered font images at proper scale
- [x] Display render metadata (backend used, render time, dimensions) under each image
- [x] Handle rendering errors with user-friendly messages in the status bar
**Impact:** Users can now see real render outputs; further polish can focus on layout and error surfacing in-view.

### [!] 2. Real Font Management Integration (CRITICAL) 
**Current State:** FontLift calls are implemented in `FontListManager`, but platform features (`platform-mac` / `platform-win`) must be enabled and the flow is unvalidated in-app.
- [x] Replace dummy font management with actual fontlift-core API calls
- [x] Implement real font installation to user/system font directories using platform managers (feature-flagged builds)
- [x] Add proper error handling for permission issues and font conflicts
- [x] Update GUI to show real installation status by checking system font directories
- [x] Test font installation works on macOS (~/Library/Fonts, /Library/Fonts)
**Impact:** Font management is a key differentiator - need verified platform builds and UI feedback.

### [x] 3. Interactive Backend Selection Widget (HIGH)
 **Current State:** Backend picker now maps "Orge" → Typf Opixa and hides JSON in UI, with capability detection and persistence in place.
- [x] Wire backend picker to real typf backend names (Opixa) and exclude non-visual JSON backend from UI
- [x] Enhance [`backend_selector`](crates/testypf-gui/src/main.rs:685) to dynamically show available backends
- [x] Detect platform capabilities (CoreGraphics on macOS, Skia if feature enabled)
- [x] Add backend-specific settings and capabilities display (e.g., "Vector rendering" for Opixa)
- [x] Remember backend selection between sessions using simple config file
- [x] Add backend testing functionality to verify backend works before rendering
**Impact:** Users need easy access to different rendering backends to test typf capabilities

 ### [x] 4. Enhanced Drag & Drop UX (HIGH)
 **Current State:** File-drop subscription is active with hover feedback and recursive scanning; still missing richer styling tweaks and multi-format drag tests.
 - [x] Improve [`DragActiveStyle`](crates/testypf-gui/src/main.rs:1195) with better visual feedback (border animation, color transitions)
 - [x] Add progress indicators for folder scanning operations with file count display
 - [x] Implement file validation before processing with proper error messages
 - [x] Add support for drag & drop of multiple font formats simultaneously
 - [x] Show preview of files being processed in status area
**Impact:** Power users testing large font collections need good UX feedback for batch operations

### [!] 5. Build System Validation & Project Structure Completion (REQUIRED)
**Current State:** [`build.sh`](build.sh) is comprehensive but needs testing and validation; project structure is mostly complete
- [x] Comprehensive build script with dependency handling exists
- [x] Test build script on fresh macOS environment to ensure it works for new developers
 - [x] Add error handling for missing fontlift/typf dependencies with helpful setup instructions
 - [x] Ensure typf Python bindings build correctly in all scenarios
 - [x] Add validation steps to verify integration works after build
 - [x] Create missing examples/ directory with usage examples
 - [x] Ensure all project documentation is consistent and cross-referenced
**Impact:** Without working build system, no one can use or contribute to the application

---

## POST-MVP ENHANCEMENTS (Future Development)

### Phase 2 - UI/UX Polish
- [x] **Multi-Window Architecture**: Separate floating panels for font list, controls, render window
- [x] **Transparent Render Window**: Borderless window with transparency support for overlay testing
- [x] **Color Picker Controls**: Foreground/background color selection with transparency
- [x] **Variable Font Axes**: Dynamic sliders for variation axes with live Typf wiring (fvar parsed in core; GUI now seeds defaults, clamps ranges, and sends variation coords to renders)
- [x] **Export Functionality**: Save rendered previews as PNGs via in-app export action

### Phase 3 - Advanced Integration
- [x] **Side-by-Side Comparison**: Compare multiple backends simultaneously in split view
- [x] **Font Metadata Panel**: Display detailed font information (name, family, style, PostScript, path, install state)
- [~] **Performance Profiling**: Show render times (per-preview timing added); memory usage still pending
- [ ] **Typg Discovery Integration**: Import fonts from typg search results
- [ ] **High-DPI Optimization**: Proper scaling for retina displays

### Phase 4 - Production Ready
- [ ] **Cross-Platform Testing**: Ensure consistency on Windows/Linux
- [ ] **Comprehensive Testing**: Unit tests, integration tests, UI tests
- [ ] **Documentation完善**: Screenshots, examples, troubleshooting guide
- [x] **Keyboard Shortcuts**: Power user keyboard navigation
- [ ] **Error Recovery**: Better error messages and recovery suggestions
 - [x] **Error Recovery**: Better error messages and recovery suggestions (permission hints, platform-feature guidance, typf/PYTHONPATH remediation)

---

## COMPLETED FOUNDATIONS ✅

### Phase 0 - Analysis & Planning (COMPLETE)
- [x] **API Audit**: Analyzed typf, fontlift, and typg integration points and capabilities
- [x] **GUI Toolkit Evaluation**: Selected Iced framework with comprehensive technical analysis
- [x] **Architecture Definition**: Multi-window design with clean separation of concerns
- [x] **Project Structure**: Established workspace with crates/testypf-core and crates/testypf-gui

### Phase 1 - Core Implementation (COMPLETE)
- [x] **Basic GUI Structure**: Working Iced application with proper window management
- [x] **Widget System**: Buttons, text inputs, containers, scrollable areas, layout management
- [x] **Font Loading**: Real font metadata extraction using read-fonts crate
- [x] **Typf Integration**: Full Python bindings integration with working typf engine
- [x] **Event System**: Complete message passing and state management framework
- [x] **Basic Drag & Drop**: File and folder drop with recursive scanning
- [x] **Build System**: Comprehensive macOS build script with dependency management

---

## TECHNICAL IMPLEMENTATION STATUS

### ✅ Working Components
1. **Core Architecture**: Clean separation between testypf-core and testypf-gui crates
2. **Font Loading**: Real font parsing with metadata extraction using read-fonts
3. **Typf Integration**: Full Python bindings working - generates actual render data (Opixa default)
4. **Visual Rendering**: GUI renders Typf RGBA8 buffers as images with metadata
5. **Basic GUI**: All widgets, layout, and event handling functional
6. **Build System**: Comprehensive script handles all dependencies and build types

### ❌ Critical MVP Gaps
1. **Font Management Validation**: FontLift flow needs platform-feature builds and UI feedback
2. **Backend Controls**: Missing dynamic detection and persistence; JSON backend still debug-only
3. **Drag & Drop UX**: No progress indication for folder scans; styling can improve
4. **Build Validation**: Script exists but needs testing on fresh environments

### Integration Points Status
- **Typf**: ✅ Full integration working - generates real render results
- **FontLift**: 🚧 Real APIs wired; requires platform feature enablement and validation
- **Typg**: ✅ Core integration complete - DiscoveryManager + SearchCriteria API ready; GUI search panel pending

---

## DEVELOPMENT APPROACH

### MVP Success Criteria
1. **Visual Rendering**: Users can see actual rendered font images in GUI
2. **Real Font Management**: Font install/uninstall uses actual fontlift APIs
3. **Interactive Backend Selection**: Users can switch backends via UI controls
4. **Enhanced UX**: Drag & drop has visual feedback and progress indication
5. **Validated Build**: Fresh environment can build and run without errors
6. **Complete Structure**: All project files present and documentation consistent

### Implementation Principles
Following ruthless minimalism:
1. **Test First**: Write failing tests for each integration point
2. **Minimal Implementation**: Only what's needed for MVP functionality
3. **Real Integration**: Replace all dummy implementations with actual API calls
4. **User Experience**: Visual feedback and error handling are essential

---

## KNOWN DEPENDENCIES & BUILD REQUIREMENTS

### External Dependencies
- **Rust 1.75+**: Core language and toolchain
- **Python 3.12+**: For typf Python bindings
- **uv**: Python package management (installed by build script)
- **maturin**: Python-Rust build tool (installed by build script)

### Workspace Dependencies
- **fontlift-core**: Font management APIs (needs real integration)
- **fontlift-platform-mac**: macOS-specific font operations
- **typf-py**: Python bindings for font rendering (working)

### Platform Requirements
- **macOS**: Primary target - CoreGraphics backend available
- **Windows**: Secondary target - Skia/Orge backends planned
- **Linux**: Future target - Orge backend available

---

**Priority Focus:** Complete the 5 MVP CRITICAL items to deliver a working application that users can actually use to test font rendering.
**Current Sprint:** Start with Priority 1 (Real Rendering Display) as it provides immediate user value and validates the entire Typf integration pipeline.
**Timeline Target:** Complete MVP critical path in focused development sessions.
*Last Updated: 2025-11-21*
