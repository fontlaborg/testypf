//! Main window view for testypf GUI application.

use crate::app::TestypfApp;
use crate::helpers;
use crate::message::Message;
use crate::styles::DragActiveStyle;
use crate::types::{InstallScope, LayoutMode, RenderAvailability};

use iced::widget::{
    button, checkbox, column, container, image as iced_image, pick_list, row, scrollable, text,
    text_input,
};
use iced::{Element, Length};

/// Render the main window view.
pub fn render(app: &TestypfApp) -> Element<'_, Message> {
    let title = text("Testypf - Typf GUI Tester")
        .size(24)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(
            0.2, 0.2, 0.8,
        )));

    let status = text(&app.status).size(14);
    let visible_indices = app.visible_font_indices();
    let render_state = RenderAvailability::derive(
        app.fonts.len(),
        visible_indices.len(),
        app.render_selected_only,
        app.selected_font,
    );

    let quickstart: Option<Element<Message>> = if app.fonts.is_empty() {
        Some(
            container(
                column![
                    text("Quick start").size(16),
                    text("1) Add fonts with the button below or drop files/folders anywhere on this window.")
                        .size(12),
                    text("2) Pick install scope if you plan to install (User is safest).")
                        .size(12),
                    text("3) Click Render Previews to see output; previews open in the overlay window.")
                        .size(12),
                    text("Supports .ttf, .otf, .ttc/.otc, .woff/.woff2, .dfont, .pfa/.pfb, .eot, .svg")
                        .size(10)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.45, 0.45, 0.45))),
                ]
                .spacing(6),
            )
            .padding(12)
            .style(iced::theme::Container::Box)
            .into(),
        )
    } else {
        None
    };

    // Font list section
    let font_list_header = text("Font List").size(18);

    let font_filter = text_input("Filter fonts (name, family, style, path)", &app.font_filter)
        .on_input(Message::FontFilterChanged)
        .size(12);
    let filter_hint = text(format!(
        "Showing {} of {}",
        visible_indices.len(),
        app.fonts.len()
    ))
    .size(10)
    .style(iced::theme::Text::Color(iced::Color::from_rgb(
        0.45, 0.45, 0.45,
    )));

    let install_scope_picker = row![
        text("Install scope").size(12),
        pick_list(
            InstallScope::OPTIONS.to_vec(),
            Some(app.install_scope),
            Message::InstallScopeChanged
        ),
        text(app.install_scope.description())
            .size(10)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.45, 0.45, 0.45
            )))
    ]
    .spacing(10)
    .align_items(iced::Alignment::Center);

    let font_list: Element<Message> = font_list_view(app, &visible_indices);

    let font_ops_notice: Option<Element<Message>> = if !app.font_ops_available {
        Some(
            text("Font install/uninstall disabled: rebuild with platform-mac or platform-win features to enable system font management.")
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.6, 0.2, 0.2,
                )))
                .into(),
        )
    } else {
        None
    };

    let metadata_panel: Element<Message> = metadata_panel_view(app);

    let drop_area = drop_area_view(app);

    // Render controls section
    let render_controls = render_controls_view(app, render_state);

    // Preview section
    let preview_area: Element<Message> = preview_area_view(app, &visible_indices, render_state);

    // Layout everything
    let content = column![
        title,
        container(status).padding(10),
        quickstart.unwrap_or_else(|| text("").into()),
        // Font list section
        font_list_header,
        row![font_filter, filter_hint]
            .spacing(10)
            .align_items(iced::Alignment::Center),
        install_scope_picker,
        font_list,
        font_ops_notice.unwrap_or_else(|| text("").into()),
        metadata_panel,
        drop_area,
        // Render controls and preview
        render_controls,
        preview_area,
    ]
    .spacing(20)
    .padding(20)
    .width(iced::Length::Fill);

    container(content)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center_x()
        .into()
}

