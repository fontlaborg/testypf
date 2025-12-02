//! Render window view for testypf GUI application.

use crate::app::TestypfApp;
use crate::helpers;
use crate::message::Message;

use iced::widget::{column, container, image as iced_image, scrollable, text};
use iced::{Element, Length};

/// Render the transparent render window view.
pub fn render(app: &TestypfApp) -> Element<'_, Message> {
    let header = text("Render Previews (Transparent Window)")
        .size(18)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(
            0.9, 0.9, 0.95,
        )));

    let subtitle = text("Use the main window to add fonts and trigger renders.")
        .size(12)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(
            0.8, 0.8, 0.85,
        )));

    let body: Element<Message> = if app.render_previews.is_empty() {
        container(
            text("No render previews yet. Click \"Render Previews\" in the main window.")
                .size(14)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.85, 0.85, 0.9,
                ))),
        )
        .padding(20)
        .width(Length::Fill)
        .style(iced::theme::Container::Transparent)
        .into()
    } else {
        scrollable(preview_rows(app, true)).into()
    };

    container(column![header, subtitle, body].spacing(10))
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Transparent)
        .into()
}

/// Build preview rows for the render window.
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

/// Lighter card for the transparent render window.
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
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.85, 0.9, 0.95,
                    ))),
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
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.9, 0.7, 0.7,
                ))),
        )
        .padding(12)
        .style(iced::theme::Container::Transparent)
        .width(Length::FillPortion(1))
        .into()
    }
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
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.2, 0.2, 0.8
                    ))),
                text(format!("Sample: \"{}\"", app.render_settings.sample_text)).size(12),
                image_widget,
                text(metadata_text)
                    .size(10)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.6, 0.6, 0.6
                    ))),
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
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.8, 0.3, 0.3,
                ))),
        )
        .padding(10)
        .width(Length::FillPortion(1))
        .style(iced::theme::Container::Box)
        .into()
    }
}
