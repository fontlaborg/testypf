//! UI components for testypf GUI application
//!
//! This module contains the various UI components used by the testypf application.

use iced::{
    widget::{button, column, container, row, text, text_input},
    Element, Length,
};

use crate::Message;
use testypf_core::{FontInfo, RenderSettings};

/// Create font list view
pub fn font_list_view(fonts: &[FontInfo]) -> Element<Message> {
    if fonts.is_empty() {
        text("No fonts loaded")
            .size(14)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.5, 0.5, 0.5,
            )))
            .into()
    } else {
        container(
            column(
                fonts
                    .iter()
                    .enumerate()
                    .map(|(i, font)| {
                        let font_info =
                            text(format!("{} ({})", font.full_name, font.family_name)).size(14);

                        let remove_btn = button("Remove").on_press(Message::RemoveFont(i));

                        let install_btn = button("Install").on_press(Message::InstallFont(i));

                        let uninstall_btn = button("Uninstall").on_press(Message::UninstallFont(i));

                        row![font_info, remove_btn, install_btn, uninstall_btn]
                            .spacing(10)
                            .align_items(iced::Alignment::Center)
                            .into()
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(5),
        )
        .into()
    }
}

/// Create render controls view
pub fn render_controls_view(settings: &RenderSettings) -> Element<Message> {
    let sample_text_input = text_input("Enter sample text...", &settings.sample_text)
        .on_input(Message::SampleTextChanged)
        .size(14);

    let font_size_input = text_input("Font size", &settings.font_size.to_string())
        .on_input(Message::FontSizeChanged)
        .size(14);

    let backend_text = text(format!("Backend: {:?}", settings.backend)).size(14);

    let render_btn = button("Render Previews").on_press(Message::RenderPreviews);

    column![sample_text_input, font_size_input, backend_text, render_btn,]
        .spacing(10)
        .into()
}

/// Create status bar view
pub fn status_bar_view(status: &str) -> Element<Message> {
    container(text(status).size(14))
        .padding(10)
        .width(Length::Fill)
        .into()
}
