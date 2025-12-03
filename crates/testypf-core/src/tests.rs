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
        source: &fontlift_core::FontliftFontSource,
    ) -> fontlift_core::FontResult<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.installs.push(source.scope.unwrap_or(FontScope::User));
        inner.installed.insert(source.path.clone());
        Ok(())
    }

    fn uninstall_font(
        &self,
        source: &fontlift_core::FontliftFontSource,
    ) -> fontlift_core::FontResult<()> {
        let mut inner = self.inner.lock().unwrap();
        inner
            .uninstalls
            .push(source.scope.unwrap_or(FontScope::User));
        inner.installed.remove(&source.path);
        Ok(())
    }

    fn remove_font(
        &self,
        source: &fontlift_core::FontliftFontSource,
    ) -> fontlift_core::FontResult<()> {
        self.uninstall_font(source)?;
        std::fs::remove_file(&source.path)?;
        Ok(())
    }

    fn is_font_installed(
        &self,
        source: &fontlift_core::FontliftFontSource,
    ) -> fontlift_core::FontResult<bool> {
        let inner = self.inner.lock().unwrap();
        Ok(inner.installed.contains(&source.path))
    }

    fn list_installed_fonts(
        &self,
    ) -> fontlift_core::FontResult<Vec<fontlift_core::FontliftFontFaceInfo>> {
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

fn sample_font_info(path: PathBuf) -> TestypfFontInfo {
    TestypfFontInfo {
        source: FontliftFontSource::new(path),
        postscript_name: "DummyPS".to_string(),
        full_name: "Dummy Font".to_string(),
        family_name: "Dummy".to_string(),
        style: "Regular".to_string(),
        is_installed: false,
        variation_axes: Vec::new(),
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
    assert!(inner.installed.contains(&font.source.path));
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

#[test]
fn discovery_manager_initializes_with_platform_roots() {
    let dm = DiscoveryManager::new();
    // On macOS, should have at least one default root
    #[cfg(target_os = "macos")]
    assert!(!dm.default_roots().is_empty());
}

#[test]
fn search_criteria_defaults_are_empty() {
    let criteria = SearchCriteria::default();
    assert!(criteria.name_pattern.is_none());
    assert!(criteria.features.is_empty());
    assert!(criteria.scripts.is_empty());
    assert!(criteria.axes.is_empty());
    assert!(!criteria.variable_only);
    assert!(!criteria.follow_symlinks);
}

#[test]
fn variation_axis_stores_tag_and_range() {
    let axis = TestypfVariationAxis {
        tag: "wght".to_string(),
        name: "Weight".to_string(),
        min_value: 100.0,
        default_value: 400.0,
        max_value: 900.0,
    };
    assert_eq!(axis.tag, "wght");
    assert_eq!(axis.min_value, 100.0);
    assert_eq!(axis.default_value, 400.0);
    assert_eq!(axis.max_value, 900.0);
}

#[test]
fn render_settings_variation_coords_default_to_empty() {
    let settings = RenderSettings::default();
    assert!(settings.variation_coords.is_empty());
}
