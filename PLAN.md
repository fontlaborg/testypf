# Testypf Implementation Plan

## Project Overview

Testypf is a minimal-yet-fast cross-platform GUI application that showcases typf rendering capabilities, typg font discovery, and fontlift font management flows. Built with Rust and the Iced GUI framework, it provides an intuitive interface for font testing, comparison, and management.

## Architecture

### Crate Structure

```
testypf/
├── crates/
│   ├── testypf-core/           # Core abstractions and business logic
│   │   ├── src/
│   │   │   ├── lib.rs          # Main exports and engine
│   │   │   ├── font.rs         # Font management integration
│   │   │   ├── render.rs       # Typf rendering integration
│   │   │   ├── discovery.rs    # Typg discovery integration (planned)
│   │   │   ├── error.rs        # Error types and handling
│   │   │   └── types.rs        # Shared data structures
│   │   └── Cargo.toml
│   └── testypf-gui/            # Iced GUI application
│       ├── src/
│       │   ├── main.rs         # Application entry point
│       │   ├── app.rs          # Main application state
│       │   ├── ui/
│       │   │   ├── mod.rs
│       │   │   ├── font_list.rs    # Font list panel
│       │   │   ├── controls.rs     # Control panel
│       │   │   ├── render_view.rs  # Render window
│       │   │   └── components.rs   # Shared UI components
│       │   └── windows/
│       │       ├── mod.rs
│       │       ├── font_list_window.rs
│       │       ├── control_window.rs
│       │       └── render_window.rs
│       └── Cargo.toml
├── examples/                   # Usage examples
├── tests/                      # Integration tests
├── Cargo.toml                  # Workspace configuration
├── build.sh                    # Build script
└── README.md
```

### GUI Architecture

#### Multi-Window Design
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Font List      │    │  Control Panel  │    │   Render Window │
│  (Floating)     │    │  (Floating)     │    │  (Transparent)  │
│                 │    │                 │    │                 │
│ • Drag & Drop   │◄──►│ • Settings Sync │◄──►│ • Live Preview  │
│ • Font Management │   │ • Backend Select│   │ • Side-by-Side  │
│ • Typg Search   │    │ • Color/Size    │    │ • High-DPI      │
│ • Install Actions│   │ • Feature Controls│   │ • Export/Capture│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

#### Data Flow Architecture
```rust
// Core application state
pub struct TestypfEngine {
    font_manager: Box<dyn FontManager>,      // FontLift integration
    text_renderer: Box<dyn TextRenderer>,    // Typf integration
    discovery_manager: Option<DiscoveryManager>, // Typg integration (future)
}

// GUI state management
pub struct AppState {
    engine: TestypfEngine,
    fonts: Vec<FontInfo>,
    selected_fonts: HashSet<usize>,
    render_settings: RenderSettings,
    render_cache: HashMap<RenderKey, RenderResult>,
    ui_state: UiState,
}

// Communication between windows
pub enum Message {
    // Font management
    FontAdded(PathBuf),
    FontRemoved(usize),
    FontSelected(usize),
    FontInstalled(usize),
    
    // Rendering
    SettingsChanged(RenderSettings),
    RenderRequested,
    RenderCompleted(Vec<(usize, RenderResult)>),
    
    // UI state
    WindowFocused(WindowId),
    PanelMoved(WindowId, Position),
}
```

## Integration Points

### Typf Rendering Integration
**Purpose**: Display font renderings using different backends

**API Integration**:
```rust
// typf → testypf integration
pub struct TypfRenderer {
    engine: typf::Engine,
    backend: RendererBackend,
}

impl TextRenderer for TypfRenderer {
    fn render_text(&self, font_path: &Path, settings: &RenderSettings) -> TestypfResult<RenderResult> {
        // Convert settings and call typf pipeline
        let typf_settings = self.convert_settings(settings);
        let typf_result = self.engine.render(font_path, typf_settings)?;
        Ok(self.convert_result(typf_result))
    }
}
```

**Rendering Pipeline**:
1. **Settings Conversion**: Convert GUI settings to typf settings
2. **Backend Selection**: Choose appropriate typf backend
3. **Render Execution**: Call typf rendering pipeline
4. **Result Processing**: Convert typf output to GUI format
5. **Caching**: Cache results for performance

**Available Backends**:
- Orge (vector rendering)
- Skia (high-performance bitmap)
- Zeno (alternative vector)
- CoreGraphics (macOS native)
- JSON (debug/metadata)

### FontLift Management Integration
**Purpose**: Install, uninstall, and manage system fonts

**API Integration**:
```rust
// fontlift → testypf integration
pub struct FontListManager {
    fontlift_manager: Arc<dyn fontlift_core::FontManager>,
    gui_fonts: Vec<FontInfo>,
}

impl FontManager for FontListManager {
    fn install_font(&self, font: &FontInfo) -> TestypfResult<()> {
        // Use fontlift to install system font
        self.fontlift_manager
            .install_font(&font.path, FontScope::User)?;
        Ok(())
    }
}
```

