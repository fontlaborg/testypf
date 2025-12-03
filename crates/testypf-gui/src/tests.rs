//! Tests for testypf GUI application.

use crate::app::TestypfApp;
use crate::helpers;
use crate::message::Message;
use crate::types::{AppConfig, DropPathKind, LayoutMode, RenderPreview};

use iced::keyboard;
use iced::widget::image::Handle;
use iced::window;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use testypf_core::{
    FontliftFontSource, RenderResult, RenderSettings, RendererBackend, TestypfFontInfo,
};

#[test]
fn render_window_settings_are_transparent() {
    let settings = TestypfApp::render_window_settings();

    assert!(
        settings.transparent,
        "Render window should be transparent for overlay previews"
    );
    assert!(
        !settings.decorations,
        "Render window should be borderless for overlay mode"
    );
    assert_eq!(
        settings.level,
        window::Level::AlwaysOnTop,
        "Render window should stay above other windows"
    );
    assert_eq!(
        settings.size,
        iced::Size::new(900.0, 650.0),
        "Render window should default to 900x650 logical size"
    );
}

#[test]
fn image_handle_from_render_accepts_valid_rgba() {
    let render_result = RenderResult {
        width: 2,
        height: 2,
        data: vec![255; 16],
        format: "Rgba8".to_string(),
    };

    let handle =
        helpers::image_handle_from_render(&render_result).expect("Should build image handle");

    // Handle should be cloneable for display reuse
    let _clone = handle.clone();
}

#[test]
fn set_install_state_updates_flag() {
    let mut fonts = vec![TestypfFontInfo {
        source: FontliftFontSource::new(PathBuf::from("demo.ttf")),
        postscript_name: "DemoPS".into(),
        full_name: "Demo Font".into(),
        family_name: "Demo".into(),
        style: "Regular".into(),
        is_installed: false,
        variation_axes: Vec::new(),
    }];

    let updated = helpers::set_install_state(&mut fonts, 0, true);

    assert!(updated, "should report an update occurred");
    assert!(fonts[0].is_installed, "flag should flip to installed");
}

#[test]
fn set_install_state_out_of_bounds_is_noop() {
    let mut fonts = Vec::<TestypfFontInfo>::new();

    let updated = helpers::set_install_state(&mut fonts, 3, true);

    assert!(
        !updated,
        "out-of-bounds updates should be ignored without panicking"
    );
}

#[test]
fn image_handle_from_render_rejects_length_mismatch() {
    let render_result = RenderResult {
        width: 2,
        height: 2,
        data: vec![0; 12],
        format: "Rgba8".to_string(),
    };

    let err = helpers::image_handle_from_render(&render_result)
        .expect_err("Expected length mismatch error");

    assert!(err.contains("Pixel data length mismatch"));
}

#[test]
fn export_preview_writes_png() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let path = env::temp_dir().join(format!("testypf_preview_export_{ts}.png"));
    let pixels = vec![255u8; 16];

    let preview = RenderPreview {
        font_index: 0,
        width: 2,
        height: 2,
        format: "Rgba8".to_string(),
        pixels: pixels.clone(),
        handle: Handle::from_pixels(2, 2, pixels),
        duration_ms: 0,
    };

    helpers::export_preview_to_path(&preview, &path).expect("export should succeed");

    let bytes = fs::read(&path).expect("png should be written");
    assert!(
        bytes.starts_with(&[0x89, b'P', b'N', b'G']),
        "file should have PNG signature"
    );

    fs::remove_file(&path).ok();
}

#[test]
fn scan_folder_for_fonts_collects_stats() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let base = env::temp_dir().join(format!("testypf_scan_{ts}"));
    let nested = base.join("nested");

    fs::create_dir_all(&nested).expect("create test dirs");
    fs::write(base.join("a.ttf"), b"").expect("create font file");
    fs::write(nested.join("b.otf"), b"").expect("create nested font file");
    fs::write(base.join("ignore.txt"), b"").expect("create non-font file");

    let (fonts, stats) = helpers::scan_folder_for_fonts(&base).expect("scan should succeed");

    assert_eq!(fonts.len(), 2);
    assert_eq!(stats.fonts_found, 2);
    assert!(stats.directories_scanned >= 2); // base + nested
    assert!(stats.files_checked >= 3);
    assert!(stats.sample_files.iter().any(|name| name.contains("a")));

    fs::remove_dir_all(&base).ok();
}

