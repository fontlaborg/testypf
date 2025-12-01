# TestYPF Usage Guide

This guide provides comprehensive usage examples for TestYPF, the GUI application for testing Typf font rendering capabilities.

## Quick Start

### Running the Application

```bash
# Build and run
cargo run

# Build release version
cargo build --release

# Run from release binary
./target/release/testypf
```

### Basic Workflow

1. **Add Fonts**: Click "Add Fonts..." or drag font files onto the application
2. **Configure Text**: Enter sample text and adjust font size
3. **Select Backend**: Choose rendering backend from available options
4. **Preview**: Click "Render Previews" to see rendered text
5. **Manage Fonts**: Use Install/Uninstall buttons for font management

## GUI Interface Overview

### Main Window Layout

```
┌─────────────────────────────────────────────────────┐
│ Testypf - Typf GUI Tester                           │
├─────────────────────────────────────────────────────┤
│ Status: Ready                                       │
├─────────────────────────────────────────────────────┤
│ Font List                                           │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Font Family (Style)                    [ Remove ]│ │
│ │                                   [ Install ] [ Uninstall ]│ │
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

### Font List Panel

- **Font Display**: Shows family name and style for each loaded font
- **Remove Button**: Removes font from current session (doesn't uninstall)
- **Install Button**: Installs font system-wide using FontLift
- **Uninstall Button**: Uninstalls font from system using FontLift
- **Add Fonts Button**: Opens file dialog to select font files

### Render Controls Panel

- **Sample Text**: Text to render with all loaded fonts
- **Font Size**: Point size for rendered text (16.0pt default)
- **Backend Selector**: Available rendering backends
- **Render Previews**: Generates rendered text for all fonts

## Library Usage

TestYPF can also be used as a library in your own applications:

> Looking for a runnable sample? See `examples/render_once.rs` for a minimal CLI that renders a single font via typf.

### Basic Usage

```rust
use testypf_core::{TestypfEngine, RenderSettings, RendererBackend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    
    Ok(())
}
```

### Font Management

```rust
use testypf_core::{FontInfo, FontManager};

let font_manager = engine.font_manager();

// Add fonts
let font_path = std::path::PathBuf::from("arial.ttf");
let font_info = font_manager.add_font(&font_path)?;

// List loaded fonts
let fonts = font_manager.get_fonts()?;
for font in fonts {
    println!("{}: {} ({})", font.family_name, font.style, font.path.display());
}

// Install font system-wide
font_manager.install_font(&font_info)?;

// Check installation status
if font_info.is_installed {
    println!("Font is installed system-wide");
}
```

### Multiple Backend Rendering

```rust
use testypf_core::{RendererBackend, RenderSettings};

let text_renderer = engine.text_renderer();

// List available backends
let backends = text_renderer.get_backends();
for backend in &backends {
    println!("Available backend: {:?}", backend);
}

// Switch backend
text_renderer.set_backend(RendererBackend::Orge)?;
text_renderer.set_backend(RendererBackend::Json)?;

// Render with different backends
let mut results = Vec::new();
for backend in backends {
    text_renderer.set_backend(backend.clone())?;
    let result = text_renderer.render_text(&font_path, &settings)?;
    results.push((backend, result));
}
```

## Rendering Backends

### Available Backends

| Backend | Description | Platform Support |
|---------|-------------|------------------|
| Orge | Vector rendering, pure Rust | ✅ macOS, ✅ Windows |
| Json | Debug output, data export | ✅ macOS, ✅ Windows |
| CoreGraphics | macOS native rendering | ✅ macOS |
| Skia | GPU-accelerated rendering | ✅ macOS, ✅ Windows |
| Zeno | Experimental renderer | ✅ macOS, ✅ Windows |

### Backend Configuration

```rust
// Configure render settings
let settings = RenderSettings {
    sample_text: "The quick brown fox".to_string(),
    font_size: 18.0,
    foreground_color: (0, 0, 0, 255),      // Black text
    background_color: Some((255, 255, 255, 255)), // White background
    backend: RendererBackend::Orge,
    padding: 10,
};

// Transparent background
let settings = RenderSettings {
    background_color: None, // Transparent
    ..Default::default()
};
```

## Integration with Other Libraries

### FontLift Integration

TestYPF uses FontLift for font management operations:

```rust
use fontlift_core::FontScope;

// Install font for current user
engine.font_manager().install_font(&font_info);

// System-wide installation (requires admin)
// This would need elevated privileges in a real implementation
```

### Typf Integration

TestYPF integrates with Typf for text rendering:

```rust
// The text renderer internally uses Typf for shaping and rendering
// Different backends provide different Typf renderers
let renderer = engine.text_renderer();
```

## Error Handling

TestYPF provides comprehensive error handling:

```rust
use testypf_core::TestypfError;

match font_manager.add_font(&font_path) {
    Ok(font_info) => println!("Font loaded: {}", font_info.family_name),
    Err(TestypfError::InvalidFont(msg)) => {
        println!("Invalid font file: {}", msg);
    },
    Err(TestypfError::FontManagementFailed(msg)) => {
        println!("Font management error: {}", msg);
    },
    Err(TestypfError::RenderFailed(msg)) => {
        println!("Rendering failed: {}", msg);
    },
    Err(e) => println!("Other error: {}", e),
}
```

## Performance Considerations

### Optimization Tips

1. **Font Caching**: TestYPF caches font information after loading
2. **Lazy Rendering**: Text is only rendered when explicitly requested
3. **Backend Selection**: Choose appropriate backend for your use case:
   - Orge for fast vector rendering
   - Json for data export/debugging
   - CoreGraphics/Skia for GPU acceleration

### Memory Usage

- Font data is loaded on-demand and cached
- Render results are temporary unless explicitly saved
- Consider memory usage when loading many fonts

## File Support

### Supported Font Formats

- TrueType (.ttf, .ttc)
- OpenType (.otf, .otc)
- Web Open Font Format (.woff, .woff2)
- macOS dfont (.dfont)

### Drag and Drop

- Single font files supported
- Multiple file selection supported
- Font folders (recursive loading planned)

## Troubleshooting

### Common Issues

1. **Font Loading Fails**: Check file format and permissions
2. **Rendering Errors**: Verify backend availability and font compatibility
3. **Installation Failures**: Check system privileges and font protection

### Debug Mode

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug cargo run
```

### Performance Profiling

TestYPF includes performance monitoring for rendering operations. Check the status bar for timing information during rendering operations.
