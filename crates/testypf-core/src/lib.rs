//! testypf-core - Core library for testypf GUI application
//!
//! This library provides the core functionality for the testypf GUI,
//! including integration with typf for rendering and fontlift for font management.

use std::path::PathBuf;
use thiserror::Error;

pub use fontlift_core::FontScope;

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
}

/// Result type for testypf operations
pub type TestypfResult<T> = Result<T, TestypfError>;

/// Font information for GUI display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FontInfo {
    /// File path to the font
    pub path: PathBuf,

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
    fn add_font(&mut self, path: &PathBuf) -> TestypfResult<FontInfo>;

    /// Remove a font from the font list
    fn remove_font(&mut self, path: &PathBuf) -> TestypfResult<()>;

    /// Get all fonts
    fn get_fonts(&self) -> TestypfResult<Vec<FontInfo>>;

    /// Check whether a font path is installed on the system
    fn is_font_installed(&self, path: &PathBuf) -> TestypfResult<bool>;

    /// Set target installation scope (user/system)
    fn set_install_scope(&mut self, scope: FontScope);

    /// Get current installation scope
    fn install_scope(&self) -> FontScope;

    /// Install font using fontlift
    fn install_font(&mut self, font: &FontInfo) -> TestypfResult<()>;

    /// Uninstall font using fontlift
    fn uninstall_font(&mut self, font: &FontInfo) -> TestypfResult<()>;
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
    ) -> TestypfResult<Vec<(FontInfo, RenderResult)>> {
        let fonts = self.font_manager.get_fonts()?;
        let mut results = Vec::new();

        for font in fonts {
            let render_result = self.text_renderer.render_text(&font.path, settings)?;
            results.push((font, render_result));
        }

        Ok(results)
    }
}

/// Font management module
pub mod font {
    use super::*;
    use read_fonts::FontRef;
    use std::sync::Arc;

