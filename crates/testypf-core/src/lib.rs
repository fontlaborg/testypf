//! testypf-core - Core library for testypf GUI application
//!
//! This library provides the core functionality for the testypf GUI,
//! including integration with typf for rendering, fontlift for font management,
//! and typg for font discovery.

use std::path::PathBuf;
use thiserror::Error;

pub use fontlift_core::{FontScope, FontliftFontSource};

// Re-export discovery types for GUI use
pub use discovery::{DiscoveryManager, FontDiscoveryResult, SearchCriteria};

/// Core errors for testypf
#[derive(Error, Debug)]
pub enum TestypfError {
    #[error("Font rendering failed: {0}")]
    RenderFailed(String),

    #[error("Font management failed: {0}")]
    FontManagementFailed(String),

    #[error("Invalid font file: {0}")]
    InvalidFont(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Font discovery failed: {0}")]
    DiscoveryFailed(String),
}

/// Result type for testypf operations
pub type TestypfResult<T> = Result<T, TestypfError>;

/// Variable font axis information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TestypfVariationAxis {
    /// Four-character axis tag (e.g., "wght", "wdth", "ital")
    pub tag: String,

    /// Human-readable axis name
    pub name: String,

    /// Minimum value
    pub min_value: f32,

    /// Default value
    pub default_value: f32,

    /// Maximum value
    pub max_value: f32,
}

/// Font face information for GUI display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestypfFontInfo {
    /// Source information for the font (path, format, optional index)
    pub source: FontliftFontSource,

    /// PostScript name
    pub postscript_name: String,

    /// Full display name
    pub full_name: String,

    /// Font family name
    pub family_name: String,

    /// Font style
    pub style: String,

    /// Whether font is installed
    pub is_installed: bool,

    /// Variable font axes (empty if not a variable font)
    pub variation_axes: Vec<TestypfVariationAxis>,
}

impl TestypfFontInfo {
    pub fn path(&self) -> &PathBuf {
        &self.source.path
    }

    pub fn with_scope(&self, scope: FontScope) -> FontliftFontSource {
        self.source.clone().with_scope(Some(scope))
    }
}

/// Render settings for text rendering
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RenderSettings {
    /// Sample text to render
    pub sample_text: String,

    /// Font size in points
    pub font_size: f32,

    /// Foreground color (RGBA)
    pub foreground_color: (u8, u8, u8, u8),

    /// Background color (RGBA), None for transparent
    pub background_color: Option<(u8, u8, u8, u8)>,

    /// Renderer backend to use
    pub backend: RendererBackend,

    /// Render padding
    pub padding: u32,

    /// Variable font axis coordinates (tag -> value)
    #[serde(default)]
    pub variation_coords: std::collections::HashMap<String, f32>,
}

/// Available rendering backends
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum RendererBackend {
    Orge,
    Json,
    #[cfg(target_os = "macos")]
    CoreGraphics,
    #[cfg(feature = "render-skia")]
    Skia,
    #[cfg(feature = "render-zeno")]
    Zeno,
}

impl std::fmt::Display for RendererBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RendererBackend::Orge => write!(f, "Opixa"),
            RendererBackend::Json => write!(f, "Json"),
            #[cfg(target_os = "macos")]
            RendererBackend::CoreGraphics => write!(f, "CoreGraphics"),
            #[cfg(feature = "render-skia")]
            RendererBackend::Skia => write!(f, "Skia"),
            #[cfg(feature = "render-zeno")]
            RendererBackend::Zeno => write!(f, "Zeno"),
        }
    }
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            sample_text: "The quick brown fox jumps over the lazy dog".to_string(),
            font_size: 16.0,
            foreground_color: (0, 0, 0, 255),
            background_color: None,
            backend: RendererBackend::Orge,
            padding: 10,
            variation_coords: std::collections::HashMap::new(),
        }
    }
}

