//! Helper functions for testypf GUI application.

use crate::message::Message;
use crate::types::{
    AppConfig, DropPathKind, InstallScope, LayoutMode, RenderPreview, ScanStats, FONT_EXTENSIONS,
};
use iced::widget::image::Handle;
use iced::{keyboard, Element};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use testypf_core::{
    RenderResult, RenderSettings, RendererBackend, TestypfFontInfo, TestypfVariationAxis,
};

// =============================================================================
// Font File Operations
// =============================================================================

/// Determine whether a file path looks like a supported font.
pub fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| FONT_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Categorize a dropped path to drive validation messaging.
pub fn classify_drop_path(path: &Path) -> DropPathKind {
    if !path.exists() {
        return DropPathKind::Missing;
    }

    if path.is_dir() {
        return DropPathKind::Directory;
    }

    if path.is_file() && is_font_file(path) {
        DropPathKind::FontFile
    } else {
        DropPathKind::Unsupported
    }
}

/// Recursively scan a folder for font files with progress feedback.
pub fn scan_folder_for_fonts(folder_path: &PathBuf) -> Result<(Vec<PathBuf>, ScanStats), String> {
    let mut font_files = Vec::new();
    let mut stats = ScanStats::default();

    fn visit_dir(
        dir: &PathBuf,
        font_files: &mut Vec<PathBuf>,
        stats: &mut ScanStats,
    ) -> Result<(), std::io::Error> {
        stats.directories_scanned += 1;

        if dir.is_dir() {
            let entries = std::fs::read_dir(dir)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                stats.files_checked += 1;

                if path.is_dir() {
                    visit_dir(&path, font_files, stats)?;
                } else if path.is_file() && is_font_file(&path) {
                    stats.fonts_found += 1;
                    if stats.sample_files.len() < 3 {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            stats.sample_files.push(name.to_string());
                        }
                    }
                    font_files.push(path);
                }
            }
        }
        Ok(())
    }

    println!("Scanning folder: {:?}", folder_path);

    if let Err(e) = visit_dir(folder_path, &mut font_files, &mut stats) {
        return Err(format!(
            "Failed to scan directory after scanning {} directories and {} files: {}",
            stats.directories_scanned, stats.files_checked, e
        ));
    }

    println!(
        "Folder scan complete: found {} font files in {} directories, {} total files checked",
        font_files.len(),
        stats.directories_scanned,
        stats.files_checked
    );

    Ok((font_files, stats))
}

/// Aggregate extension counts for status messaging.
pub fn extension_stats(paths: &[PathBuf]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for path in paths {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if FONT_EXTENSIONS.contains(&ext_lower.as_str()) {
                *counts.entry(ext_lower).or_insert(0) += 1;
            }
        }
    }
    counts
}

/// Format extension statistics as a human-readable summary.
pub fn format_extension_summary(ext_stats: &HashMap<String, usize>) -> String {
    if ext_stats.is_empty() {
        return String::new();
    }

    let mut parts: Vec<String> = ext_stats
        .iter()
        .map(|(ext, count)| format!("{}x .{}", count, ext))
        .collect();
    parts.sort();
    format!(" Formats: {}", parts.join(", "))
}

/// Get supported formats as display text.
pub fn supported_formats_text() -> String {
    let mut exts = FONT_EXTENSIONS
        .iter()
        .map(|ext| format!(".{}", ext))
        .collect::<Vec<_>>();
    exts.sort();
    exts.join(", ")
}

// =============================================================================
// Font Metadata
// =============================================================================

/// Derive user-friendly metadata lines for a font.
pub fn font_metadata_lines(font: &TestypfFontInfo, file_size_bytes: Option<u64>) -> Vec<String> {
    let mut lines = vec![
        format!("Name: {}", font.full_name),
        format!("Family: {}", font.family_name),
        format!("Style: {}", font.style),
        format!("PostScript: {}", font.postscript_name),
        format!(
            "Install state: {}",
            if font.is_installed {
                "Installed"
            } else {
                "Not installed"
            }
        ),
    ];

    if let Some(size) = file_size_bytes {
        lines.push(format!("File size: {}", format_file_size(size)));
    }

    lines.push(format!("Path: {}", font.path().display()));
    lines
}

