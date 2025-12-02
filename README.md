# TestYPF

A GUI application for testing and demonstrating Typf (font rendering engine) capabilities, integrated with FontLift for font management and typg for font discovery.

## Overview

TestYPF is a minimal-yet-fast cross-platform GUI application that showcases:
- **Typf Rendering**: Multiple backend text rendering capabilities
- **Font Discovery**: Integration with typg for font search
- **Font Management**: FontLift integration for install/uninstall operations
- **Real-time Preview**: Live font rendering with adjustable parameters

## Architecture

The project is organized into two main crates:

- `testypf-core`: Core engine and business logic
- `testypf-gui`: GUI frontend using the Iced framework

### Core Components

- **Font Manager**: Handles font loading, installation, and management
- **Text Renderer**: Interfaces with Typf for multiple backend rendering
- **GUI Interface**: Provides user controls and preview windows
- **Event System**: Reacts to user interactions and updates rendering

## Features

### Font Management
- ✅ Drag-and-drop font file loading
- ✅ Multiple font selection support
- ✅ Font installation/uninstall via FontLift
- ✅ Font listing with metadata
- ✅ Session-based font management

### Text Rendering
- ✅ Multiple rendering backends (Orge, Json, CoreGraphics, Skia, Zeno)
- ✅ Real-time text preview
- ✅ Adjustable font size and sample text
- ✅ Color picker for foreground/background
- ✅ Support for transparency
- ✅ Variable font axis controls

### User Interface
- ✅ Clean, minimal design
- ✅ Responsive layout
- ✅ Keyboard accessible controls
- ✅ Status notifications
- ✅ Error handling feedback

## Quick Start

### Running the Application

```bash
# Build and run
cargo run

# Build release
cargo build --release

# Run from release binary
./target/release/testypf

# Run the CLI example (requires typf Python bindings)
cargo run --example render_once -- /path/to/font.ttf "Sample text"
```

### Basic Usage

1. **Add Fonts**: Click "Add Fonts..." or drag font files onto the application
2. **Configure Text**: Enter sample text and adjust font size
3. **Select Backend**: Choose rendering backend from available options
4. **Preview**: Click "Render Previews" to see rendered text
5. **Manage Fonts**: Use Install/Uninstall buttons for font management

## GUI Layout

```
┌─────────────────────────────────────────────────────┐
│ Testypf - Typf GUI Tester                           │
├─────────────────────────────────────────────────────┤
│ Status: Ready                                       │
├─────────────────────────────────────────────────────┤
│ Font List                                           │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Font Family (Style)                    [ Remove ]│ │
│ │                                 [ Install ] [ Uninstall ]│ │
│ └─────────────────────────────────────────────────┘ │
│ [ Add Fonts... ]                                    │
├─────────────────────────────────────────────────────┤
│ Render Controls                                     │
│ Sample Text: [ The quick brown fox...            ] │
│ Font Size:   [ 16                                ] │
│ Backend: Orge                                       │
│ [ Render Previews ]                                 │
└─────────────────────────────────────────────────────┘
```

## As a Rust Library

You can also use testypf-core as a library in your own applications:

```rust
use testypf_core::{TestypfEngine, RenderSettings, RendererBackend};

// Create engine
let mut engine = TestypfEngine::new()?;

// Configure rendering
let settings = RenderSettings {
    sample_text: "Hello, World!".to_string(),
    font_size: 24.0,
    backend: RendererBackend::Orge,
    ..Default::default()
};

// Add font
let font_path = std::path::PathBuf::from("my-font.ttf");
let font_info = engine.font_manager().add_font(&font_path)?;

// Render text
let render_result = engine.text_renderer()
    .render_text(&font_path, &settings)?;

println!("Rendered {}x{} image", render_result.width, render_result.height);
```

## Platform Support

### Currently Supported
- ✅ macOS (with CoreGraphics rendering)
- ✅ Windows (with Skia/Orge rendering)

### Backend Availability
| Backend | macOS | Windows | Notes |
|---------|-------|---------|-------|
| Orge    | ✅    | ✅      | Vector rendering |
| Json    | ✅    | ✅      | Debug output |
| CoreGraphics | ✅ | ❌      | macOS native |
| Skia    | ✅    | ✅      | GPU rendering |
| Zeno    | ✅    | ✅      | Experimental |

## Building