/// Render result containing bitmap data
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// Image width in pixels
    pub width: u32,

    /// Image height in pixels
    pub height: u32,

    /// RGBA pixel data
    pub data: Vec<u8>,

    /// Format description
    pub format: String,
}

/// Font manager interface
pub trait FontManager: Send + Sync {
    /// Add a font to the font list
    fn add_font(&mut self, source: &FontliftFontSource) -> TestypfResult<TestypfFontInfo>;

    /// Remove a font from the font list
    fn remove_font(&mut self, source: &FontliftFontSource) -> TestypfResult<()>;

    /// Get all fonts
    fn get_fonts(&self) -> TestypfResult<Vec<TestypfFontInfo>>;

    /// Check whether a font path is installed on the system
    fn is_font_installed(&self, source: &FontliftFontSource) -> TestypfResult<bool>;

    /// Set target installation scope (user/system)
    fn set_install_scope(&mut self, scope: FontScope);

    /// Get current installation scope
    fn install_scope(&self) -> FontScope;

    /// Install font using fontlift
    fn install_font(&mut self, font: &TestypfFontInfo) -> TestypfResult<()>;

    /// Uninstall font using fontlift
    fn uninstall_font(&mut self, font: &TestypfFontInfo) -> TestypfResult<()>;
}

/// Text renderer interface
pub trait TextRenderer: Send + Sync {
    /// Render text with given font and settings
    fn render_text(
        &self,
        font_path: &PathBuf,
        settings: &RenderSettings,
    ) -> TestypfResult<RenderResult>;

    /// Get available backends
    fn get_backends(&self) -> Vec<RendererBackend>;

    /// Set backend
    fn set_backend(&mut self, backend: RendererBackend) -> TestypfResult<()>;
}

/// Main testypf engine
pub struct TestypfEngine {
    font_manager: Box<dyn FontManager>,
    text_renderer: Box<dyn TextRenderer>,
}

impl TestypfEngine {
    /// Create a new testypf engine
    pub fn new() -> TestypfResult<Self> {
        let font_manager = Box::new(crate::font::FontListManager::new());
        let text_renderer = Box::new(crate::render::TypfRenderer::new()?);

        Ok(Self {
            font_manager,
            text_renderer,
        })
    }

    /// Get font manager
    pub fn font_manager(&mut self) -> &mut dyn FontManager {
        &mut *self.font_manager
    }

    /// Get text renderer
    pub fn text_renderer(&mut self) -> &mut dyn TextRenderer {
        &mut *self.text_renderer
    }

    /// Set the font installation scope (user vs system)
    pub fn set_font_install_scope(&mut self, scope: FontScope) {
        self.font_manager.set_install_scope(scope);
    }

    /// Get current font installation scope
    pub fn font_install_scope(&self) -> FontScope {
        self.font_manager.install_scope()
    }

    /// Determine if platform font install/uninstall is supported in this build
    pub fn font_ops_available() -> bool {
        crate::font::FontListManager::platform_support_enabled()
    }

    /// Render preview for multiple fonts
    pub fn render_previews(
        &mut self,
        settings: &RenderSettings,
    ) -> TestypfResult<Vec<(TestypfFontInfo, RenderResult)>> {
        let fonts = self.font_manager.get_fonts()?;
        let mut results = Vec::new();

        for font in fonts {
            let render_result = self
                .text_renderer
                .render_text(&font.source.path, settings)?;
            results.push((font, render_result));
        }

        Ok(results)
    }
}

/// Font management module
pub mod font {
    use super::*;
    use read_fonts::{FontRef, TableProvider};
    use std::sync::Arc;

    /// Font list manager that tracks fonts for the GUI
    pub struct FontListManager {
        fonts: Vec<TestypfFontInfo>,
        install_scope: FontScope,
        #[cfg(test)]
        platform_override: Option<Arc<dyn fontlift_core::FontManager>>,
    }

    impl FontListManager {
        /// Create a new font list manager
        pub fn new() -> Self {
            Self {
                fonts: Vec::new(),
                install_scope: FontScope::User,
                #[cfg(test)]
                platform_override: None,
            }
        }

