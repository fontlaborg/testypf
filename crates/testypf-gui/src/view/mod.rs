//! View rendering for testypf GUI application.

mod main_view;
mod render_view;

use crate::app::TestypfApp;
use crate::message::Message;
use iced::{window, Element};

/// Dispatch to the appropriate view based on window ID.
pub fn render<'a>(app: &'a TestypfApp, window: window::Id) -> Element<'a, Message> {
    if Some(window) == app.render_window_id {
        render_view::render(app)
    } else {
        main_view::render(app)
    }
}