#[test]
fn config_round_trip_to_custom_path() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let path = env::temp_dir().join(format!("testypf_cfg_{ts}.json"));

    let cfg = AppConfig {
        backend: RendererBackend::Orge,
    };

    helpers::save_config_to(&path, &cfg).expect("save config");
    let loaded = helpers::load_config_from(&path).expect("load config");

    assert_eq!(loaded, cfg);

    fs::remove_file(&path).ok();
}

#[test]
fn detects_font_extensions() {
    assert!(helpers::is_font_file(&PathBuf::from("font.TTF")));
    assert!(helpers::is_font_file(&PathBuf::from("font.woff2")));
    assert!(!helpers::is_font_file(&PathBuf::from("font.txt")));
}

#[test]
fn supports_additional_font_formats() {
    assert!(helpers::is_font_file(&PathBuf::from("demo.dfont")));
    assert!(helpers::is_font_file(&PathBuf::from("demo.eot")));
    assert!(helpers::is_font_file(&PathBuf::from("demo.svg")));
    assert!(helpers::is_font_file(&PathBuf::from("demo.pfa")));
    assert!(helpers::is_font_file(&PathBuf::from("demo.pfb")));
}

#[test]
fn aggregates_extension_stats() {
    let paths = vec![
        PathBuf::from("a.ttf"),
        PathBuf::from("b.otf"),
        PathBuf::from("c.OTF"),
        PathBuf::from("d.woff2"),
        PathBuf::from("e.woff"),
        PathBuf::from("ignored.txt"),
    ];

    let stats = helpers::extension_stats(&paths);
    assert_eq!(stats.get("ttf"), Some(&1));
    assert_eq!(stats.get("otf"), Some(&2));
    assert_eq!(stats.get("woff2"), Some(&1));
    assert_eq!(stats.get("woff"), Some(&1));
    assert!(!stats.contains_key("txt"));
}

#[test]
fn parses_hex_colors() {
    assert_eq!(
        helpers::parse_rgba_hex("#112233"),
        Some((0x11, 0x22, 0x33, 0xFF))
    );
    assert_eq!(
        helpers::parse_rgba_hex("44556677"),
        Some((0x44, 0x55, 0x66, 0x77))
    );
    assert_eq!(helpers::parse_rgba_hex("12"), None);
    assert_eq!(helpers::parse_rgba_hex("GGHHII"), None);
}

#[test]
fn cache_hit_only_when_settings_and_fonts_match() {
    let settings = RenderSettings::default();
    let other_settings = RenderSettings {
        font_size: settings.font_size + 1.0,
        ..RenderSettings::default()
    };
    let fonts = vec![PathBuf::from("a.ttf"), PathBuf::from("b.otf")];
    let other_fonts = vec![PathBuf::from("a.ttf")];

    assert!(
        !helpers::should_use_cache(&None, &[], &settings, &fonts, true),
        "No previous settings means no cache hit"
    );
    assert!(
        !helpers::should_use_cache(&Some(settings.clone()), &fonts, &settings, &fonts, false),
        "No previews means no cache hit"
    );
    assert!(
        helpers::should_use_cache(&Some(settings.clone()), &fonts, &settings, &fonts, true),
        "Matching settings and fonts with previews should hit cache"
    );
    assert!(
        !helpers::should_use_cache(&Some(other_settings), &fonts, &settings, &fonts, true),
        "Changed settings invalidates cache"
    );
    assert!(
        !helpers::should_use_cache(
            &Some(settings.clone()),
            &other_fonts,
            &settings,
            &fonts,
            true
        ),
        "Changed fonts invalidate cache"
    );
}

#[test]
fn classify_drop_path_flags_invalid_inputs() {
    let base = env::temp_dir().join("testypf_classify_drop");
    let nested_dir = base.join("fonts");

    fs::create_dir_all(&nested_dir).expect("create test dir");
    let font_path = nested_dir.join("demo.ttf");
    let text_path = nested_dir.join("readme.md");
    fs::write(&font_path, b"").expect("create font file");
    fs::write(&text_path, b"").expect("create txt file");

    assert_eq!(
        helpers::classify_drop_path(&font_path),
        DropPathKind::FontFile
    );
    assert_eq!(
        helpers::classify_drop_path(&nested_dir),
        DropPathKind::Directory
    );
    assert_eq!(
        helpers::classify_drop_path(&text_path),
        DropPathKind::Unsupported
    );
    let missing = nested_dir.join("missing.otf");
    assert_eq!(helpers::classify_drop_path(&missing), DropPathKind::Missing);

    fs::remove_dir_all(&base).ok();
}