// =============================================================================
// Variable Fonts
// =============================================================================

/// Clamp a variation coordinate to the axis range.
pub fn clamp_variation_value(value: f32, axis: &TestypfVariationAxis) -> f32 {
    value.max(axis.min_value).min(axis.max_value)
}

/// Align render settings to the provided axes: seed defaults, clamp existing, and drop unknowns.
pub fn sync_variations_for_axes(settings: &mut RenderSettings, axes: &[TestypfVariationAxis]) {
    let mut updated = HashMap::new();

    for axis in axes {
        let current = settings
            .variation_coords
            .get(&axis.tag)
            .copied()
            .unwrap_or(axis.default_value);
        let clamped = clamp_variation_value(current, axis);
        updated.insert(axis.tag.clone(), clamped);
    }

    settings.variation_coords = updated;
}

/// Human-readable summary for variation coordinates (sorted by tag).
pub fn variation_summary(settings: &RenderSettings) -> Option<String> {
    if settings.variation_coords.is_empty() {
        return None;
    }

    let mut pairs: Vec<_> = settings.variation_coords.iter().collect();
    pairs.sort_by(|a, b| a.0.cmp(b.0));

    let parts: Vec<String> = pairs
        .into_iter()
        .map(|(tag, value)| format!("{tag}={value:.1}"))
        .collect();

    Some(parts.join(", "))
}

/// Get file size in bytes.
pub fn font_file_size(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|m| m.len())
}

/// Format file size for display.
pub fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;

    if bytes < 1024 {
        return format!("{} B", bytes);
    }

    let bytes_f = bytes as f64;
    if bytes_f < MB {
        return format!("{:.1} KB", bytes_f / KB);
    }

    format!("{:.1} MB", bytes_f / MB)
}

// =============================================================================
// Rendering
// =============================================================================

/// Convert Typf RenderResult (RGBA8) into an iced image handle.
pub fn image_handle_from_render(render_result: &RenderResult) -> Result<Handle, String> {
    let expected_len = (render_result.width as usize)
        .saturating_mul(render_result.height as usize)
        .saturating_mul(4);
    if expected_len == 0 {
        return Err("Render result contains no pixel data".to_string());
    }

    if render_result.data.len() != expected_len {
        return Err(format!(
            "Pixel data length mismatch (expected {}, got {})",
            expected_len,
            render_result.data.len()
        ));
    }

    if !render_result.format.to_lowercase().contains("rgba") {
        return Err(format!(
            "Unsupported render format for preview: {}",
            render_result.format
        ));
    }

    Ok(Handle::from_pixels(
        render_result.width,
        render_result.height,
        render_result.data.clone(),
    ))
}

/// Build a render preview with an iced image handle.
pub fn build_render_preview(
    font_index: usize,
    render_result: RenderResult,
    duration_ms: u128,
) -> Result<RenderPreview, String> {
    let handle = image_handle_from_render(&render_result)?;
    Ok(RenderPreview {
        font_index,
        width: render_result.width,
        height: render_result.height,
        format: render_result.format.clone(),
        pixels: render_result.data.clone(),
        handle,
        duration_ms,
    })
}

/// Human-readable metadata string for a render preview.
pub fn preview_metadata_text(
    preview: &RenderPreview,
    font: &TestypfFontInfo,
    settings: &RenderSettings,
) -> String {
    let variation_text = variation_summary(settings)
        .map(|s| format!(" | Variations: {}", s))
        .unwrap_or_default();

    format!(
        "Dimensions: {}x{} | Format: {} | Backend: {} | Style: {} | Family: {} | Render time: {} ms{}",
        preview.width,
        preview.height,
        preview.format,
        settings.backend,
        font.style,
        font.family_name,
        preview.duration_ms,
        variation_text
    )
}

/// Save a render preview to a PNG file on disk.
pub fn export_preview_to_path(preview: &RenderPreview, path: &Path) -> Result<(), String> {
    if preview.width == 0 || preview.height == 0 {
        return Err("Preview has zero dimensions".to_string());
    }

    if preview.pixels.len()
        != (preview.width as usize)
            .saturating_mul(preview.height as usize)
            .saturating_mul(4)
    {
        return Err("Preview pixel data length is invalid".to_string());
    }

    ::image::save_buffer_with_format(
        path,
        &preview.pixels,
        preview.width,
        preview.height,
        ::image::ColorType::Rgba8,
        ::image::ImageFormat::Png,
    )
    .map_err(|e| format!("Failed to write preview PNG: {}", e))
}