/// Build the font list view.
fn font_list_view<'a>(app: &'a TestypfApp, visible_indices: &[usize]) -> Element<'a, Message> {
    if app.fonts.is_empty() {
        text("No fonts loaded")
            .size(14)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.5, 0.5, 0.5,
            )))
            .into()
    } else if visible_indices.is_empty() {
        text("No fonts match the current filter")
            .size(14)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.5, 0.3, 0.3,
            )))
            .into()
    } else {
        column(
            visible_indices
                .iter()
                .filter_map(|&i| app.fonts.get(i).map(|font| (i, font)))
                .map(|(i, font)| {
                    let font_info = text(format!("{} ({})", font.full_name, font.family_name)).size(14);

                    let install_status = if font.is_installed {
                        text("Installed")
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.5, 0.0)))
                    } else {
                        text("Not installed")
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.2, 0.2)))
                    };

                    let mut details_btn = button("Details");
                    if app.selected_font == Some(i) {
                        details_btn = details_btn.style(iced::theme::Button::Primary);
                    }
                    details_btn = details_btn.on_press(Message::SelectFont(i));

                    let remove_btn = button("Remove").on_press(Message::RemoveFont(i));

                    let mut install_btn = button("Install");
                    if !app.font_ops_available {
                        install_btn = install_btn.style(iced::theme::Button::Secondary);
                    } else if !font.is_installed {
                        install_btn = install_btn.on_press(Message::InstallFont(i));
                    }

                    let mut uninstall_btn = button("Uninstall");
                    if !app.font_ops_available {
                        uninstall_btn = uninstall_btn.style(iced::theme::Button::Secondary);
                    } else if font.is_installed {
                        uninstall_btn = uninstall_btn.on_press(Message::UninstallFont(i));
                    }

                    row![
                        font_info,
                        install_status,
                        details_btn,
                        remove_btn,
                        install_btn,
                        uninstall_btn
                    ]
                    .spacing(10)
                    .align_items(iced::Alignment::Center)
                    .into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(5)
        .into()
    }
}

/// Build the metadata panel view.
fn metadata_panel_view(app: &TestypfApp) -> Element<'_, Message> {
    if let Some(selected) = app.selected_font.and_then(|i| app.fonts.get(i)) {
        let file_size = helpers::font_file_size(&selected.path);
        let lines = helpers::font_metadata_lines(selected, file_size);
        let rows = lines
            .iter()
            .map(|line| text(line).size(12).into())
            .collect::<Vec<_>>();

        container(column![text("Font Metadata").size(16), column(rows).spacing(4),].spacing(8))
            .padding(12)
            .width(Length::Fill)
            .style(iced::theme::Container::Box)
            .into()
    } else {
        container(
            column![
                text("Font Metadata").size(16),
                text("Select a font to view its details.")
                    .size(12)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5,))),
            ]
            .spacing(6),
        )
        .padding(12)
        .width(Length::Fill)
        .style(iced::theme::Container::Box)
        .into()
    }
}

/// Build the drop area view.
fn drop_area_view(app: &TestypfApp) -> Element<'_, Message> {
    let scan_summary = app.last_scan_stats.as_ref().map(|stats| {
        let sample_preview = if !stats.sample_files.is_empty() {
            format!(" e.g. {}", stats.sample_files.join(", "))
        } else {
            String::new()
        };

        format!(
            "Last scan: {} font(s) from {} file(s) across {} folder(s){}",
            stats.fonts_found, stats.files_checked, stats.directories_scanned, sample_preview
        )
    });

    let drop_area_content = if app.is_dragging {
        let hover_info = if let Some(ref hovered_file) = app.hovered_file {
            let file_name = hovered_file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            if hovered_file.is_dir() {
                text(format!(
                    "📂 Folder: {} (Click to scan recursively)",
                    file_name
                ))
                .size(14)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.4, 0.8)))
            } else {
                text(format!("📄 File: {} (Click to add this font)", file_name))
                    .size(14)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.6, 0.0)))
            }
        } else {
            text("🎯 Drop fonts or folders here!")
                .size(18)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.6, 0.0)))
        };

        column![
            hover_info,
            text(format!("Supports {}", helpers::supported_formats_text()))
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.3, 0.3, 0.3))),
            text("✨ Recursive folder scanning with progress feedback")
                .size(10)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.4, 0.4, 0.4))),
            text("💡 Tip: You can drop multiple files and folders at once!")
                .size(9)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
            scan_summary
                .as_ref()
                .map(|summary| {
                    text(summary)
                        .size(10)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.3, 0.5, 0.3)))
                })
                .unwrap_or_else(|| text("").into()),
        ]
        .spacing(6)
        .align_items(iced::Alignment::Center)
    } else {
        let status_text = if app.fonts.is_empty() {
            text("📁 Drag & drop fonts to get started")
                .size(16)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.4, 0.0)))
        } else {
            text(format!(
                "📁 Drag & drop more fonts ({} loaded)",
                app.fonts.len()
            ))
            .size(16)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.2, 0.8)))
        };

        column![
            status_text,
            text(format!(
                "Supports {} via files or folders (recursive)",
                helpers::supported_formats_text()
            ))
            .size(12)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
            button("Add Fonts...")
                .on_press(Message::AddFonts)
                .style(iced::theme::Button::Secondary),
            scan_summary
                .as_ref()
                .map(|summary| {
                    text(summary)
                        .size(10)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.3, 0.5, 0.3)))
                })
                .unwrap_or_else(|| text("").into()),
        ]
        .spacing(12)
        .align_items(iced::Alignment::Center)
    };

    let drop_area_style = if app.is_dragging {
        iced::theme::Container::Custom(Box::new(DragActiveStyle))
    } else {
        iced::theme::Container::Box
    };

    container(drop_area_content)
        .padding(20)
        .width(iced::Length::Fill)
        .height(iced::Length::Fixed(140.0))
        .center_x()
        .center_y()
        .style(drop_area_style)
        .into()
}