        /// Determine whether platform font operations are available in this build
        pub fn platform_support_enabled() -> bool {
            cfg!(all(target_os = "macos", feature = "platform-mac"))
                || cfg!(all(target_os = "windows", feature = "platform-win"))
        }

        /// Extract font information from file using read-fonts crate and FontLift validation
        fn extract_font_info(&self, source: &FontliftFontSource) -> TestypfResult<TestypfFontInfo> {
            // Use FontLift's validation first
            fontlift_core::validation::validate_font_file(&source.path)
                .map_err(|e| TestypfError::InvalidFont(format!("Font validation failed: {}", e)))?;

            // Read font file
            let font_data = std::fs::read(&source.path).map_err(|e| {
                TestypfError::InvalidFont(format!("Failed to read font file: {}", e))
            })?;

            let font = FontRef::from_index(&font_data, 0)
                .map_err(|e| TestypfError::InvalidFont(format!("Failed to parse font: {}", e)))?;

            // Extract variable font axes from fvar table if present
            let variation_axes = Self::extract_variation_axes(&font);

            // For now, use filename-based extraction since FontLift validation already confirmed it's a valid font
            // The font parsing and name table extraction can be improved later
            let postscript_name = source
                .path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let full_name = postscript_name.clone();
            let family_name = postscript_name.clone();
            let style = "Regular".to_string();

            let mut font_info = TestypfFontInfo {
                source: source.clone(),
                postscript_name,
                full_name,
                family_name,
                style,
                is_installed: false,
                variation_axes,
            };

            // Check if font is already installed using FontLift
            if let Ok(font_manager) = self.create_platform_font_manager() {
                match font_manager.is_font_installed(source) {
                    Ok(is_installed) => {
                        font_info.is_installed = is_installed;
                    }
                    Err(e) => {
                        // Don't fail font loading due to installation check failure
                        eprintln!("Warning: Could not check font installation status: {}", e);
                    }
                }
            }

            Ok(font_info)
        }

        /// Extract variation axes from font's fvar table
        fn extract_variation_axes(font: &FontRef) -> Vec<TestypfVariationAxis> {
            let fvar = match font.fvar() {
                Ok(fvar) => fvar,
                Err(_) => return Vec::new(), // Not a variable font
            };

            let axes = match fvar.axes() {
                Ok(axes) => axes,
                Err(_) => return Vec::new(),
            };

            axes.iter()
                .map(|axis| {
                    let tag_bytes = axis.axis_tag().to_be_bytes();
                    let tag = String::from_utf8_lossy(&tag_bytes).to_string();

                    // Use tag as name for now; could look up in name table
                    let name = match tag.as_str() {
                        "wght" => "Weight".to_string(),
                        "wdth" => "Width".to_string(),
                        "ital" => "Italic".to_string(),
                        "slnt" => "Slant".to_string(),
                        "opsz" => "Optical Size".to_string(),
                        _ => tag.clone(),
                    };

                    TestypfVariationAxis {
                        tag,
                        name,
                        min_value: axis.min_value().to_f32(),
                        default_value: axis.default_value().to_f32(),
                        max_value: axis.max_value().to_f32(),
                    }
                })
                .collect()
        }

        #[cfg(test)]
        pub fn with_platform_override(manager: Arc<dyn fontlift_core::FontManager>) -> Self {
            Self {
                fonts: Vec::new(),
                install_scope: FontScope::User,
                platform_override: Some(manager),
            }
        }

        #[cfg(test)]
        pub fn push_font_for_tests(&mut self, font: TestypfFontInfo) {
            self.fonts.push(font);
        }

        fn platform_manager(&self) -> TestypfResult<Arc<dyn fontlift_core::FontManager>> {
            #[cfg(test)]
            if let Some(manager) = &self.platform_override {
                return Ok(manager.clone());
            }

            self.create_platform_font_manager()
        }