/// Make a filesystem-safe filename stem from user-facing font names.
pub fn sanitized_file_stem(name: &str) -> String {
    let mut stem: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    while stem.contains("__") {
        stem = stem.replace("__", "_");
    }

    stem.trim_matches('_')
        .chars()
        .take(40)
        .collect::<String>()
        .to_lowercase()
}

// =============================================================================
// Color Parsing
// =============================================================================

/// Parse a hex RGBA color string into a tuple. Accepts #RRGGBB or #RRGGBBAA.
pub fn parse_rgba_hex(value: &str) -> Option<(u8, u8, u8, u8)> {
    let trimmed = value.trim();
    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
    let len = hex.len();

    if len != 6 && len != 8 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    let a = if len == 8 {
        u8::from_str_radix(&hex[6..8], 16).ok()?
    } else {
        255
    };

    Some((r, g, b, a))
}

// =============================================================================
// Cache Logic
// =============================================================================

/// Check if cached render settings match current settings.
pub fn should_use_cache(
    last_settings: &Option<RenderSettings>,
    last_paths: &[PathBuf],
    current_settings: &RenderSettings,
    current_paths: &[PathBuf],
    has_previews: bool,
) -> bool {
    if !has_previews {
        return false;
    }

    match last_settings {
        Some(previous) if previous == current_settings => last_paths == current_paths,
        _ => false,
    }
}

// =============================================================================
// Render Target Derivation
// =============================================================================

/// Derive which font indices to render.
pub fn derive_render_targets(
    selected: Option<usize>,
    visible_indices: &[usize],
    render_selected_only: bool,
) -> Vec<usize> {
    if render_selected_only {
        return selected.into_iter().collect();
    }

    visible_indices.to_vec()
}

// =============================================================================
// Keyboard Shortcuts
// =============================================================================

/// Map keyboard events to messages.
pub fn shortcut_to_message(event: &keyboard::Event) -> Option<Message> {
    if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
        if !modifiers.command() {
            return None;
        }

        let key_lower = match key.as_ref() {
            keyboard::Key::Character(ch) => ch.to_lowercase(),
            keyboard::Key::Named(keyboard::key::Named::Enter) => "enter".to_string(),
            _ => String::new(),
        };

        match key_lower.as_str() {
            "o" => Some(Message::AddFonts),
            "r" => Some(Message::RenderPreviews),
            "e" => Some(Message::ExportPreviews),
            "w" => Some(Message::OpenRenderWindow),
            _ => None,
        }
    } else {
        None
    }
}

// =============================================================================
// Font Install State
// =============================================================================

/// Update the cached install flag for a font.
pub fn set_install_state(fonts: &mut [TestypfFontInfo], index: usize, is_installed: bool) -> bool {
    if let Some(font) = fonts.get_mut(index) {
        font.is_installed = is_installed;
        true
    } else {
        false
    }
}

// =============================================================================
// Backend Info
// =============================================================================

/// Get human-readable description for a rendering backend.
pub fn get_backend_description(backend: &RendererBackend) -> String {
    match backend {
        RendererBackend::Orge => "Opixa (High quality raster)".to_string(),
        RendererBackend::Json => "JSON (debug output)".to_string(),
        #[cfg(target_os = "macos")]
        RendererBackend::CoreGraphics => "CoreGraphics (macOS native)".to_string(),
        #[cfg(feature = "render-skia")]
        RendererBackend::Skia => "Skia (GPU accelerated)".to_string(),
        #[cfg(feature = "render-zeno")]
        RendererBackend::Zeno => "Zeno (experimental vector)".to_string(),
    }
}

