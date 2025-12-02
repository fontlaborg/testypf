//! Custom styles for testypf GUI application.

use iced::{widget::container, Theme};

/// Custom container style for active drag state.
pub struct DragActiveStyle;

impl container::StyleSheet for DragActiveStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.85, 0.95, 1.0, 0.9,
            ))),
            text_color: Some(iced::Color::from_rgb(0.05, 0.25, 0.45)),
            border: iced::border::Border {
                color: iced::Color::from_rgb(0.0, 0.55, 0.9),
                width: 3.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.3, 0.6, 0.35),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 18.0,
            },
        }
    }
}
