//! Type definitions for testypf GUI application.

use iced::widget::image::Handle;
use testypf_core::{FontScope, RendererBackend};

/// Font installation scope (user vs system).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallScope {
    User,
    System,
}

impl InstallScope {
    pub const OPTIONS: [InstallScope; 2] = [InstallScope::User, InstallScope::System];

    pub fn to_font_scope(self) -> FontScope {
        match self {
            InstallScope::User => FontScope::User,
            InstallScope::System => FontScope::System,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            InstallScope::User => "User (~/Library/Fonts)",
            InstallScope::System => "System (/Library/Fonts, requires admin)",
        }
    }
}

impl std::fmt::Display for InstallScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}

/// Layout mode for preview display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Single,
    SideBySide,
}

impl LayoutMode {
    pub fn options() -> Vec<Self> {
        vec![Self::Single, Self::SideBySide]
    }
}

impl std::fmt::Display for LayoutMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutMode::Single => write!(f, "Single column"),
            LayoutMode::SideBySide => write!(f, "Side-by-side"),
        }
    }
}

/// Render availability state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderAvailability {
    Ready,
    NoFonts,
    NoFilterMatches,
    NeedsSelection,
}

impl RenderAvailability {
    pub fn derive(
        font_total: usize,
        visible_count: usize,
        render_selected_only: bool,
        selected: Option<usize>,
    ) -> Self {
        if font_total == 0 {
            return Self::NoFonts;
        }

        if render_selected_only && selected.is_none() {
            return Self::NeedsSelection;
        }

        if visible_count == 0 {
            return Self::NoFilterMatches;
        }

        Self::Ready
    }

    pub fn can_render(self) -> bool {
        matches!(self, Self::Ready)
    }

    pub fn cta_label(self) -> &'static str {
        match self {
            Self::Ready => "Render Previews",
            Self::NoFonts => "Add fonts to render",
            Self::NoFilterMatches => "Adjust filter to render",
            Self::NeedsSelection => "Select a font to render",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Ready => "Ready to render with current settings.",
            Self::NoFonts => "Load at least one font via Add Fonts or drag & drop.",
            Self::NoFilterMatches => "No fonts match the current filter; clear it to render.",
            Self::NeedsSelection => {
                "Choose a font before rendering when 'selected only' is enabled."
            }
        }
    }
}

/// Statistics from folder scanning.
#[derive(Debug, Clone, Default)]
pub struct ScanStats {
    pub directories_scanned: usize,
    pub files_checked: usize,
    pub fonts_found: usize,
    pub sample_files: Vec<String>,
}

/// Classification for dropped paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPathKind {
    FontFile,
    Directory,
    Unsupported,
    Missing,
}

/// Persistent application configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AppConfig {
    pub backend: RendererBackend,
}

/// Rendered font preview data.
#[derive(Clone)]
pub struct RenderPreview {
    pub font_index: usize,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub pixels: Vec<u8>,
    pub handle: Handle,
    pub duration_ms: u128,
}

/// Supported font file extensions.
pub const FONT_EXTENSIONS: &[&str] = &[
    "ttf", "otf", "ttc", "otc", "woff", "woff2", "dfont", "eot", "svg", "pfa", "pfb", "otb",
];