### Prerequisites
- Rust 1.75+
- Platform GUI dependencies:
  - macOS: System dependencies included with Iced
  - Windows: Visual Studio Build Tools

### Build Commands

```bash
# Build GUI application
cargo build -p testypf-gui

# Build with all features
cargo build --all-features

# Build release
cargo build --release

# Run tests
cargo test --workspace
```

### GPU/Renderer Dependencies

Some backends require additional dependencies:

```bash
# For Skia backend
cargo build --features "render-skia"

# For Zeno backend  
cargo build --features "render-zeno"

# For all backends
cargo build --features "render-skia,render-zeno"

# Override typf feature flags used by build.sh
TYPF_FEATURES="shaping-hb,render-opixa,render-skia" ./build.sh --verify
```

## Examples

- `examples/render_once.rs`: Minimal CLI that loads a font, renders sample text through typf, and prints render metadata. See `examples/README.md` for setup (typf Python bindings required).

## Configuration

### Environment Variables
- `RUST_LOG`: Set logging level (e.g., `debug`, `info`, `warn`)
- `Typf_BACKEND_DIR`: Directory for Typf backend libraries

### Default Settings
- Default sample text: "The quick brown fox jumps over the lazy dog"
- Default font size: 16.0pt
- Default backend: Orge
- Default colors: Black text on transparent background

## Performance

### Optimizations
- Efficient font loading and caching
- GPU-accelerated rendering where available
- Lazy loading of render backends
- Minimal UI redraws

### Benchmarks
- Font loading: <100ms for typical fonts
- Text rendering: <10ms for standard text blocks
- UI responsiveness: 60 FPS target

## Error Handling

The application provides clear feedback for common errors:

- **Invalid Font Files**: Shows file format errors
- **Installation Failures**: Displays permission issues
- **Rendering Errors**: Shows backend problems
- **Network Issues**: Handles download/update failures

## Integration Points

### With Typf
```rust
// Rendering with specific backend
engine.text_renderer().set_backend(RendererBackend::Skia)?;

// Access Typf directly
let typf_instance = engine.typf_instance();
```

### With FontLift
```rust
// Install font system-wide
engine.font_manager().install_font(&font_info, FontScope::System)?;

// List installed fonts
let installed = engine.font_manager().get_installed_fonts()?;
```

### With Typg
```rust
// Search for fonts
let search_results = engine.font_discovery()
    .search("Arial")?
    .into_font_list();
```

## Development

### Project Structure
```
testypf/
├── Cargo.toml              # Workspace configuration
├── README.md
├── crates/
│   ├── testypf-core/       # Core engine
│   │   ├── src/
│   │   │   ├── lib.rs      # Main API
│   │   │   ├── font/       # Font management
│   │   │   └── render/     # Rendering integration
│   └── testypf-gui/        # GUI application
│       ├── src/
│       │   ├── main.rs     # Application entry
│       │   └── ui/         # UI components
└── tests/
    ├── integration/        # Integration tests
    └── benchmarks/         # Performance tests
```

### Adding New Features
1. Add functionality to `testypf-core`
2. Expose through engine API
3. Add GUI controls in `testypf-gui`
4. Write tests for new functionality
5. Update documentation

### Code Style
- Follow Rust idioms and Iced patterns
- Use `cargo fmt` and `cargo clippy`
- Document all public APIs
- Handle errors gracefully

## Testing

```bash
# Run unit tests
cargo test -p testypf-core

# Run integration tests
cargo test -p testypf-gui

# Run benchmarks
cargo test --release --features "benchmarks"

# Test specific functionality
cargo test font::tests
cargo test render::tests
```

## Roadmap

### Near Term
- [ ] Multiple font comparison view
- [ ] Export rendered images
- [ ] Font metadata display panel
- [ ] Keyboard shortcuts
- [ ] Dark/light theme support

### Medium Term  
- [ ] Variable font axes controls
- [ ] OpenType feature dropdown
- [ ] Font conflict detection
- [ ] Batch rendering operations
- [ ] Performance profiling view

### Long Term
- [ ] Plugin system for custom backends
- [ ] Scriptable rendering workflows
- [ ] Cloud font integration
- [ ] Collaborative font testing
- [ ] Mobile platform support

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality  
4. Ensure all tests pass
5. Update documentation
6. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

TestYPF is licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.

---

Made by FontLab https://www.fontlab.com/
