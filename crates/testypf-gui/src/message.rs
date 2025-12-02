//! Message types for testypf GUI application.

use crate::types::{InstallScope, LayoutMode};
use iced::window;
use std::path::PathBuf;
use testypf_core::RendererBackend;

/// All possible messages the application can receive.
#[derive(Debug, Clone)]
pub enum Message {
    // Font management
    AddFonts,
    FontsSelected(Option<Vec<PathBuf>>),
    FilesDropped(Vec<PathBuf>),
    RemoveFont(usize),
    InstallFont(usize),
    UninstallFont(usize),
    InstallScopeChanged(InstallScope),
    SelectFont(usize),
    FontFilterChanged(String),

    // Rendering controls
    SampleTextChanged(String),
    FontSizeChanged(String),
    BackendChanged(RendererBackend),
    TestBackend,
    ForegroundChanged(String),
    BackgroundChanged(String),
    BackgroundToggled(bool),
    LayoutChanged(LayoutMode),
    RenderSelectedOnlyToggled(bool),
    ExportPreviews,
    ExportDestinationChosen(Option<PathBuf>),

    // UI actions
    RenderPreviews,
    OpenRenderWindow,
    WindowClosed(window::Id),

    // Drag and drop state
    #[allow(dead_code)]
    DragEnter,
    DragLeave,
    FileHovered(PathBuf),
    ProcessPendingDrops,

    // Status messages (placeholder for future async status updates)
    #[allow(dead_code)]
    StatusUpdate(String),

    // No operation (placeholder for discarded events)
    #[allow(dead_code)]
    None,
}