        /// Create platform-specific font manager for real font operations
        fn create_platform_font_manager(
            &self,
        ) -> TestypfResult<Arc<dyn fontlift_core::FontManager>> {
            #[cfg(target_os = "macos")]
            {
                #[cfg(feature = "platform-mac")]
                {
                    let manager = Arc::new(fontlift_platform_mac::MacFontManager::new());
                    return Ok(manager);
                }
                #[cfg(not(feature = "platform-mac"))]
                {
                    return Err(TestypfError::FontManagementFailed(
                        "macOS platform support not enabled. Build with --features platform-mac"
                            .to_string(),
                    ));
                }
            }

            #[cfg(target_os = "windows")]
            {
                #[cfg(feature = "platform-win")]
                {
                    let manager = Arc::new(fontlift_platform_win::WinFontManager::new());
                    return Ok(manager);
                }
                #[cfg(not(feature = "platform-win"))]
                {
                    return Err(TestypfError::FontManagementFailed(
                        "Windows platform support not enabled. Build with --features platform-win"
                            .to_string(),
                    ));
                }
            }

            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            {
                return Err(TestypfError::FontManagementFailed(
                    "Font management not supported on this platform".to_string(),
                ));
            }
        }
    }

    impl super::FontManager for FontListManager {
        fn set_install_scope(&mut self, scope: FontScope) {
            self.install_scope = scope;
        }

        fn install_scope(&self) -> FontScope {
            self.install_scope
        }

        fn add_font(&mut self, source: &FontliftFontSource) -> TestypfResult<TestypfFontInfo> {
            let font_info = self.extract_font_info(source)?;

            // Check if font already exists
            if self.fonts.iter().any(|f| f.source.path == source.path) {
                return Err(TestypfError::InvalidFont("Font already added".to_string()));
            }

            self.fonts.push(font_info.clone());
            Ok(font_info)
        }

        fn remove_font(&mut self, source: &FontliftFontSource) -> TestypfResult<()> {
            self.fonts.retain(|f| f.source.path != source.path);
            Ok(())
        }

        fn get_fonts(&self) -> TestypfResult<Vec<TestypfFontInfo>> {
            Ok(self.fonts.clone())
        }

        fn is_font_installed(&self, source: &FontliftFontSource) -> TestypfResult<bool> {
            let font_manager = self.platform_manager().map_err(|e| {
                TestypfError::FontManagementFailed(format!(
                    "Failed to create platform font manager: {}",
                    e
                ))
            })?;

            font_manager.is_font_installed(source).map_err(|e| {
                TestypfError::FontManagementFailed(format!(
                    "Failed to check font installation status: {}",
                    e
                ))
            })
        }

        fn install_font(&mut self, font: &TestypfFontInfo) -> TestypfResult<()> {
            // Use real FontLift integration
            let font_manager = self.platform_manager().map_err(|e| {
                TestypfError::FontManagementFailed(format!(
                    "Failed to create platform font manager: {}",
                    e
                ))
            })?;

            // Validate font before installation
            fontlift_core::validation::validate_font_file(&font.source.path).map_err(|e| {
                TestypfError::FontManagementFailed(format!("Font validation failed: {}", e))
            })?;

            let source_with_scope = font.with_scope(self.install_scope);

            // Install font at user level (safer default)
            font_manager.install_font(&source_with_scope).map_err(|e| {
                TestypfError::FontManagementFailed(format!("Font installation failed: {}", e))
            })?;

            // Update local font list state
            if let Some(index) = self
                .fonts
                .iter_mut()
                .position(|f| f.source.path == font.source.path)
            {
                self.fonts[index].is_installed = true;

                // Verify installation was successful
                match font_manager.is_font_installed(&font.source) {
                    Ok(true) => {
                        // Installation confirmed
                    }
                    Ok(false) => {
                        self.fonts[index].is_installed = false;
                        return Err(TestypfError::FontManagementFailed(
                            "Font installation verification failed".to_string(),
                        ));
                    }
                    Err(e) => {
                        // Still mark as installed but log warning
                        eprintln!("Warning: Could not verify font installation: {}", e);
                    }
                }
            } else {
                return Err(TestypfError::FontManagementFailed(
                    "Font not found in list".to_string(),
                ));
            }

            Ok(())
        }