/// Provide backend-specific capabilities for quick reference.
pub fn backend_capabilities(backend: &RendererBackend) -> &'static str {
    match backend {
        RendererBackend::Orge => "Opixa: fast raster previews; best default choice",
        RendererBackend::Json => "JSON backend outputs metadata only (hidden in picker)",
        #[cfg(target_os = "macos")]
        RendererBackend::CoreGraphics => "CoreGraphics: macOS native, color-managed previews",
        #[cfg(feature = "render-skia")]
        RendererBackend::Skia => "Skia: GPU-accelerated bitmaps; enable render-skia feature",
        #[cfg(feature = "render-zeno")]
        RendererBackend::Zeno => "Zeno: experimental vector backend",
    }
}

// =============================================================================
// Error Messages
// =============================================================================

/// Provide a user-friendly error message for font install/uninstall failures.
pub fn friendly_font_op_error(action: &str, scope: InstallScope, raw: &str) -> String {
    let lower = raw.to_lowercase();

    if lower.contains("permission") || lower.contains("access denied") {
        let scope_hint = match scope {
            InstallScope::System => "System scope often needs admin rights; retry with User scope or run with elevated permissions.",
            InstallScope::User => "Switch to System scope or ensure the font file is readable/writable.",
        };
        return format!("{action} failed: {raw}. {scope_hint}");
    }

    if lower.contains("platform support not enabled") || lower.contains("build with --features") {
        return format!(
            "{action} failed: {raw}. Enable platform features (platform-mac/platform-win) when building to use font installs.",
        );
    }

    if lower.contains("not supported on this platform") {
        return format!(
            "{action} failed: {raw}. Platform font management is unavailable in this build.",
        );
    }

    format!("{action} failed: {raw}")
}

/// Provide a user-friendly render error with remediation guidance.
pub fn friendly_render_error(font_name: &str, raw: &str) -> String {
    let lower = raw.to_lowercase();

    if lower.contains("import typfpy") || lower.contains("no module named") {
        return format!(
            "Render failed for '{font_name}': typfpy Python module missing. Run ./build.sh --verify or ensure typfpy is on PYTHONPATH.",
        );
    }

    if lower.contains("backend") && lower.contains("unknown") {
        return format!(
            "Render failed for '{font_name}': backend unavailable. Use 'Test Backend' to verify installed backends.",
        );
    }

    format!("Failed to render font {font_name}: {raw}")
}

// =============================================================================
// Configuration
// =============================================================================

/// Get the config file path.
pub fn config_path() -> PathBuf {
    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("testypf-config.json")
}

/// Load configuration from disk.
pub fn load_config() -> Result<AppConfig, String> {
    let path = config_path();
    load_config_from(&path)
}

/// Load configuration from a specific path.
pub fn load_config_from(path: &Path) -> Result<AppConfig, String> {
    let contents = fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;
    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse config: {}", e))
}

/// Save configuration to disk.
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    save_config_to(&path, config)
}

/// Save configuration to a specific path.
pub fn save_config_to(path: &Path, config: &AppConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    let contents = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(path, contents).map_err(|e| format!("Failed to write config: {}", e))
}

// =============================================================================
// Layout
// =============================================================================

/// How many rows are needed for the chosen layout and preview count.
pub fn layout_row_count(preview_count: usize, mode: LayoutMode) -> usize {
    match mode {
        LayoutMode::Single => preview_count,
        LayoutMode::SideBySide => (preview_count + 1) / 2,
    }
}

/// Arrange preview cards into rows based on the selected layout.
pub fn layout_previews(
    previews: Vec<Element<'_, Message>>,
    mode: LayoutMode,
) -> Element<'_, Message> {
    use iced::widget::{column, container, row};
    use iced::Length;

    match mode {
        LayoutMode::Single => column(previews).spacing(10).into(),
        LayoutMode::SideBySide => {
            let mut rows = Vec::with_capacity(layout_row_count(previews.len(), mode));
            let mut iter = previews.into_iter();

            while let Some(first) = iter.next() {
                let second = iter.next();
                let mut row_children = vec![container(first).width(Length::FillPortion(1)).into()];
                if let Some(second) = second {
                    row_children.push(container(second).width(Length::FillPortion(1)).into());
                }

                rows.push(
                    row(row_children)
                        .spacing(12)
                        .width(Length::Fill)
                        .align_items(iced::Alignment::Start)
                        .into(),
                );
            }

            column(rows).spacing(12).into()
        }
    }
}