**Font Management Flow**:
1. **Font Discovery**: Import fonts via drag/drop or typg search
2. **Metadata Extraction**: Use fontlift to extract font information
3. **System Integration**: Install/uninstall via fontlift APIs
4. **Status Updates**: Reflect installation status in GUI

### Typg Discovery Integration (Planned)
**Purpose**: Search and discover fonts on the local system

**API Integration**:
```rust
// typg → testypf integration (future)
pub struct DiscoveryManager {
    typg_engine: typg::Engine,
}

impl DiscoveryManager {
    pub fn search_fonts(&self, criteria: &SearchCriteria) -> TestypfResult<Vec<FontInfo>> {
        let results = self.typg_engine.search(criteria)?;
        Ok(results.into_iter().map(FontInfo::from).collect())
    }
}
```

## GUI Implementation Strategy

### Iced Framework Implementation

#### Multi-Window Architecture
```rust
// Each window as separate Iced application
pub struct FontListWindow {
    fonts: Vec<FontInfo>,
    selected: HashSet<usize>,
    filter: String,
}

pub struct ControlWindow {
    settings: RenderSettings,
    backend_options: Vec<RendererBackend>,
}

pub struct RenderWindow {
    render_results: Vec<RenderResult>,
    layout_mode: LayoutMode, // Single, Side-by-side, Grid
}
```

#### Drag & Drop Implementation
```rust
use iced::widget::file_drop;

impl FontListWindow {
    fn view(&self) -> Element<Message> {
        container(
            column![
                // Drag & drop area
                file_drop("Drag fonts here or click to browse...")
                    .on_accept(Message::FontsDropped),
                
                // Font list with actions
                scrollable(
                    column(self.fonts.iter().enumerate().map(|(i, font)| {
                        row![
                            checkbox("", self.selected.contains(&i))
                                .on_toggle(Message::FontSelected(i)),
                            text(&font.full_name),
                            button("Install").on_press(Message::FontInstall(i)),
                            button("Remove").on_press(Message::FontRemove(i)),
                        ]
                        .spacing(10)
                        .into()
                    }))
                ),
                
                // Typg search integration (future)
                text_input("Search fonts...", &self.filter)
                    .on_input(Message::FilterChanged),
            ]
            .spacing(10)
        )
        .into()
    }
}
```

#### Transparent Render Window
```rust
impl Application for RenderWindow {
    fn settings() -> Settings {
        Settings {
            window: iced::window::Settings {
                transparent: true,
                decorations: false,
                position: Position::Specific(500, 100),
                size: (800, 600),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    fn view(&self) -> Element<Message> {
        // Transparent background with font previews
        container(
            // Render results layout
            match self.layout_mode {
                LayoutMode::Single => self.single_view(),
                LayoutMode::SideBySide => self.comparison_view(),
                LayoutMode::Grid => self.grid_view(),
            }
        )
        .style(iced::theme::Container::Transparent)
        .into()
    }
}
```

### Performance Optimization

#### Rendering Pipeline Optimization
```rust
pub struct RenderCache {
    cache: HashMap<RenderKey, RenderResult>,
    dirty_fonts: HashSet<usize>,
    max_cache_size: usize,
}

impl RenderCache {
    pub fn render_dirty(&mut self, engine: &mut TestypfEngine, fonts: &[FontInfo], settings: &RenderSettings) -> Vec<(usize, RenderResult)> {
        let mut results = Vec::new();
        
        for &font_index in &self.dirty_fonts.clone() {
            if let Some(font) = fonts.get(font_index) {
                let key = RenderKey::new(font_index, settings);
                if let Ok(result) = engine.render_preview(&font.path, settings) {
                    self.cache.insert(key, result.clone());
                    results.push((font_index, result));
                }
            }
        }
        
        self.dirty_fonts.clear();
        self.evict_if_needed();
        results
    }
}
```

#### Lazy Loading Strategy
```rust
impl FontListWindow {
    pub fn scroll_to_font(&mut self, font_index: usize) {
        // Only render fonts that are visible
        let visible_range = self.calculate_visible_range();
        for i in visible_range {
            if !self.render_cache.contains_key(&i) {
                self.dirty_fonts.insert(i);
            }
        }
        
        // Request render for visible fonts
        Message::RenderVisibleFonts(visible_range)
    }
}
```

## Implementation Phases

### Phase 0 – Foundations ✅ COMPLETE
**Status**: Finished
**Duration**: Analysis and planning

**Tasks Completed**:
- [x] Audited typf, fontlift, and typg APIs
- [x] Evaluated GUI toolkits (Iced recommended)
- [x] Created comprehensive API audit document
- [x] Documented GUI toolkit evaluation with technical analysis
- [x] Defined architecture and integration strategy