        fn uninstall_font(&mut self, font: &TestypfFontInfo) -> TestypfResult<()> {
            // Use real FontLift integration
            let font_manager = self.platform_manager().map_err(|e| {
                TestypfError::FontManagementFailed(format!(
                    "Failed to create platform font manager: {}",
                    e
                ))
            })?;

            // Check if font is actually installed before uninstalling
            let is_installed = font_manager.is_font_installed(&font.source).map_err(|e| {
                TestypfError::FontManagementFailed(format!(
                    "Failed to check font installation status: {}",
                    e
                ))
            })?;

            if !is_installed {
                return Err(TestypfError::FontManagementFailed(
                    "Font is not installed".to_string(),
                ));
            }

            let source_with_scope = font.with_scope(self.install_scope);

            // Uninstall font from user level
            font_manager
                .uninstall_font(&source_with_scope)
                .map_err(|e| {
                    TestypfError::FontManagementFailed(format!("Font uninstallation failed: {}", e))
                })?;

            // Update local font list state
            if let Some(index) = self
                .fonts
                .iter_mut()
                .position(|f| f.source.path == font.source.path)
            {
                self.fonts[index].is_installed = false;

                // Verify uninstallation was successful
                match font_manager.is_font_installed(&font.source) {
                    Ok(false) => {
                        // Uninstallation confirmed
                    }
                    Ok(true) => {
                        return Err(TestypfError::FontManagementFailed(
                            "Font uninstallation verification failed".to_string(),
                        ));
                    }
                    Err(e) => {
                        // Still mark as uninstalled but log warning
                        eprintln!("Warning: Could not verify font uninstallation: {}", e);
                    }
                }
            } else {
                return Err(TestypfError::FontManagementFailed(
                    "Font not found in list".to_string(),
                ));
            }

            Ok(())
        }
    }
}

/// Text rendering module
pub mod render {
    use super::*;
    use pyo3::{
        types::{
            PyAnyMethods, PyBytes, PyBytesMethods, PyDict, PyDictMethods, PyString, PyStringMethods,
        },
        PyObject, Python,
    };
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    /// Python typf module cache
    static TYPF_MODULE: OnceLock<Mutex<Option<PyObject>>> = OnceLock::new();

    /// Initialize the typf Python module
    fn ensure_typf_module() -> TestypfResult<()> {
        let module_cache = TYPF_MODULE.get_or_init(|| Mutex::new(None));
        let mut guard = module_cache.lock().unwrap();

        if guard.is_none() {
            let typf_module = Python::with_gil(|py| {
                // Import the typfpy Python module (typf bindings)
                let typf_module = py.import_bound("typfpy").map_err(|e| {
                    TestypfError::RenderFailed(format!("Failed to import typfpy module: {}", e))
                })?;

                Ok::<PyObject, TestypfError>(typf_module.into())
            })?;

            *guard = Some(typf_module);
        }

        Ok(())
    }

    /// Typf-based text renderer using Python bindings
    pub struct TypfRenderer {
        shaper: String,
        backend: String,
    }

    impl TypfRenderer {
        /// Create a new typf renderer
        pub fn new() -> TestypfResult<Self> {
            // Ensure typf Python module is available
            ensure_typf_module()?;

            Ok(Self {
                shaper: "harfbuzz".to_string(),
                backend: "opixa".to_string(),
            })
        }

