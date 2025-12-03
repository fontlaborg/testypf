//! Message handling for testypf GUI application.

use crate::app::TestypfApp;
use crate::helpers;
use crate::message::Message;
use crate::types::{AppConfig, DropPathKind, ScanStats};

use iced::{window, Command};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use testypf_core::FontliftFontSource;

/// Handle an incoming message and return any resulting command.
pub fn handle_message(app: &mut TestypfApp, message: Message) -> Command<Message> {
    match message {
        Message::SampleTextChanged(text) => {
            app.render_settings.sample_text = text;
            app.status = "Sample text updated".to_string();
            app.invalidate_render_cache();
        }

        Message::FontSizeChanged(size_str) => {
            if let Ok(size) = size_str.parse::<f32>() {
                app.render_settings.font_size = size;
                app.status = "Font size updated".to_string();
                app.invalidate_render_cache();
            } else {
                app.status = "Invalid font size".to_string();
            }
        }

        Message::BackendChanged(backend) => {
            let _ = app.engine.text_renderer().set_backend(backend.clone());
            app.render_settings.backend = backend.clone();
            if let Err(e) = helpers::save_config(&AppConfig {
                backend: backend.clone(),
            }) {
                eprintln!("Failed to persist backend selection: {}", e);
            }
            app.status = format!("Backend changed to {}", backend);
            app.invalidate_render_cache();
        }

        Message::ForegroundChanged(value) => {
            app.foreground_input = value.clone();
            match helpers::parse_rgba_hex(&value) {
                Some(color) => {
                    app.render_settings.foreground_color = color;
                    app.status = "Foreground color updated".to_string();
                    app.invalidate_render_cache();
                }
                None => {
                    app.status = "Foreground color must be #RRGGBB or #RRGGBBAA".to_string();
                }
            }
        }

        Message::BackgroundChanged(value) => {
            app.background_input = value.clone();
            if !app.background_enabled {
                return Command::none();
            }

            match helpers::parse_rgba_hex(&value) {
                Some(color) => {
                    app.render_settings.background_color = Some(color);
                    app.status = "Background color updated".to_string();
                    app.invalidate_render_cache();
                }
                None => {
                    app.status = "Background color must be #RRGGBB or #RRGGBBAA".to_string();
                }
            }
        }

        Message::BackgroundToggled(enabled) => {
            app.background_enabled = enabled;
            if enabled {
                if let Some(color) = helpers::parse_rgba_hex(&app.background_input) {
                    app.render_settings.background_color = Some(color);
                    app.status = "Background enabled".to_string();
                    app.invalidate_render_cache();
                } else {
                    app.render_settings.background_color = Some((0, 0, 0, 0));
                    app.status = "Background enabled with default transparent color".to_string();
                    app.invalidate_render_cache();
                }
            } else {
                app.render_settings.background_color = None;
                app.status = "Background disabled (transparent)".to_string();
                app.invalidate_render_cache();
            }
        }

        Message::LayoutChanged(mode) => {
            if app.layout_mode != mode {
                app.layout_mode = mode;
                app.status = format!("Layout changed to {}", app.layout_mode);
            }
        }

        Message::RenderSelectedOnlyToggled(enabled) => {
            app.render_selected_only = enabled;
            if enabled && app.selected_font.is_none() {
                app.status = "Select a font to render when 'selected only' is enabled".to_string();
            } else if enabled {
                app.status = "Rendering limited to the selected font".to_string();
            } else {
                app.status = "Rendering all visible fonts".to_string();
            }
        }

        Message::InstallScopeChanged(scope) => {
            app.install_scope = scope;
            app.engine
                .font_manager()
                .set_install_scope(scope.to_font_scope());
            app.status = format!("Install scope set to {}", scope);
        }

        Message::TestBackend => {
            if let Some(font) = app.fonts.first().cloned() {
                let mut settings = app.render_settings.clone();
                settings.sample_text = "Backend self-test".to_string();
                settings.font_size = 18.0;
                let started = Instant::now();

                match app
                    .engine
                    .text_renderer()
                    .render_text(font.path(), &settings)
                {
                    Ok(_) => {
                        let elapsed = started.elapsed().as_millis();
                        app.status = format!(
                            "Backend {} OK in {} ms using {}",
                            settings.backend, elapsed, font.full_name
                        );
                    }
                    Err(e) => {
                        app.status = format!("Backend test failed: {}", e);
                    }
                }
            } else {
                app.status = "Load a font before testing the backend".to_string();
            }
        }

        Message::AddFonts => {
            app.status = "Opening file dialog...".to_string();
            return Command::perform(
                async {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    rfd::FileDialog::new()
                        .add_filter("Font Files", &["ttf", "otf", "ttc", "otc", "woff", "woff2"])
                        .pick_files()
                },
                Message::FontsSelected,
            );
        }

        Message::FontsSelected(paths) => match paths {
            Some(paths) => process_dropped_paths(app, paths),
            None => {
                app.status = "No fonts selected".to_string();
            }
        },

        Message::FilesDropped(paths) => {
            app.is_dragging = false;
            app.hovered_file = None;
            app.pending_drop_paths.extend(paths);
            return enqueue_drop_processing(app);
        }

        Message::ProcessPendingDrops => {
            app.drop_processing_scheduled = false;
            let pending = std::mem::take(&mut app.pending_drop_paths);
            if pending.is_empty() {
                return Command::none();
            }
            process_dropped_paths(app, pending);
        }

        Message::DragEnter => {
            app.is_dragging = true;
            app.status = "Drag files or folders here...".to_string();
        }

        Message::DragLeave => {
            app.is_dragging = false;
            app.hovered_file = None;
            app.status = if app.fonts.is_empty() {
                "No fonts loaded. Add fonts to get started.".to_string()
            } else {
                format!("Loaded {} font(s)", app.fonts.len())
            };
        }

        Message::FileHovered(path) => {
            app.is_dragging = true;
            app.hovered_file = Some(path.clone());
            if let Some(ref file) = app.hovered_file {
                app.status = format!(
                    "Drop: {} (Ready to add this font)",
                    file.file_name().unwrap_or_default().to_string_lossy()
                );
            }
        }

        Message::RemoveFont(index) => {
            if index < app.fonts.len() {
                let font = &app.fonts[index];
                let _ = app.engine.font_manager().remove_font(&font.source);
                app.fonts.remove(index);
                if app.selected_font == Some(index)
                    || app.selected_font.map(|i| i > index).unwrap_or(false)
                {
                    app.selected_font = None;
                    app.render_settings.variation_coords.clear();
                }
                app.status = "Font removed".to_string();
                app.invalidate_render_cache();
                app.render_previews.clear();
            }
        }

        Message::SelectFont(index) => {
            if index < app.fonts.len() {
                if app.selected_font == Some(index) {
                    app.selected_font = None;
                    app.render_settings.variation_coords.clear();
                    app.status = "Font details hidden; variations reset".to_string();
                    app.invalidate_render_cache();
                } else {
                    app.selected_font = Some(index);
                    helpers::sync_variations_for_axes(
                        &mut app.render_settings,
                        &app.fonts[index].variation_axes,
                    );
                    app.invalidate_render_cache();
                    app.status = format!("Showing details for {}", app.fonts[index].full_name);
                }
            }
        }

        Message::VariationAxisChanged(tag, value) => {
            if let Some(selected) = app.selected_font {
                if let Some(font) = app.fonts.get(selected) {
                    if let Some(axis) = font.variation_axes.iter().find(|a| a.tag == tag) {
                        let clamped = helpers::clamp_variation_value(value, axis);
                        app.render_settings
                            .variation_coords
                            .insert(tag.clone(), clamped);
                        app.status = format!("{} set to {:.1}", axis.name, clamped);
                        app.invalidate_render_cache();
                    } else {
                        app.status = format!("Axis {} not found for selected font", tag);
                    }
                }
            } else {
                app.status = "Select a variable font to adjust axes".to_string();
            }
        }

        Message::FontFilterChanged(filter) => {
            app.font_filter = filter;
            app.status = format!(
                "Filtered fonts: showing {} of {}",
                app.visible_font_indices().len(),
                app.fonts.len()
            );
        }

        Message::InstallFont(index) => {
            if !app.font_ops_available {
                app.status = "Font install unavailable in this build; enable platform-mac or platform-win features.".to_string();
                return Command::none();
            }

            if index < app.fonts.len() {
                let font_full_name = app.fonts[index].full_name.clone();
                let font = app.fonts[index].clone();
                match app.engine.font_manager().install_font(&font) {
                    Ok(()) => {
                        app.refresh_install_status(index);
                        app.status = format!(
                            "Font '{}' installed to {}",
                            font_full_name,
                            app.install_scope.description()
                        );
                    }
                    Err(e) => {
                        app.status = helpers::friendly_font_op_error(
                            "Install",
                            app.install_scope,
                            &e.to_string(),
                        );
                    }
                }
            }
        }

        Message::UninstallFont(index) => {
            if !app.font_ops_available {
                app.status = "Font uninstall unavailable in this build; enable platform-mac or platform-win features.".to_string();
                return Command::none();
            }

            if index < app.fonts.len() {
                let font_full_name = app.fonts[index].full_name.clone();
                let font = app.fonts[index].clone();
                match app.engine.font_manager().uninstall_font(&font) {
                    Ok(()) => {
                        app.refresh_install_status(index);
                        app.status = format!(
                            "Font '{}' uninstalled from {}",
                            font_full_name,
                            app.install_scope.description()
                        );
                    }
                    Err(e) => {
                        app.status = helpers::friendly_font_op_error(
                            "Uninstall",
                            app.install_scope,
                            &e.to_string(),
                        );
                    }
                }
            }
        }

        Message::RenderPreviews => {
            return handle_render_previews(app);
        }

        Message::OpenRenderWindow => {
            let mut cmds = vec![app.ensure_render_window()];
            if let Some(id) = app.render_window_id {
                cmds.push(window::gain_focus(id));
            }
            return Command::batch(cmds);
        }

        Message::ExportPreviews => {
            if app.render_previews.is_empty() {
                app.status = "Render previews before exporting them".to_string();
                return Command::none();
            }

            app.status = "Choose a folder to save PNG previews...".to_string();
            return Command::perform(
                async { rfd::FileDialog::new().pick_folder() },
                Message::ExportDestinationChosen,
            );
        }

        Message::ExportDestinationChosen(destination) => match destination {
            Some(folder) => match export_previews_to_folder(app, &folder) {
                Ok(written) => {
                    app.status = format!("Exported {} preview(s) to {}", written, folder.display());
                }
                Err(e) => {
                    app.status = format!("Export failed: {}", e);
                }
            },
            None => {
                app.status = "Export cancelled".to_string();
            }
        },

        Message::WindowClosed(id) => {
            if Some(id) == app.render_window_id {
                app.render_window_id = None;
                app.status = "Render window closed".to_string();
            }
        }

        Message::StatusUpdate(msg) => {
            app.status = msg;
        }

        Message::None => {}
    }

    Command::none()
}