    /// Font list manager that tracks fonts for the GUI
    pub struct FontListManager {
        fonts: Vec<FontInfo>,
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
        fn extract_font_info(&self, path: &PathBuf) -> TestypfResult<FontInfo> {
            // Use FontLift's validation first
            fontlift_core::validation::validate_font_file(path)
                .map_err(|e| TestypfError::InvalidFont(format!("Font validation failed: {}", e)))?;

            // Read font file
            let font_data = std::fs::read(path).map_err(|e| {
                TestypfError::InvalidFont(format!("Failed to read font file: {}", e))
            })?;

            let _font = FontRef::from_index(&font_data, 0)
                .map_err(|e| TestypfError::InvalidFont(format!("Failed to parse font: {}", e)))?;

            // For now, use filename-based extraction since FontLift validation already confirmed it's a valid font
            // The font parsing and name table extraction can be improved later
            let postscript_name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let full_name = postscript_name.clone();
            let family_name = postscript_name.clone();
            let style = "Regular".to_string();

            let mut font_info = FontInfo {
                path: path.clone(),
                postscript_name,
                full_name,
                family_name,
                style,
                is_installed: false,
            };

            // Check if font is already installed using FontLift
            if let Ok(font_manager) = self.create_platform_font_manager() {
                match font_manager.is_font_installed(path) {
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

        #[cfg(test)]
        pub fn with_platform_override(manager: Arc<dyn fontlift_core::FontManager>) -> Self {
            Self {
                fonts: Vec::new(),
                install_scope: FontScope::User,
                platform_override: Some(manager),
            }
        }

        #[cfg(test)]
        pub fn push_font_for_tests(&mut self, font: FontInfo) {
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

        fn add_font(&mut self, path: &PathBuf) -> TestypfResult<FontInfo> {
            let font_info = self.extract_font_info(path)?;

            // Check if font already exists
            if self.fonts.iter().any(|f| f.path == *path) {
                return Err(TestypfError::InvalidFont("Font already added".to_string()));
            }

            self.fonts.push(font_info.clone());
            Ok(font_info)
        }

        fn remove_font(&mut self, path: &PathBuf) -> TestypfResult<()> {
            self.fonts.retain(|f| f.path != *path);
            Ok(())
        }

        fn get_fonts(&self) -> TestypfResult<Vec<FontInfo>> {
            Ok(self.fonts.clone())
        }

        fn is_font_installed(&self, path: &PathBuf) -> TestypfResult<bool> {
            let font_manager = self.platform_manager().map_err(|e| {
                TestypfError::FontManagementFailed(format!(
                    "Failed to create platform font manager: {}",
                    e
                ))
            })?;

            font_manager.is_font_installed(path).map_err(|e| {
                TestypfError::FontManagementFailed(format!(
                    "Failed to check font installation status: {}",
                    e
                ))
            })
        }

        fn install_font(&mut self, font: &FontInfo) -> TestypfResult<()> {
            // Use real FontLift integration
            let font_manager = self.platform_manager().map_err(|e| {
                TestypfError::FontManagementFailed(format!(
                    "Failed to create platform font manager: {}",
                    e
                ))
            })?;

            // Validate font before installation
            fontlift_core::validation::validate_font_file(&font.path).map_err(|e| {
                TestypfError::FontManagementFailed(format!("Font validation failed: {}", e))
            })?;

            // Install font at user level (safer default)
            font_manager
                .install_font(&font.path, self.install_scope)
                .map_err(|e| {
                    TestypfError::FontManagementFailed(format!("Font installation failed: {}", e))
                })?;

            // Update local font list state
            if let Some(index) = self.fonts.iter_mut().position(|f| f.path == font.path) {
                self.fonts[index].is_installed = true;

                // Verify installation was successful
                match font_manager.is_font_installed(&font.path) {
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

        fn uninstall_font(&mut self, font: &FontInfo) -> TestypfResult<()> {
            // Use real FontLift integration
            let font_manager = self.platform_manager().map_err(|e| {
                TestypfError::FontManagementFailed(format!(
                    "Failed to create platform font manager: {}",
                    e
                ))
            })?;

            // Check if font is actually installed before uninstalling
            let is_installed = font_manager.is_font_installed(&font.path).map_err(|e| {
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

            // Uninstall font from user level
            font_manager
                .uninstall_font(&font.path, self.install_scope)
                .map_err(|e| {
                    TestypfError::FontManagementFailed(format!("Font uninstallation failed: {}", e))
                })?;

            // Update local font list state
            if let Some(index) = self.fonts.iter_mut().position(|f| f.path == font.path) {
                self.fonts[index].is_installed = false;

                // Verify uninstallation was successful
                match font_manager.is_font_installed(&font.path) {
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
    use std::sync::{Mutex, OnceLock};

    /// Python typf module cache
    static TYPF_MODULE: OnceLock<Mutex<Option<PyObject>>> = OnceLock::new();

    /// Initialize the typf Python module
    fn ensure_typf_module() -> TestypfResult<()> {
        let module_cache = TYPF_MODULE.get_or_init(|| Mutex::new(None));
        let mut guard = module_cache.lock().unwrap();

        if guard.is_none() {
            let typf_module = Python::with_gil(|py| {
                // Import the typf Python module
                let typf_module = py.import_bound("typf").map_err(|e| {
                    TestypfError::RenderFailed(format!("Failed to import typf module: {}", e))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontListManager;
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MockPlatformManager {
        inner: Mutex<MockInner>,
    }

    #[derive(Default)]
    struct MockInner {
        installs: Vec<FontScope>,
        uninstalls: Vec<FontScope>,
        installed: HashSet<PathBuf>,
    }

    impl fontlift_core::FontManager for MockPlatformManager {
        fn install_font(
            &self,
            path: &std::path::Path,
            scope: FontScope,
        ) -> fontlift_core::FontResult<()> {
            let mut inner = self.inner.lock().unwrap();
            inner.installs.push(scope);
            inner.installed.insert(path.to_path_buf());
            Ok(())
        }

        fn uninstall_font(
            &self,
            path: &std::path::Path,
            scope: FontScope,
        ) -> fontlift_core::FontResult<()> {
            let mut inner = self.inner.lock().unwrap();
            inner.uninstalls.push(scope);
            inner.installed.remove(path);
            Ok(())
        }

        fn remove_font(
            &self,
            path: &std::path::Path,
            scope: FontScope,
        ) -> fontlift_core::FontResult<()> {
            self.uninstall_font(path, scope)?;
            std::fs::remove_file(path)?;
            Ok(())
        }

        fn is_font_installed(&self, path: &std::path::Path) -> fontlift_core::FontResult<bool> {
            let inner = self.inner.lock().unwrap();
            Ok(inner.installed.contains(path))
        }

        fn list_installed_fonts(&self) -> fontlift_core::FontResult<Vec<fontlift_core::FontInfo>> {
            Ok(Vec::new())
        }

        fn clear_font_caches(&self, _scope: FontScope) -> fontlift_core::FontResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_render_settings_default() {
        let settings = RenderSettings::default();
        assert_eq!(
            settings.sample_text,
            "The quick brown fox jumps over the lazy dog"
        );
        assert_eq!(settings.font_size, 16.0);
        assert_eq!(settings.foreground_color, (0, 0, 0, 255));
    }

    fn temp_font_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("testypf_fontlift_tests");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(name);
        let _ = std::fs::write(&path, b"dummy");
        path
    }

    fn sample_font_info(path: PathBuf) -> FontInfo {
        FontInfo {
            path,
            postscript_name: "DummyPS".to_string(),
            full_name: "Dummy Font".to_string(),
            family_name: "Dummy".to_string(),
            style: "Regular".to_string(),
            is_installed: false,
        }
    }

    #[test]
    fn install_defaults_to_user_scope() {
        let mock = Arc::new(MockPlatformManager::default());
        let mut manager = FontListManager::with_platform_override(mock.clone());
        let font = sample_font_info(temp_font_path("user_scope.ttf"));
        manager.push_font_for_tests(font.clone());

        manager.install_font(&font).expect("install");

        let inner = mock.inner.lock().unwrap();
        assert_eq!(inner.installs, vec![FontScope::User]);
        assert!(inner.installed.contains(&font.path));
    }

    #[test]
    fn install_scope_can_be_switched_to_system() {
        let mock = Arc::new(MockPlatformManager::default());
        let mut manager = FontListManager::with_platform_override(mock.clone());
        let font = sample_font_info(temp_font_path("system_scope.ttf"));
        manager.push_font_for_tests(font.clone());

        manager.set_install_scope(FontScope::System);
        manager.install_font(&font).expect("install");
        manager.uninstall_font(&font).expect("uninstall");

        let inner = mock.inner.lock().unwrap();
        assert_eq!(inner.installs, vec![FontScope::System]);
        assert_eq!(inner.uninstalls, vec![FontScope::System]);
    }
}