### Phase 1 – Core Implementation (Current)
**Status**: In Progress
**Duration**: Basic functionality

**Core Tasks**:
- [x] Basic Iced application structure
- [x] Widget system and state management
- [x] File dialog integration
- [x] Implement drag & drop functionality
- [x] Add multi-window support for floating panels
- [x] Create transparent render window
- [x] Integrate real typf rendering pipeline
- [x] Replace dummy fontlift integration

**Notes (2025-12-01)**:
- GUI now renders Typf RGBA8 outputs into Iced image widgets; JSON renderer remains debug-only and is not shown in the backend picker.
- FontLift calls are live behind platform feature flags (`platform-mac` / `platform-win`); verify on target OS before release.
- Backend picker now shows capability summaries, offers a self-test action, and drag/drop rejects unsupported or missing files with explicit status messages.
- Font list shows installation state from FontLift checks and disables install/uninstall actions accordingly to avoid confusing no-op clicks.
- Install scope toggle (user vs system) wires through FontLift; system scope still requires elevated permissions on macOS.

**Milestone**: Basic font list, controls, and render window working

### Phase 2 – Rendering Integration
**Status**: Planned
**Duration**: Typf integration

**Core Tasks**:
- [ ] Complete typf rendering pipeline integration
- [x] Implement backend selection and switching
- [x] Add side-by-side rendering comparison
- [x] Implement render result caching
- [x] Add color and transparency controls
- [x] Performance optimization for large font sets

**Milestone**: Full typf backend support with performance optimization

### Phase 3 – Font Management Integration
**Status**: Planned
**Duration**: FontLift integration

**Core Tasks**:
- [x] Complete fontlift integration (user/system scope selectable)
- [x] Add font installation/uninstallation UI
- [x] Implement font status display
- [x] Add font validation and error handling
- [ ] Integrate with font discovery workflows

**Milestone**: Complete font management functionality

### Phase 4 – Advanced Features
**Status**: Planned
**Duration**: Enhancement features

**Core Tasks**:
- [ ] Typg discovery integration
- [ ] Advanced color picker with transparency
- [x] Font export and screenshot functionality (PNG export of render previews)
- [ ] Keyboard accessibility improvements
- [~] Performance profiling and optimization (per-preview render timings surfaced; memory/throughput profiling pending)

**Milestone**: Full feature completeness with polish

### Phase 5 – Testing & Polish
**Status**: Planned
**Duration**: Quality assurance

**Core Tasks**:
- [ ] Comprehensive testing (unit, integration, UI)
- [ ] Cross-platform consistency verification
- [ ] Performance benchmarking
- [ ] Documentation completion
- [ ] User experience refinement

**Milestone**: Production-ready application

## Success Metrics

### Functional Metrics
- **Rendering Fidelity**: Exact typf output reproduction
- **Feature Coverage**: All specified features implemented
- **Integration Success**: Seamless typf, fontlift, typg integration
- **Cross-Platform**: Consistent behavior on macOS and Windows

### Performance Metrics
- **Font Loading**: <100ms for typical font files
- **Rendering Latency**: <500ms for preview rendering
- **UI Responsiveness**: <16ms frame time (60 FPS)
- **Memory Usage**: <500MB for typical workflows

### Usability Metrics
- **Learnability**: <5 minutes for basic workflows
- **Efficiency**: <10 seconds for complete testing cycle
- **Error Recovery**: Clear error messages and recovery paths
- **Accessibility**: Keyboard navigation and screen reader support

## Technical Decisions

### GUI Framework: Iced ✅
- **Rust Native**: Perfect integration with typf and fontlift
- **Performance**: GPU-accelerated rendering for smooth UI
- **Feature Support**: All required features achievable
- **Development Velocity**: Fast development with good ecosystem

### Architecture: Multi-Window Design
- **Flexibility**: Independent panels for different workflows
- **Productivity**: Users can arrange workspace as needed
- **Focus**: Separate concerns for different UI areas
- **Scalability**: Easy to add new panels/features

### State Management: Centralized Engine
- **Integration**: Single point for typf/fontlift integration
- **Consistency**: Shared state across windows
- **Performance**: Efficient caching and resource management
- **Testing**: Clear separation of concerns for testing

### Performance Strategy: Lazy Loading + Caching
- **Responsiveness**: Only render visible/needed content
- **Memory**: Intelligent cache management
- **Speed**: Reuse render results when possible
- **Scalability**: Handle large font collections efficiently

---

**Status**: Phase 0 Complete, Phase 1 In Progress
**Current Focus**: Multi-window support and backend/FontLift UX polish
**Next Milestone**: Complete core GUI functionality with real typf integration

*Last Updated: 2025-12-01*