/// Handle the RenderPreviews message.
fn handle_render_previews(app: &mut TestypfApp) -> Command<Message> {
    if app.fonts.is_empty() {
        app.status = "No fonts to render".to_string();
        return Command::none();
    }

    let visible_indices = app.visible_font_indices();
    let target_indices = helpers::derive_render_targets(
        app.selected_font,
        &visible_indices,
        app.render_selected_only,
    );

    if target_indices.is_empty() {
        app.status = if app.render_selected_only {
            "Select a font to render when 'selected only' is enabled".to_string()
        } else {
            "No fonts match the current filter".to_string()
        };
        return Command::none();
    }

    let font_paths: Vec<PathBuf> = target_indices
        .iter()
        .filter_map(|&i| app.fonts.get(i).map(|f| f.path().to_path_buf()))
        .collect();

    if font_paths.is_empty() {
        app.status = "No fonts available to render".to_string();
        return Command::none();
    }

    if app.render_cache_hit(&font_paths) {
        let mut cmds = Vec::new();
        cmds.push(app.ensure_render_window());
        if let Some(id) = app.render_window_id {
            cmds.push(window::gain_focus(id));
        }
        app.status = "Render settings unchanged - using cached previews".to_string();
        return Command::batch(cmds);
    }

    app.status = format!(
        "Rendering {} of {} font(s)...",
        font_paths.len(),
        app.fonts.len()
    );

    // Clear previous results
    app.render_previews.clear();

    // Render each selected font
    let render_start = Instant::now();
    let mut previews = Vec::new();
    for font_index in target_indices {
        if let Some(font) = app.fonts.get(font_index) {
            let per_start = Instant::now();
            match app
                .engine
                .text_renderer()
                .render_text(font.path(), &app.render_settings)
            {
                Ok(render_result) => {
                    let duration_ms = per_start.elapsed().as_millis();
                    match helpers::build_render_preview(font_index, render_result, duration_ms) {
                        Ok(preview) => previews.push(preview),
                        Err(e) => {
                            app.status = format!(
                                "Failed to create preview for font {}: {}",
                                font.full_name, e
                            );
                            return Command::none();
                        }
                    }
                }
                Err(e) => {
                    app.status = helpers::friendly_render_error(&font.full_name, &e.to_string());
                    return Command::none();
                }
            }
        }
    }

    app.render_previews = previews;
    app.last_render_settings = Some(app.render_settings.clone());
    app.last_render_font_paths = font_paths;
    app.status = format!(
        "Rendering complete - {} preview(s) generated in {} ms",
        app.render_previews.len(),
        render_start.elapsed().as_millis()
    );

    let mut cmds = Vec::new();
    cmds.push(app.ensure_render_window());
    if let Some(id) = app.render_window_id {
        cmds.push(window::gain_focus(id));
    }

    Command::batch(cmds)
}