#[test]
fn layout_row_count_matches_layout_mode() {
    assert_eq!(helpers::layout_row_count(0, LayoutMode::Single), 0);
    assert_eq!(helpers::layout_row_count(3, LayoutMode::Single), 3);
    assert_eq!(helpers::layout_row_count(0, LayoutMode::SideBySide), 0);
    assert_eq!(helpers::layout_row_count(1, LayoutMode::SideBySide), 1);
    assert_eq!(helpers::layout_row_count(2, LayoutMode::SideBySide), 1);
    assert_eq!(helpers::layout_row_count(3, LayoutMode::SideBySide), 2);
    assert_eq!(helpers::layout_row_count(4, LayoutMode::SideBySide), 2);
}

#[test]
fn format_file_size_scales_units() {
    assert_eq!(helpers::format_file_size(0), "0 B");
    assert_eq!(helpers::format_file_size(532), "532 B");
    assert_eq!(helpers::format_file_size(1536), "1.5 KB");
    assert_eq!(helpers::format_file_size(5_242_880), "5.0 MB");
}

#[test]
fn preview_metadata_text_includes_duration_and_backend() {
    let font = TestypfFontInfo {
        source: FontliftFontSource::new(PathBuf::from("demo.ttf")),
        postscript_name: "DemoPS".into(),
        full_name: "Demo Font".into(),
        family_name: "Demo".into(),
        style: "Regular".into(),
        is_installed: true,
        variation_axes: Vec::new(),
    };

    let preview = RenderPreview {
        font_index: 0,
        width: 32,
        height: 16,
        format: "Rgba8".to_string(),
        pixels: vec![255; 32 * 16 * 4],
        handle: Handle::from_pixels(32, 16, vec![255; 32 * 16 * 4]),
        duration_ms: 12,
    };

    let text = helpers::preview_metadata_text(&preview, &font, &RenderSettings::default());

    assert!(text.contains("12 ms"), "render duration should be shown");
    assert!(
        text.contains(&RenderSettings::default().backend.to_string()),
        "backend label should be present"
    );
}

#[test]
fn font_metadata_lines_include_path_and_install_state() {
    let font = TestypfFontInfo {
        source: FontliftFontSource::new(PathBuf::from("/tmp/metadata/demo.ttf")),
        postscript_name: "DemoPS".into(),
        full_name: "Demo Font".into(),
        family_name: "Demo".into(),
        style: "Bold".into(),
        is_installed: false,
        variation_axes: Vec::new(),
    };

    let lines = helpers::font_metadata_lines(&font, Some(2048));

    let combined = lines.join("\n");
    assert!(combined.contains("Demo Font"), "full name should appear");
    assert!(combined.contains("Bold"), "style should appear");
    assert!(
        combined.contains("Not installed"),
        "install status should be shown"
    );
    assert!(
        combined.contains("2.0 KB"),
        "formatted file size should be included"
    );
    assert!(
        combined.contains("/tmp/metadata/demo.ttf"),
        "path should be present"
    );
}

#[test]
fn render_targets_respect_selection_toggle() {
    let visible = vec![0, 1, 2];

    assert_eq!(
        helpers::derive_render_targets(Some(2), &visible, true),
        vec![2]
    );
    assert!(
        helpers::derive_render_targets(None, &visible, true).is_empty(),
        "selection required when rendering selected only"
    );
    assert_eq!(
        helpers::derive_render_targets(None, &visible, false),
        visible
    );
}

#[test]
fn shortcut_mapping_covers_core_actions() {
    let command = keyboard::Modifiers::COMMAND;

    let render = keyboard::Event::KeyPressed {
        key: keyboard::Key::Character("r".into()),
        location: keyboard::Location::Standard,
        modifiers: command,
        text: None,
    };
    assert!(matches!(
        helpers::shortcut_to_message(&render),
        Some(Message::RenderPreviews)
    ));

    let export = keyboard::Event::KeyPressed {
        key: keyboard::Key::Character("e".into()),
        location: keyboard::Location::Standard,
        modifiers: command,
        text: None,
    };
    assert!(matches!(
        helpers::shortcut_to_message(&export),
        Some(Message::ExportPreviews)
    ));

    let ignore = keyboard::Event::KeyPressed {
        key: keyboard::Key::Character("r".into()),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: None,
    };
    assert!(
        helpers::shortcut_to_message(&ignore).is_none(),
        "shortcuts should require command modifier"
    );
}