        /// Convert backend enum to typf string
        fn backend_to_string(&self, backend: &RendererBackend) -> &str {
            match backend {
                RendererBackend::Orge => "opixa",
                RendererBackend::Json => "json",
                #[cfg(target_os = "macos")]
                RendererBackend::CoreGraphics => "coregraphics",
                #[cfg(feature = "render-skia")]
                RendererBackend::Skia => "skia",
                #[cfg(feature = "render-zeno")]
                RendererBackend::Zeno => "zeno",
            }
        }

        /// Render using actual typf Python bindings
        fn render_with_typf(
            &self,
            font_path: &PathBuf,
            settings: &RenderSettings,
        ) -> TestypfResult<RenderResult> {
            Python::with_gil(|py| {
                // Get the cached typf module
                let module_cache = TYPF_MODULE.get().ok_or_else(|| {
                    TestypfError::RenderFailed("Typf module cache not initialized".to_string())
                })?;

                let guard = module_cache.lock().unwrap();
                let typf_module = guard.as_ref().ok_or_else(|| {
                    TestypfError::RenderFailed("Typf module not initialized".to_string())
                })?;

                // Create typf instance with current backend
                let typf_class = typf_module.getattr(py, "Typf").map_err(|e| {
                    TestypfError::RenderFailed(format!("Failed to get Typf class: {}", e))
                })?;

                let typf_instance = typf_class
                    .call1(py, (self.shaper.as_str(), self.backend.as_str()))
                    .map_err(|e| {
                        TestypfError::RenderFailed(format!("Failed to create Typf instance: {}", e))
                    })?;

                // Prepare rendering parameters
                let color = Some(settings.foreground_color);
                let background = settings.background_color;
                let font_path_str = font_path.to_string_lossy();
                let variations: HashMap<String, f32> = settings
                    .variation_coords
                    .iter()
                    .map(|(tag, value)| (tag.clone(), *value))
                    .collect();

                // Call render_text method
                let result = typf_instance
                    .call_method1(
                        py,
                        "render_text",
                        (
                            settings.sample_text.as_str(),
                            font_path_str.as_ref(),
                            settings.font_size,
                            color,
                            background,
                            settings.padding,
                            if variations.is_empty() {
                                None
                            } else {
                                Some(variations)
                            },
                        ),
                    )
                    .map_err(|e| {
                        TestypfError::RenderFailed(format!("Failed to render text: {}", e))
                    })?;

                // Typf Python bindings return either a dict (bitmap) or a JSON string.
                Self::convert_py_result(py, result)
            })
        }

        /// Convert typf Python output into a Rust RenderResult
        fn convert_py_result(py: Python<'_>, result: PyObject) -> TestypfResult<RenderResult> {
            // Bitmap dictionary
            if let Ok(dict) = result.downcast_bound::<PyDict>(py) {
                let width_obj = dict.get_item("width").map_err(|e| {
                    TestypfError::RenderFailed(format!("Failed to read width: {e}"))
                })?;
                let width: u32 = width_obj
                    .ok_or_else(|| {
                        TestypfError::RenderFailed("Missing width in typf result".to_string())
                    })?
                    .extract()
                    .map_err(|e| {
                        TestypfError::RenderFailed(format!("Invalid width in typf result: {e}"))
                    })?;

                let height_obj = dict.get_item("height").map_err(|e| {
                    TestypfError::RenderFailed(format!("Failed to read height: {e}"))
                })?;
                let height: u32 = height_obj
                    .ok_or_else(|| {
                        TestypfError::RenderFailed("Missing height in typf result".to_string())
                    })?
                    .extract()
                    .map_err(|e| {
                        TestypfError::RenderFailed(format!("Invalid height in typf result: {e}"))
                    })?;

                let format: String = dict
                    .get_item("format")
                    .ok()
                    .flatten()
                    .and_then(|item| item.extract().ok())
                    .unwrap_or_else(|| "unknown".to_string());

                let data_obj = dict
                    .get_item("data")
                    .map_err(|e| TestypfError::RenderFailed(format!("Failed to read data: {e}")))?;
                let data: Vec<u8> = data_obj
                    .ok_or_else(|| {
                        TestypfError::RenderFailed("Missing data in typf result".to_string())
                    })?
                    .downcast::<PyBytes>()
                    .map_err(|e| {
                        TestypfError::RenderFailed(format!(
                            "Invalid data format in typf result: {e}"
                        ))
                    })?
                    .as_bytes()
                    .to_vec();

                return Ok(RenderResult {
                    width,
                    height,
                    data,
                    format,
                });
            }

            // JSON renderer returns a string
            if let Ok(json_str) = result.downcast_bound::<PyString>(py) {
                let data = json_str.to_string_lossy().as_bytes().to_vec();
                return Ok(RenderResult {
                    width: 0,
                    height: 0,
                    data,
                    format: "json".to_string(),
                });
            }

            Err(TestypfError::RenderFailed(
                "Unexpected typf render result type".to_string(),
            ))
        }
    }