/// Build the render controls view.
fn render_controls_view(app: &TestypfApp, render_state: RenderAvailability) -> Element<'_, Message> {
    let render_header = text("Render Controls").size(18);

    let sample_text_input = text_input("Enter sample text...", &app.render_settings.sample_text)
        .on_input(Message::SampleTextChanged)
        .size(14);

    let font_size_input = text_input("Font size", &app.render_settings.font_size.to_string())
        .on_input(Message::FontSizeChanged)
        .size(14);

    // Backend selector
    let backend_options = app.get_available_backends();
    let backend_descriptions = backend_options
        .iter()
        .map(|backend| app.get_backend_description(backend))
        .collect::<Vec<_>>();

    let backend_selector = pick_list(
        backend_options,
        Some(app.render_settings.backend.clone()),
        Message::BackendChanged,
    );
    let backend_info = text(format!("Available: {}", backend_descriptions.join(", ")))
        .size(10)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)));

    let backend_caps = text(app.backend_capabilities(&app.render_settings.backend))
        .size(12)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.35, 0.35, 0.35)));

    let foreground_input = text_input("#RRGGBB or #RRGGBBAA", &app.foreground_input)
        .on_input(Message::ForegroundChanged)
        .size(14);

    let background_toggle =
        checkbox("Use background", app.background_enabled).on_toggle(Message::BackgroundToggled);

    let background_input = text_input("#RRGGBB or #RRGGBBAA", &app.background_input)
        .on_input(Message::BackgroundChanged)
        .size(14);

    let background_hint = if app.background_enabled {
        "Background is enabled; lower alpha for transparency."
    } else {
        "Background disabled → transparent renders."
    };
    let background_hint = text(background_hint)
        .size(10)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.45, 0.45, 0.45)));

    let backend_row = column![
        row![
            text("Rendering Backend: ").size(14),
            backend_selector,
            button("Test Backend").on_press(Message::TestBackend),
        ]
        .spacing(10)
        .align_items(iced::Alignment::Center),
        backend_info,
        backend_caps,
    ]
    .spacing(4);

    let layout_selector = pick_list(
        LayoutMode::options(),
        Some(app.layout_mode),
        Message::LayoutChanged,
    );
    let layout_hint = text(match app.layout_mode {
        LayoutMode::Single => "Single column for detailed metadata",
        LayoutMode::SideBySide => "Pairs previews for quick comparison",
    })
    .size(10)
    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.45, 0.45, 0.45)));

    let layout_controls = column![
        text("Preview Layout").size(16),
        row![layout_selector]
            .spacing(8)
            .align_items(iced::Alignment::Center),
        layout_hint,
    ]
    .spacing(6);

    let render_scope_toggle = checkbox("Render selected font only", app.render_selected_only)
        .on_toggle(Message::RenderSelectedOnlyToggled);
    let render_scope_hint = text("Use this to speed up renders when working with large font sets; falls back to all visible fonts when disabled.")
        .size(10)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.45, 0.45, 0.45)));

    let color_controls = column![
        text("Colors").size(16),
        row![text("Foreground").size(12), foreground_input]
            .spacing(8)
            .align_items(iced::Alignment::Center),
        row![background_toggle, background_input]
            .spacing(8)
            .align_items(iced::Alignment::Center),
        background_hint,
    ]
    .spacing(6);

    let mut render_btn = button(render_state.cta_label());
    if render_state.can_render() {
        render_btn = render_btn.on_press(Message::RenderPreviews);
    } else {
        render_btn = render_btn.style(iced::theme::Button::Secondary);
    }
    let open_render_window_btn = button("Open Render Window")
        .on_press(Message::OpenRenderWindow)
        .style(iced::theme::Button::Secondary);
    let export_btn = button("Export PNGs")
        .on_press(Message::ExportPreviews)
        .style(iced::theme::Button::Secondary);
    let shortcut_hint = text("Shortcuts: ⌘/Ctrl+O add fonts, ⌘/Ctrl+R render, ⌘/Ctrl+E export, ⌘/Ctrl+W open render window")
        .size(10)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.45, 0.45, 0.45)));
    let render_hint = text(render_state.hint())
        .size(10)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.45, 0.45, 0.45)));

    column![
        render_header,
        sample_text_input,
        font_size_input,
        backend_row,
        layout_controls,
        render_scope_toggle,
        render_scope_hint,
        color_controls,
        row![render_btn, open_render_window_btn, export_btn]
            .spacing(10)
            .align_items(iced::Alignment::Center),
        render_hint,
        shortcut_hint,
    ]
    .spacing(10)
    .into()
}