/// Enqueue drop processing with a small delay.
fn enqueue_drop_processing(app: &mut TestypfApp) -> Command<Message> {
    if app.drop_processing_scheduled {
        return Command::none();
    }
    app.drop_processing_scheduled = true;
    Command::perform(
        async {
            std::thread::sleep(Duration::from_millis(60));
        },
        |_| Message::ProcessPendingDrops,
    )
}

/// Process dropped paths (files and folders).
fn process_dropped_paths(app: &mut TestypfApp, paths: Vec<PathBuf>) {
    let mut added_count = 0;
    let mut font_paths = Vec::new();
    let mut aggregated_stats = ScanStats::default();
    let mut invalid_paths: Vec<(PathBuf, DropPathKind)> = Vec::new();

    for path in paths {
        match helpers::classify_drop_path(&path) {
            DropPathKind::FontFile => {
                aggregated_stats.files_checked += 1;
                aggregated_stats.fonts_found += 1;
                if aggregated_stats.sample_files.len() < 3 {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        aggregated_stats.sample_files.push(name.to_string());
                    }
                }
                font_paths.push(path);
            }
            DropPathKind::Directory => {
                app.status = format!("Scanning folder for fonts: {:?}", path);
                match helpers::scan_folder_for_fonts(&path) {
                    Ok((mut folder_fonts, stats)) => {
                        aggregated_stats.directories_scanned += stats.directories_scanned;
                        aggregated_stats.files_checked += stats.files_checked;
                        aggregated_stats.fonts_found += stats.fonts_found;
                        for name in stats.sample_files {
                            if aggregated_stats.sample_files.len() < 3 {
                                aggregated_stats.sample_files.push(name);
                            }
                        }
                        font_paths.append(&mut folder_fonts);
                    }
                    Err(e) => {
                        app.status = format!("Failed to scan folder {:?}: {}", path, e);
                        return;
                    }
                }
            }
            DropPathKind::Unsupported => {
                aggregated_stats.files_checked += 1;
                invalid_paths.push((path, DropPathKind::Unsupported));
                continue;
            }
            DropPathKind::Missing => {
                invalid_paths.push((path, DropPathKind::Missing));
                continue;
            }
        }
    }

    let ext_stats = helpers::extension_stats(&font_paths);

    // Add all discovered font files
    for font_path in font_paths {
        let format = font_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());
        let source = FontliftFontSource::new(font_path.clone()).with_format(format);

        match app.engine.font_manager().add_font(&source) {
            Ok(font_info) => {
                app.fonts.push(font_info);
                added_count += 1;
            }
            Err(e) => {
                app.status = format!("Failed to load font {:?}: {}", font_path, e);
                return;
            }
        }
    }

    if added_count > 0 {
        app.status = format!("Dropped and added {} font(s)", added_count);
    } else {
        app.status = "No valid font files found".to_string();
    }

    if added_count > 0 {
        app.invalidate_render_cache();
        app.render_previews.clear();
    }

    if !invalid_paths.is_empty() {
        let unsupported = invalid_paths
            .iter()
            .filter(|(_, kind)| matches!(kind, DropPathKind::Unsupported))
            .count();
        let missing = invalid_paths.len() - unsupported;
        let details = invalid_paths
            .iter()
            .take(2)
            .map(|(p, kind)| {
                format!(
                    "{}: {}",
                    match kind {
                        DropPathKind::Unsupported => "unsupported",
                        DropPathKind::Missing => "missing",
                        _ => "skipped",
                    },
                    p.display()
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let suffix = if invalid_paths.len() > 2 { "..." } else { "" };
        app.status = format!(
            "{} | Skipped {} unsupported and {} missing item(s) ({}){}",
            app.status, unsupported, missing, details, suffix
        );
    }

    if aggregated_stats.files_checked > 0 || aggregated_stats.directories_scanned > 0 {
        app.last_scan_stats = Some(aggregated_stats.clone());
        let sample_preview = if !aggregated_stats.sample_files.is_empty() {
            format!(" e.g. {}", aggregated_stats.sample_files.join(", "))
        } else {
            String::new()
        };

        let ext_summary = helpers::format_extension_summary(&ext_stats);

        app.status = format!(
            "Scanned {} folder(s), checked {} file(s), found {} font(s){}{}",
            aggregated_stats.directories_scanned,
            aggregated_stats.files_checked,
            aggregated_stats.fonts_found,
            sample_preview,
            ext_summary
        );
    }
}

/// Export all current render previews into the given folder.
fn export_previews_to_folder(app: &TestypfApp, folder: &std::path::Path) -> Result<usize, String> {
    std::fs::create_dir_all(folder)
        .map_err(|e| format!("Failed to create export folder {:?}: {}", folder, e))?;

    let mut written = 0usize;

    for (i, preview) in app.render_previews.iter().enumerate() {
        let stem = app
            .fonts
            .get(preview.font_index)
            .map(|f| helpers::sanitized_file_stem(&f.full_name))
            .unwrap_or_else(|| format!("font{}", i + 1));

        let mut candidate = folder.join(format!("{:02}_{}.png", i + 1, stem));
        let mut suffix = 1;
        while candidate.exists() {
            candidate = folder.join(format!("{:02}_{}_{suffix}.png", i + 1, stem));
            suffix += 1;
        }

        helpers::export_preview_to_path(preview, &candidate)?;
        written += 1;
    }

    Ok(written)
}