    impl TextRenderer for TypfRenderer {
        fn render_text(
            &self,
            font_path: &PathBuf,
            settings: &RenderSettings,
        ) -> TestypfResult<RenderResult> {
            // Use real Typf integration via Python bindings
            self.render_with_typf(font_path, settings)
        }

        fn get_backends(&self) -> Vec<RendererBackend> {
            vec![
                RendererBackend::Orge,
                RendererBackend::Json,
                #[cfg(target_os = "macos")]
                RendererBackend::CoreGraphics,
                #[cfg(feature = "render-skia")]
                RendererBackend::Skia,
                #[cfg(feature = "render-zeno")]
                RendererBackend::Zeno,
            ]
        }

        fn set_backend(&mut self, backend: RendererBackend) -> TestypfResult<()> {
            self.backend = self.backend_to_string(&backend).to_string();
            Ok(())
        }
    }
}

/// Font discovery module using typg
pub mod discovery {
    use super::*;
    use typg_core::query::Query;
    use typg_core::search::{search, SearchOptions, TypgFontFaceMatch};

    /// Search criteria for font discovery
    #[derive(Debug, Clone, Default)]
    pub struct SearchCriteria {
        /// Name pattern to search for (regex)
        pub name_pattern: Option<String>,
        /// Required OpenType features (e.g., "smcp", "liga")
        pub features: Vec<String>,
        /// Required OpenType scripts (e.g., "latn", "cyrl")
        pub scripts: Vec<String>,
        /// Required font axes (e.g., "wght", "wdth") - variable fonts only
        pub axes: Vec<String>,
        /// Only match variable fonts
        pub variable_only: bool,
        /// Follow symlinks when scanning directories
        pub follow_symlinks: bool,
    }

    /// Result from font discovery search
    #[derive(Debug, Clone)]
    pub struct FontDiscoveryResult {
        /// Path to the font file
        pub path: PathBuf,
        /// Font names extracted from the file
        pub names: Vec<String>,
        /// OpenType feature tags present in the font
        pub features: Vec<String>,
        /// OpenType script tags present in the font
        pub scripts: Vec<String>,
        /// Whether the font is a variable font
        pub is_variable: bool,
        /// TTC index if applicable
        pub ttc_index: Option<u32>,
    }

    impl From<TypgFontFaceMatch> for FontDiscoveryResult {
        fn from(m: TypgFontFaceMatch) -> Self {
            Self {
                path: m.source.path,
                names: m.metadata.names,
                features: m
                    .metadata
                    .feature_tags
                    .iter()
                    .map(|t| t.to_string())
                    .collect(),
                scripts: m
                    .metadata
                    .script_tags
                    .iter()
                    .map(|t| t.to_string())
                    .collect(),
                is_variable: m.metadata.is_variable,
                ttc_index: m.source.ttc_index,
            }
        }
    }

    /// Manager for font discovery operations
    pub struct DiscoveryManager {
        default_roots: Vec<PathBuf>,
    }