/// Build the preview area view.
fn preview_area_view<'a>(
    app: &'a TestypfApp,
    visible_indices: &[usize],
    render_state: RenderAvailability,
) -> Element<'a, Message> {
    let preview_header = text("Font Previews").size(18);

    let preview_content: Element<Message> = if app.fonts.is_empty() {
        text("No fonts loaded - add fonts to see previews")
            .size(14)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5)))
            .into()
    } else if visible_indices.is_empty() {
        text("No fonts match the current filter")
            .size(14)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.3, 0.3)))
            .into()
    } else if app.render_previews.is_empty() {
        container(
            column![
                text("No previews yet").size(14),
                text(render_state.hint())
                    .size(12)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
            ]
            .spacing(6),
        )
        .padding(12)
        .width(Length::Fill)
        .style(iced::theme::Container::Box)
        .into()
    } else {
        scrollable(preview_rows(app, false)).into()
    };

    column![preview_header, preview_content].spacing(10).into()
}

/// Build preview rows for the main window.
fn preview_rows(app: &TestypfApp, condensed: bool) -> Element<'_, Message> {
    let cards: Vec<Element<Message>> = app
        .render_previews
        .iter()
        .filter_map(|preview| {
            let font = app.fonts.get(preview.font_index)?;
            let passes_filter = app.font_matches_filter(font)
                || (app.render_selected_only && app.selected_font == Some(preview.font_index));
            if !passes_filter {
                return None;
            }
            Some(if condensed {
                overlay_preview_card(app, preview)
            } else {
                preview_card(app, preview)
            })
        })
        .collect();

    helpers::layout_previews(cards, app.layout_mode)
}

/// Card used in the main window with full metadata.
fn preview_card<'a>(
    app: &'a TestypfApp,
    preview: &'a crate::types::RenderPreview,
) -> Element<'a, Message> {
    if let Some(font) = app.fonts.get(preview.font_index) {
        let image_widget = iced_image::Image::new(preview.handle.clone())
            .width(Length::Shrink)
            .height(Length::Shrink);
        let metadata_text = helpers::preview_metadata_text(preview, font, &app.render_settings);

        container(
            column![
                text(format!("{} - Rendered", font.full_name))
                    .size(16)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.2, 0.8))),
                text(format!("Sample: \"{}\"", app.render_settings.sample_text)).size(12),
                image_widget,
                text(metadata_text)
                    .size(10)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
            ]
            .spacing(5),
        )
        .padding(10)
        .width(Length::FillPortion(1))
        .style(iced::theme::Container::Box)
        .into()
    } else {
        container(
            text("Font data not found for render result")
                .size(14)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.3, 0.3))),
        )
        .padding(10)
        .width(Length::FillPortion(1))
        .style(iced::theme::Container::Box)
        .into()
    }
}

/// Lighter card for the transparent render window (used in main when condensed=true).
fn overlay_preview_card<'a>(
    app: &'a TestypfApp,
    preview: &'a crate::types::RenderPreview,
) -> Element<'a, Message> {
    if let Some(font) = app.fonts.get(preview.font_index) {
        let image_widget = iced_image::Image::new(preview.handle.clone())
            .width(Length::Shrink)
            .height(Length::Shrink);
        let metadata_text = helpers::preview_metadata_text(preview, font, &app.render_settings);

        container(
            column![
                text(metadata_text)
                    .size(14)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.85, 0.9, 0.95))),
                image_widget,
            ]
            .spacing(8),
        )
        .padding(12)
        .style(iced::theme::Container::Transparent)
        .width(Length::FillPortion(1))
        .into()
    } else {
        container(
            text("Font data missing for render preview")
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.7, 0.7))),
        )
        .padding(12)
        .style(iced::theme::Container::Transparent)
        .width(Length::FillPortion(1))
        .into()
    }
}
