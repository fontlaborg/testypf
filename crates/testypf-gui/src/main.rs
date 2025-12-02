//! Main entry point for testypf GUI application.
//!
//! A minimal-yet-fast cross-platform GUI app showcasing typf rendering,
//! typg discovery, and fontlift install flows.

mod app;
mod helpers;
mod message;
mod styles;
mod types;
mod update;
mod view;

fn main() -> iced::Result {
    app::run()
}

#[cfg(test)]
mod tests;