    impl DiscoveryManager {
        /// Create a new discovery manager with default system font directories
        pub fn new() -> Self {
            let mut roots = Vec::new();

            #[cfg(target_os = "macos")]
            {
                // macOS font directories
                if let Some(home) = std::env::var_os("HOME") {
                    let user_fonts = PathBuf::from(home).join("Library/Fonts");
                    if user_fonts.exists() {
                        roots.push(user_fonts);
                    }
                }
                let system_fonts = PathBuf::from("/Library/Fonts");
                if system_fonts.exists() {
                    roots.push(system_fonts);
                }
                let system_core_fonts = PathBuf::from("/System/Library/Fonts");
                if system_core_fonts.exists() {
                    roots.push(system_core_fonts);
                }
            }

            #[cfg(target_os = "windows")]
            {
                // Windows font directory
                if let Some(windir) = std::env::var_os("WINDIR") {
                    let fonts = PathBuf::from(windir).join("Fonts");
                    if fonts.exists() {
                        roots.push(fonts);
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                // Linux font directories
                let system_fonts = PathBuf::from("/usr/share/fonts");
                if system_fonts.exists() {
                    roots.push(system_fonts);
                }
                let local_fonts = PathBuf::from("/usr/local/share/fonts");
                if local_fonts.exists() {
                    roots.push(local_fonts);
                }
                if let Some(home) = std::env::var_os("HOME") {
                    let user_fonts = PathBuf::from(home).join(".local/share/fonts");
                    if user_fonts.exists() {
                        roots.push(user_fonts);
                    }
                }
            }

            Self {
                default_roots: roots,
            }
        }

        /// Get default font search roots for the current platform
        pub fn default_roots(&self) -> &[PathBuf] {
            &self.default_roots
        }

        /// Search for fonts matching the given criteria in specific directories
        pub fn search_in(
            &self,
            roots: &[PathBuf],
            criteria: &SearchCriteria,
        ) -> TestypfResult<Vec<FontDiscoveryResult>> {
            let query = self.build_query(criteria)?;
            let opts = SearchOptions {
                follow_symlinks: criteria.follow_symlinks,
                jobs: None,
            };

            let matches = search(roots, &query, &opts)
                .map_err(|e| TestypfError::DiscoveryFailed(e.to_string()))?;

            Ok(matches.into_iter().map(FontDiscoveryResult::from).collect())
        }

        /// Search for fonts matching the given criteria in default system directories
        pub fn search_system(
            &self,
            criteria: &SearchCriteria,
        ) -> TestypfResult<Vec<FontDiscoveryResult>> {
            self.search_in(&self.default_roots, criteria)
        }

        /// Build a typg Query from our SearchCriteria
        fn build_query(&self, criteria: &SearchCriteria) -> TestypfResult<Query> {
            use typg_core::query::parse_tag_list;

            let mut query = Query::new();

            if !criteria.features.is_empty() {
                let tags = parse_tag_list(&criteria.features).map_err(|e| {
                    TestypfError::DiscoveryFailed(format!("Invalid feature: {}", e))
                })?;
                query = query.with_features(tags);
            }

            if !criteria.scripts.is_empty() {
                let tags = parse_tag_list(&criteria.scripts)
                    .map_err(|e| TestypfError::DiscoveryFailed(format!("Invalid script: {}", e)))?;
                query = query.with_scripts(tags);
            }

            if !criteria.axes.is_empty() {
                let tags = parse_tag_list(&criteria.axes)
                    .map_err(|e| TestypfError::DiscoveryFailed(format!("Invalid axis: {}", e)))?;
                query = query.with_axes(tags);
            }

            if criteria.variable_only {
                query = query.require_variable(true);
            }

            if let Some(ref pattern) = criteria.name_pattern {
                let re = regex::Regex::new(pattern).map_err(|e| {
                    TestypfError::DiscoveryFailed(format!("Invalid name pattern: {}", e))
                })?;
                query = query.with_name_patterns(vec![re]);
            }

            Ok(query)
        }
    }

    impl Default for DiscoveryManager {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests;
