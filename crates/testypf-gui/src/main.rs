//! Main entry point for testypf GUI application
//!
//! A minimal-yet-fast cross-platform GUI app showcasing typf rendering,
//! typg discovery, and fontlift install flows.

use iced::{
    event,
    keyboard::{self, Key},
    multi_window::{self, Application},
    widget::{
        button, checkbox, column, container, image as iced_image, pick_list, row, scrollable, text,
        text_input,
    },
    window, Command, Element, Event, Length, Settings, Subscription, Theme,
};

use iced::widget::image::Handle;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use testypf_core::{
    FontInfo, FontScope, RenderResult, RenderSettings, RendererBackend, TestypfEngine,
};

mod ui;

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
    DragEnter,
    DragLeave,
    FileHovered(PathBuf),
    ProcessPendingDrops,

    // Status messages
    StatusUpdate(String),

    // No operation
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallScope {
    User,
    System,
}

impl InstallScope {
    const OPTIONS: [InstallScope; 2] = [InstallScope::User, InstallScope::System];

    fn to_font_scope(self) -> FontScope {
        match self {
            InstallScope::User => FontScope::User,
            InstallScope::System => FontScope::System,
        }
    }

    fn description(self) -> &'static str {
        match self {
            InstallScope::User => "User (~/Library/Fonts)",
            InstallScope::System => "System (/Library/Fonts, requires admin)",
        }
    }
}

impl std::fmt::Display for InstallScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}

struct TestypfApp {
    engine: TestypfEngine,
    fonts: Vec<FontInfo>,
    render_settings: RenderSettings,
    status: String,
    render_previews: Vec<RenderPreview>,
    is_dragging: bool,
    hovered_file: Option<PathBuf>,
    available_backends: Vec<RendererBackend>,
    last_scan_stats: Option<ScanStats>,
    pending_drop_paths: Vec<PathBuf>,
    drop_processing_scheduled: bool,
    font_ops_available: bool,
    render_window_id: Option<window::Id>,
    last_render_settings: Option<RenderSettings>,
    last_render_font_paths: Vec<PathBuf>,
    foreground_input: String,
    background_input: String,
    background_enabled: bool,
    layout_mode: LayoutMode,
    install_scope: InstallScope,
    selected_font: Option<usize>,
    font_filter: String,
    render_selected_only: bool,
}

#[derive(Clone)]
struct RenderPreview {
    font_index: usize,
    width: u32,
    height: u32,
    format: String,
    pixels: Vec<u8>,
    handle: Handle,
    duration_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DropPathKind {
    FontFile,
    Directory,
    Unsupported,
    Missing,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
struct AppConfig {
    backend: RendererBackend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Single,
    SideBySide,
}

impl LayoutMode {
    fn options() -> Vec<Self> {
        vec![Self::Single, Self::SideBySide]
    }
}

impl std::fmt::Display for LayoutMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutMode::Single => write!(f, "Single column"),
            LayoutMode::SideBySide => write!(f, "Side-by-side"),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ScanStats {
    directories_scanned: usize,
    files_checked: usize,
    fonts_found: usize,
    sample_files: Vec<String>,
}

const FONT_EXTENSIONS: &[&str] = &[
    "ttf", "otf", "ttc", "otc", "woff", "woff2", "dfont", "eot", "svg", "pfa", "pfb", "otb",
];

impl multi_window::Application for TestypfApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut engine = TestypfEngine::new().expect("Failed to initialize testypf engine");
        let mut render_settings = RenderSettings::default();
        let mut status = "Ready".to_string();

        // Detect backends from the renderer and drop non-visual JSON backend from the UI
        let available_backends = engine
            .text_renderer()
            .get_backends()
            .into_iter()
            .filter(|backend| !matches!(backend, RendererBackend::Json))
            .collect::<Vec<_>>();

        let font_ops_available = TestypfEngine::font_ops_available();

        // Load persisted configuration if present and compatible with current build
        if let Ok(config) = Self::load_config() {
            if available_backends.iter().any(|b| b == &config.backend) {
                let _ = engine.text_renderer().set_backend(config.backend.clone());
                render_settings.backend = config.backend;
                status = "Loaded saved backend selection".to_string();
            }
        }

        let install_scope = InstallScope::User;
        engine.set_font_install_scope(install_scope.to_font_scope());

        let app = Self {
            engine,
            fonts: Vec::new(),
            render_settings,
            status,
            render_previews: Vec::new(),
            is_dragging: false,
            hovered_file: None,
            available_backends,
            last_scan_stats: None,
            pending_drop_paths: Vec::new(),
            drop_processing_scheduled: false,
            font_ops_available,
            render_window_id: None,
            last_render_settings: None,
            last_render_font_paths: Vec::new(),
            foreground_input: "#000000FF".to_string(),
            background_input: "#00000000".to_string(),
            background_enabled: false,
            layout_mode: LayoutMode::Single,
            install_scope,
            selected_font: None,
            font_filter: String::new(),
            render_selected_only: false,
        };

        (app, Command::none())
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status| match event {
            Event::Window(id, iced::window::Event::FileHovered(path)) if id == window::Id::MAIN => {
                Some(Message::FileHovered(path))
            }
            Event::Window(id, iced::window::Event::FileDropped(path)) if id == window::Id::MAIN => {
                Some(Message::FilesDropped(vec![path]))
            }
            Event::Window(id, iced::window::Event::FilesHoveredLeft) if id == window::Id::MAIN => {
                Some(Message::DragLeave)
            }
            Event::Window(id, iced::window::Event::Closed) => Some(Message::WindowClosed(id)),
            Event::Window(id, iced::window::Event::CloseRequested) => {
                Some(Message::WindowClosed(id))
            }
            Event::Keyboard(key_event) => Self::shortcut_to_message(&key_event),
            _ => None,
        })
    }

    fn title(&self, window: window::Id) -> String {
        if Some(window) == self.render_window_id {
            "Testypf Render Window".to_string()
        } else {
            "Testypf - Typf GUI Tester".to_string()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SampleTextChanged(text) => {
                self.render_settings.sample_text = text;
                self.status = "Sample text updated".to_string();
                self.invalidate_render_cache();
            }

            Message::FontSizeChanged(size_str) => {
                if let Ok(size) = size_str.parse::<f32>() {
                    self.render_settings.font_size = size;
                    self.status = "Font size updated".to_string();
                    self.invalidate_render_cache();
                } else {
                    self.status = "Invalid font size".to_string();
                }
            }

            Message::BackendChanged(backend) => {
                let _ = self.engine.text_renderer().set_backend(backend.clone());
                self.render_settings.backend = backend.clone();
                if let Err(e) = Self::save_config(&AppConfig {
                    backend: backend.clone(),
                }) {
                    eprintln!("Failed to persist backend selection: {}", e);
                }
                self.status = format!("Backend changed to {}", backend);
                self.invalidate_render_cache();
            }

            Message::ForegroundChanged(value) => {
                self.foreground_input = value.clone();
                match Self::parse_rgba_hex(&value) {
                    Some(color) => {
                        self.render_settings.foreground_color = color;
                        self.status = "Foreground color updated".to_string();
                        self.invalidate_render_cache();
                    }
                    None => {
                        self.status = "Foreground color must be #RRGGBB or #RRGGBBAA".to_string();
                    }
                }
            }

            Message::BackgroundChanged(value) => {
                self.background_input = value.clone();
                if !self.background_enabled {
                    return Command::none();
                }

                match Self::parse_rgba_hex(&value) {
                    Some(color) => {
                        self.render_settings.background_color = Some(color);
                        self.status = "Background color updated".to_string();
                        self.invalidate_render_cache();
                    }
                    None => {
                        self.status = "Background color must be #RRGGBB or #RRGGBBAA".to_string();
                    }
                }
            }

            Message::BackgroundToggled(enabled) => {
                self.background_enabled = enabled;
                if enabled {
                    if let Some(color) = Self::parse_rgba_hex(&self.background_input) {
                        self.render_settings.background_color = Some(color);
                        self.status = "Background enabled".to_string();
                        self.invalidate_render_cache();
                    } else {
                        self.render_settings.background_color = Some((0, 0, 0, 0));
                        self.status =
                            "Background enabled with default transparent color".to_string();
                        self.invalidate_render_cache();
                    }
                } else {
                    self.render_settings.background_color = None;
                    self.status = "Background disabled (transparent)".to_string();
                    self.invalidate_render_cache();
                }
            }

            Message::LayoutChanged(mode) => {
                if self.layout_mode != mode {
                    self.layout_mode = mode;
                    self.status = format!("Layout changed to {}", self.layout_mode);
                }
            }

            Message::RenderSelectedOnlyToggled(enabled) => {
                self.render_selected_only = enabled;
                if enabled && self.selected_font.is_none() {
                    self.status =
                        "Select a font to render when 'selected only' is enabled".to_string();
                } else if enabled {
                    self.status = "Rendering limited to the selected font".to_string();
                } else {
                    self.status = "Rendering all visible fonts".to_string();
                }
            }

            Message::InstallScopeChanged(scope) => {
                self.install_scope = scope;
                self.engine
                    .font_manager()
                    .set_install_scope(scope.to_font_scope());
                self.status = format!("Install scope set to {}", scope);
            }

            Message::TestBackend => {
                if let Some(font) = self.fonts.first().cloned() {
                    let mut settings = self.render_settings.clone();
                    settings.sample_text = "Backend self-test".to_string();
                    settings.font_size = 18.0;
                    let started = Instant::now();

                    match self
                        .engine
                        .text_renderer()
                        .render_text(&font.path, &settings)
                    {
                        Ok(_) => {
                            let elapsed = started.elapsed().as_millis();
                            self.status = format!(
                                "Backend {} OK in {} ms using {}",
                                settings.backend, elapsed, font.full_name
                            );
                        }
                        Err(e) => {
                            self.status = format!("Backend test failed: {}", e);
                        }
                    }
                } else {
                    self.status = "Load a font before testing the backend".to_string();
                }
            }

            Message::AddFonts => {
                // Trigger file dialog - this is handled asynchronously
                self.status = "Opening file dialog...".to_string();
                return Command::perform(
                    async {
                        // For now, use a simplified approach - in a real implementation
                        // we'd need to handle the async file dialog properly
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        rfd::FileDialog::new()
                            .add_filter(
                                "Font Files",
                                &["ttf", "otf", "ttc", "otc", "woff", "woff2"],
                            )
                            .pick_files()
                    },
                    Message::FontsSelected,
                );
            }

            Message::FontsSelected(paths) => match paths {
                Some(paths) => self.process_dropped_paths(paths),
                None => {
                    self.status = "No fonts selected".to_string();
                }
            },

            Message::FilesDropped(paths) => {
                self.is_dragging = false;
                self.hovered_file = None;
                self.pending_drop_paths.extend(paths);
                return self.enqueue_drop_processing();
            }

            Message::ProcessPendingDrops => {
                self.drop_processing_scheduled = false;
                let pending = std::mem::take(&mut self.pending_drop_paths);
                if pending.is_empty() {
                    return Command::none();
                }
                self.process_dropped_paths(pending);
            }

            Message::DragEnter => {
                self.is_dragging = true;
                self.status = "Drag files or folders here...".to_string();
            }

            Message::DragLeave => {
                self.is_dragging = false;
                self.hovered_file = None;
                self.status = if self.fonts.is_empty() {
                    "No fonts loaded. Add fonts to get started.".to_string()
                } else {
                    format!("Loaded {} font(s)", self.fonts.len())
                };
            }

            Message::FileHovered(path) => {
                self.is_dragging = true;
                self.hovered_file = Some(path.clone());
                if let Some(ref file) = self.hovered_file {
                    self.status = format!(
                        "Drop: {} (Ready to add this font)",
                        file.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
            }

            Message::RemoveFont(index) => {
                if index < self.fonts.len() {
                    let font = &self.fonts[index];
                    let _ = self.engine.font_manager().remove_font(&font.path);
                    self.fonts.remove(index);
                    if self.selected_font == Some(index)
                        || self.selected_font.map(|i| i > index).unwrap_or(false)
                    {
                        self.selected_font = None;
                    }
                    self.status = "Font removed".to_string();
                    self.invalidate_render_cache();
                    self.render_previews.clear();
                }
            }

            Message::SelectFont(index) => {
                if index < self.fonts.len() {
                    if self.selected_font == Some(index) {
                        self.selected_font = None;
                        self.status = "Font details hidden".to_string();
                    } else {
                        self.selected_font = Some(index);
                        self.status =
                            format!("Showing details for {}", self.fonts[index].full_name);
                    }
                }
            }

            Message::FontFilterChanged(filter) => {
                self.font_filter = filter;
                self.status = format!(
                    "Filtered fonts: showing {} of {}",
                    self.visible_font_indices().len(),
                    self.fonts.len()
                );
            }

            Message::InstallFont(index) => {
                if !self.font_ops_available {
                    self.status = "Font install unavailable in this build; enable platform-mac or platform-win features.".to_string();
                    return Command::none();
                }

                if index < self.fonts.len() {
                    let font_full_name = self.fonts[index].full_name.clone();
                    let font = self.fonts[index].clone();
                    match self.engine.font_manager().install_font(&font) {
                        Ok(()) => {
                            self.refresh_install_status(index);
                            self.status = format!(
                                "Font '{}' installed to {}",
                                font_full_name,
                                self.install_scope.description()
                            );
                        }
                        Err(e) => {
                            self.status = format!(
                                "Failed to install font ({}): {}",
                                self.install_scope.description(),
                                e
                            );
                        }
                    }
                }
            }

            Message::UninstallFont(index) => {
                if !self.font_ops_available {
                    self.status = "Font uninstall unavailable in this build; enable platform-mac or platform-win features.".to_string();
                    return Command::none();
                }

                if index < self.fonts.len() {
                    let font_full_name = self.fonts[index].full_name.clone();
                    let font = self.fonts[index].clone();
                    match self.engine.font_manager().uninstall_font(&font) {
                        Ok(()) => {
                            self.refresh_install_status(index);
                            self.status = format!(
                                "Font '{}' uninstalled from {}",
                                font_full_name,
                                self.install_scope.description()
                            );
                        }
                        Err(e) => {
                            self.status = format!("Failed to uninstall font: {}", e);
                        }
                    }
                }
            }

            Message::RenderPreviews => {
                if self.fonts.is_empty() {
                    self.status = "No fonts to render".to_string();
                    return Command::none();
                }

                let visible_indices = self.visible_font_indices();
                let target_indices = Self::derive_render_targets(
                    self.selected_font,
                    &visible_indices,
                    self.render_selected_only,
                );

                if target_indices.is_empty() {
                    self.status = if self.render_selected_only {
                        "Select a font to render when 'selected only' is enabled".to_string()
                    } else {
                        "No fonts match the current filter".to_string()
                    };
                    return Command::none();
                }

                let font_paths: Vec<PathBuf> = target_indices
                    .iter()
                    .filter_map(|&i| self.fonts.get(i).map(|f| f.path.clone()))
                    .collect();

                if font_paths.is_empty() {
                    self.status = "No fonts available to render".to_string();
                    return Command::none();
                }

                if self.render_cache_hit(&font_paths) {
                    let mut cmds = Vec::new();
                    cmds.push(self.ensure_render_window());
                    if let Some(id) = self.render_window_id {
                        cmds.push(window::gain_focus(id));
                    }
                    self.status = "Render settings unchanged - using cached previews".to_string();
                    return Command::batch(cmds);
                }

                self.status = format!(
                    "Rendering {} of {} font(s)...",
                    font_paths.len(),
                    self.fonts.len()
                );

                // Clear previous results
                self.render_previews.clear();

                // Render each selected font synchronously (for now)
                let render_start = Instant::now();
                let mut previews = Vec::new();
                for font_index in target_indices {
                    if let Some(font) = self.fonts.get(font_index) {
                        let per_start = Instant::now();
                        match self
                            .engine
                            .text_renderer()
                            .render_text(&font.path, &self.render_settings)
                        {
                            Ok(render_result) => {
                                let duration_ms = per_start.elapsed().as_millis();
                                match self.build_render_preview(
                                    font_index,
                                    render_result,
                                    duration_ms,
                                ) {
                                    Ok(preview) => previews.push(preview),
                                    Err(e) => {
                                        self.status = format!(
                                            "Failed to create preview for font {}: {}",
                                            font.full_name, e
                                        );
                                        return Command::none();
                                    }
                                }
                            }
                            Err(e) => {
                                self.status =
                                    format!("Failed to render font {}: {}", font.full_name, e);
                                return Command::none();
                            }
                        }
                    }
                }

                self.render_previews = previews;
                self.last_render_settings = Some(self.render_settings.clone());
                self.last_render_font_paths = font_paths;
                self.status = format!(
                    "Rendering complete - {} preview(s) generated in {} ms",
                    self.render_previews.len(),
                    render_start.elapsed().as_millis()
                );

                let mut cmds = Vec::new();

                // Ensure render window is available and focused for transparent preview
                cmds.push(self.ensure_render_window());
                if let Some(id) = self.render_window_id {
                    cmds.push(window::gain_focus(id));
                }

                return Command::batch(cmds);
            }

            Message::OpenRenderWindow => {
                let mut cmds = vec![self.ensure_render_window()];
                if let Some(id) = self.render_window_id {
                    cmds.push(window::gain_focus(id));
                }
                return Command::batch(cmds);
            }

            Message::ExportPreviews => {
                if self.render_previews.is_empty() {
                    self.status = "Render previews before exporting them".to_string();
                    return Command::none();
                }

                self.status = "Choose a folder to save PNG previews...".to_string();
                return Command::perform(
                    async { rfd::FileDialog::new().pick_folder() },
                    Message::ExportDestinationChosen,
                );
            }

            Message::ExportDestinationChosen(destination) => match destination {
                Some(folder) => match self.export_previews_to_folder(&folder) {
                    Ok(written) => {
                        self.status =
                            format!("Exported {} preview(s) to {}", written, folder.display());
                    }
                    Err(e) => {
                        self.status = format!("Export failed: {}", e);
                    }
                },
                None => {
                    self.status = "Export cancelled".to_string();
                }
            },

            Message::WindowClosed(id) => {
                if Some(id) == self.render_window_id {
                    self.render_window_id = None;
                    self.status = "Render window closed".to_string();
                }
            }

            Message::StatusUpdate(msg) => {
                self.status = msg;
            }

            Message::None => {}
        }

        Command::none()
    }

    fn view(&self, window: window::Id) -> Element<Message> {
        if Some(window) == self.render_window_id {
            return self.render_window_view();
        }

        let title = text("Testypf - Typf GUI Tester")
            .size(24)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.2, 0.2, 0.8,
            )));

        let status = text(&self.status).size(14);
        let visible_indices = self.visible_font_indices();

        // Font list section
        let font_list_header = text("Font List").size(18);

        let font_filter = text_input(
            "Filter fonts (name, family, style, path)",
            &self.font_filter,
        )
        .on_input(Message::FontFilterChanged)
        .size(12);
        let filter_hint = text(format!(
            "Showing {} of {}",
            visible_indices.len(),
            self.fonts.len()
        ))
        .size(10)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(
            0.45, 0.45, 0.45,
        )));

        let install_scope_picker = row![
            text("Install scope").size(12),
            pick_list(
                InstallScope::OPTIONS.to_vec(),
                Some(self.install_scope),
                Message::InstallScopeChanged
            ),
            text(self.install_scope.description())
                .size(10)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.45, 0.45, 0.45
                )))
        ]
        .spacing(10)
        .align_items(iced::Alignment::Center);

        let font_list: Element<Message> = if self.fonts.is_empty() {
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
                    .filter_map(|&i| self.fonts.get(i).map(|font| (i, font)))
                    .map(|(i, font)| {
                        let font_info =
                            text(format!("{} ({})", font.full_name, font.family_name)).size(14);

                        let install_status = if font.is_installed {
                            text("Installed").size(12).style(iced::theme::Text::Color(
                                iced::Color::from_rgb(0.0, 0.5, 0.0),
                            ))
                        } else {
                            text("Not installed")
                                .size(12)
                                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                                    0.5, 0.2, 0.2,
                                )))
                        };

                        let mut details_btn = button("Details");
                        if self.selected_font == Some(i) {
                            details_btn = details_btn.style(iced::theme::Button::Primary);
                        }
                        details_btn = details_btn.on_press(Message::SelectFont(i));

                        let remove_btn = button("Remove").on_press(Message::RemoveFont(i));

                        let mut install_btn = button("Install");
                        if !self.font_ops_available {
                            install_btn = install_btn.style(iced::theme::Button::Secondary);
                        } else if !font.is_installed {
                            install_btn = install_btn.on_press(Message::InstallFont(i));
                        }

                        let mut uninstall_btn = button("Uninstall");
                        if !self.font_ops_available {
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
        };

        let font_ops_notice: Option<Element<Message>> = if !self.font_ops_available {
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

        let metadata_panel: Element<Message> = if let Some(selected) =
            self.selected_font.and_then(|i| self.fonts.get(i))
        {
            let file_size = Self::font_file_size(&selected.path);
            let lines = Self::font_metadata_lines(selected, file_size);
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
                    text("Select a font to view its details.").size(12).style(
                        iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5,))
                    ),
                ]
                .spacing(6),
            )
            .padding(12)
            .width(Length::Fill)
            .style(iced::theme::Container::Box)
            .into()
        };

        let scan_summary = self.last_scan_stats.as_ref().map(|stats| {
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

        // Enhanced drop area with visual feedback and file hover info
        let drop_area_content = if self.is_dragging {
            let hover_info = if let Some(ref hovered_file) = self.hovered_file {
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
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.0, 0.4, 0.8,
                    )))
                } else {
                    text(format!("📄 File: {} (Click to add this font)", file_name))
                        .size(14)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(
                            0.0, 0.6, 0.0,
                        )))
                }
            } else {
                text("🎯 Drop fonts or folders here!")
                    .size(18)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.0, 0.6, 0.0,
                    )))
            };

            column![
                hover_info,
                text(format!("Supports {}", Self::supported_formats_text()))
                    .size(12)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.3, 0.3, 0.3
                    ))),
                text("✨ Recursive folder scanning with progress feedback")
                    .size(10)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.4, 0.4, 0.4
                    ))),
                text("💡 Tip: You can drop multiple files and folders at once!")
                    .size(9)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.5, 0.5, 0.5
                    ))),
                scan_summary
                    .as_ref()
                    .map(|summary| {
                        text(summary).size(10).style(iced::theme::Text::Color(
                            iced::Color::from_rgb(0.3, 0.5, 0.3),
                        ))
                    })
                    .unwrap_or_else(|| text("").into()),
            ]
            .spacing(6)
            .align_items(iced::Alignment::Center)
        } else {
            let status_text = if self.fonts.is_empty() {
                text("📁 Drag & drop fonts to get started").size(16).style(
                    iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.4, 0.0)),
                )
            } else {
                text(format!(
                    "📁 Drag & drop more fonts ({} loaded)",
                    self.fonts.len()
                ))
                .size(16)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.2, 0.2, 0.8,
                )))
            };

            column![
                status_text,
                text(format!(
                    "Supports {} via files or folders (recursive)",
                    Self::supported_formats_text()
                ))
                .size(12)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.5, 0.5, 0.5
                ))),
                button("Add Fonts...")
                    .on_press(Message::AddFonts)
                    .style(iced::theme::Button::Secondary),
                scan_summary
                    .as_ref()
                    .map(|summary| {
                        text(summary).size(10).style(iced::theme::Text::Color(
                            iced::Color::from_rgb(0.3, 0.5, 0.3),
                        ))
                    })
                    .unwrap_or_else(|| text("").into()),
            ]
            .spacing(12)
            .align_items(iced::Alignment::Center)
        };

        let drop_area_style = if self.is_dragging {
            iced::theme::Container::Custom(Box::new(DragActiveStyle))
        } else {
            iced::theme::Container::Box
        };

        let drop_area = container(drop_area_content)
            .padding(20)
            .width(iced::Length::Fill)
            .height(iced::Length::Fixed(140.0))
            .center_x()
            .center_y()
            .style(drop_area_style);

        // Render controls section
        let render_header = text("Render Controls").size(18);

        let sample_text_input =
            text_input("Enter sample text...", &self.render_settings.sample_text)
                .on_input(Message::SampleTextChanged)
                .size(14);

        let font_size_input = text_input("Font size", &self.render_settings.font_size.to_string())
            .on_input(Message::FontSizeChanged)
            .size(14);

        // Dynamic backend selector with platform availability
        let backend_options = self.get_available_backends();
        let backend_descriptions = backend_options
            .iter()
            .map(|backend| self.get_backend_description(backend))
            .collect::<Vec<_>>();

        let backend_selector = pick_list(
            backend_options,
            Some(self.render_settings.backend.clone()),
            Message::BackendChanged,
        );
        let backend_info = text(format!("Available: {}", backend_descriptions.join(", ")))
            .size(10)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.6, 0.6, 0.6,
            )));

        let backend_caps = text(self.backend_capabilities(&self.render_settings.backend))
            .size(12)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.35, 0.35, 0.35,
            )));

        let foreground_input = text_input("#RRGGBB or #RRGGBBAA", &self.foreground_input)
            .on_input(Message::ForegroundChanged)
            .size(14);

        let background_toggle = checkbox("Use background", self.background_enabled)
            .on_toggle(Message::BackgroundToggled);

        let background_input = text_input("#RRGGBB or #RRGGBBAA", &self.background_input)
            .on_input(Message::BackgroundChanged)
            .size(14);

        let background_hint = if self.background_enabled {
            "Background is enabled; lower alpha for transparency."
        } else {
            "Background disabled → transparent renders."
        };
        let background_hint = text(background_hint)
            .size(10)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.45, 0.45, 0.45,
            )));

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
            Some(self.layout_mode),
            Message::LayoutChanged,
        );
        let layout_hint = text(match self.layout_mode {
            LayoutMode::Single => "Single column for detailed metadata",
            LayoutMode::SideBySide => "Pairs previews for quick comparison",
        })
        .size(10)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(
            0.45, 0.45, 0.45,
        )));

        let layout_controls = column![
            text("Preview Layout").size(16),
            row![layout_selector]
                .spacing(8)
                .align_items(iced::Alignment::Center),
            layout_hint,
        ]
        .spacing(6);

        let render_scope_toggle = checkbox("Render selected font only", self.render_selected_only)
            .on_toggle(Message::RenderSelectedOnlyToggled);
        let render_scope_hint = text("Use this to speed up renders when working with large font sets; falls back to all visible fonts when disabled.")
            .size(10)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.45, 0.45, 0.45,
            )));

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

        let render_btn = button("Render Previews").on_press(Message::RenderPreviews);
        let open_render_window_btn = button("Open Render Window")
            .on_press(Message::OpenRenderWindow)
            .style(iced::theme::Button::Secondary);
        let export_btn = button("Export PNGs")
            .on_press(Message::ExportPreviews)
            .style(iced::theme::Button::Secondary);
        let shortcut_hint = text("Shortcuts: ⌘/Ctrl+O add fonts, ⌘/Ctrl+R render, ⌘/Ctrl+E export, ⌘/Ctrl+W open render window")
            .size(10)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.45, 0.45, 0.45,
            )));

        // Font preview section
        let preview_header = text("Font Previews").size(18);

        let preview_area: Element<Message> = {
            if self.fonts.is_empty() {
                text("No fonts loaded - add fonts to see previews")
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
            } else if self.render_previews.is_empty() {
                scrollable(
                    column(
                        visible_indices.iter().filter_map(|&i| self.fonts.get(i)).map(|font| {
                            container(
                                column![
                                    text(format!("{} - Size: {}", font.full_name, self.render_settings.font_size))
                                        .size(16)
                                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.2, 0.8))),
                                    text(format!("Sample: \"{}\"", self.render_settings.sample_text))
                                        .size(14),
                                    text("Preview: [Click 'Render Previews' to see actual font rendering]")
                                        .size(12)
                                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                                    text(format!("Backend: {} | Style: {} | Family: {}",
                                        self.render_settings.backend, font.style, font.family_name))
                                        .size(10)
                                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                                ]
                                .spacing(5)
                            )
                            .padding(10)
                            .width(Length::Fill)
                            .style(iced::theme::Container::Box)
                            .into()
                        })
                        .collect::<Vec<_>>(),
                    )
                    .spacing(10)
                )
                .into()
            } else {
                // Display actual render results with images
                scrollable(self.preview_rows(false)).into()
            }
        };

        // Layout everything
        let content = column![
            title,
            container(status).padding(10),
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
            // Render controls section
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
            shortcut_hint,
            // Preview section
            preview_header,
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

    fn theme(&self, _window: window::Id) -> Theme {
        Theme::Light
    }
}

impl TestypfApp {
    /// Spawn the transparent render window if needed and return the command to create/focus it
    fn ensure_render_window(&mut self) -> Command<Message> {
        if let Some(id) = self.render_window_id {
            return window::gain_focus(id);
        }

        let settings = Self::render_window_settings();
        let (id, cmd) = window::spawn(settings);
        self.render_window_id = Some(id);
        cmd
    }

    /// Transparent render window configuration for overlay previews
    fn render_window_settings() -> window::Settings {
        let mut settings = window::Settings::default();
        settings.size = iced::Size::new(900.0, 650.0);
        settings.min_size = Some(iced::Size::new(640.0, 480.0));
        settings.decorations = false;
        settings.transparent = true;
        settings.level = window::Level::AlwaysOnTop;
        settings
    }

    /// Dedicated render window view with transparent background
    fn render_window_view(&self) -> Element<Message> {
        let header =
            text("Render Previews (Transparent Window)")
                .size(18)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.9, 0.9, 0.95,
                )));

        let subtitle = text("Use the main window to add fonts and trigger renders.")
            .size(12)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.8, 0.8, 0.85,
            )));

        let body: Element<Message> = if self.render_previews.is_empty() {
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
            scrollable(self.preview_rows(true)).into()
        };

        container(column![header, subtitle, body].spacing(10))
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Transparent)
            .into()
    }

    /// Recursively scan a folder for font files with progress feedback
    fn scan_folder_for_fonts(folder_path: &PathBuf) -> Result<(Vec<PathBuf>, ScanStats), String> {
        let mut font_files = Vec::new();
        let mut stats = ScanStats::default();

        fn visit_dir(
            dir: &PathBuf,
            font_files: &mut Vec<PathBuf>,
            stats: &mut ScanStats,
        ) -> Result<(), std::io::Error> {
            stats.directories_scanned += 1;

            if dir.is_dir() {
                let entries = std::fs::read_dir(dir)?;
                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    stats.files_checked += 1;

                    if path.is_dir() {
                        // Recursively visit subdirectories
                        visit_dir(&path, font_files, stats)?;
                    } else if path.is_file() {
                        // Check if file has a font extension
                        if TestypfApp::is_font_file(&path) {
                            stats.fonts_found += 1;
                            if stats.sample_files.len() < 3 {
                                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                    stats.sample_files.push(name.to_string());
                                }
                            }
                            font_files.push(path);
                        }
                    }
                }
            }
            Ok(())
        }

        // Update status to show scanning started
        println!("Scanning folder: {:?}", folder_path);

        if let Err(e) = visit_dir(folder_path, &mut font_files, &mut stats) {
            return Err(format!(
                "Failed to scan directory after scanning {} directories and {} files: {}",
                stats.directories_scanned, stats.files_checked, e
            ));
        }

        println!(
            "Folder scan complete: found {} font files in {} directories, {} total files checked",
            font_files.len(),
            stats.directories_scanned,
            stats.files_checked
        );

        Ok((font_files, stats))
    }

    /// Build a render preview with an iced image handle
    fn build_render_preview(
        &self,
        font_index: usize,
        render_result: RenderResult,
        duration_ms: u128,
    ) -> Result<RenderPreview, String> {
        let handle = Self::image_handle_from_render(&render_result)?;
        Ok(RenderPreview {
            font_index,
            width: render_result.width,
            height: render_result.height,
            format: render_result.format.clone(),
            pixels: render_result.data.clone(),
            handle,
            duration_ms,
        })
    }

    /// Determine whether a file path looks like a supported font
    fn is_font_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| FONT_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Categorize a dropped path to drive validation messaging
    fn classify_drop_path(path: &Path) -> DropPathKind {
        if !path.exists() {
            return DropPathKind::Missing;
        }

        if path.is_dir() {
            return DropPathKind::Directory;
        }

        if path.is_file() && Self::is_font_file(path) {
            DropPathKind::FontFile
        } else {
            DropPathKind::Unsupported
        }
    }

    /// Aggregate extension counts for status messaging when multiple font formats are dropped
    fn extension_stats(paths: &[PathBuf]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for path in paths {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if FONT_EXTENSIONS.contains(&ext_lower.as_str()) {
                    *counts.entry(ext_lower).or_insert(0) += 1;
                }
            }
        }
        counts
    }

    fn enqueue_drop_processing(&mut self) -> Command<Message> {
        if self.drop_processing_scheduled {
            return Command::none();
        }
        self.drop_processing_scheduled = true;
        Command::perform(
            async {
                std::thread::sleep(Duration::from_millis(60));
            },
            |_| Message::ProcessPendingDrops,
        )
    }

    fn format_extension_summary(ext_stats: &HashMap<String, usize>) -> String {
        if ext_stats.is_empty() {
            return String::new();
        }

        let mut parts: Vec<String> = ext_stats
            .iter()
            .map(|(ext, count)| format!("{}x .{}", count, ext))
            .collect();
        parts.sort();
        format!(" Formats: {}", parts.join(", "))
    }

    fn supported_formats_text() -> String {
        let mut exts = FONT_EXTENSIONS
            .iter()
            .map(|ext| format!(".{}", ext))
            .collect::<Vec<_>>();
        exts.sort();
        exts.join(", ")
    }

    /// How many rows are needed for the chosen layout and preview count
    fn layout_row_count(preview_count: usize, mode: LayoutMode) -> usize {
        match mode {
            LayoutMode::Single => preview_count,
            LayoutMode::SideBySide => (preview_count + 1) / 2,
        }
    }

    /// Arrange preview cards into rows based on the selected layout
    fn layout_previews(previews: Vec<Element<Message>>, mode: LayoutMode) -> Element<Message> {
        match mode {
            LayoutMode::Single => column(previews).spacing(10).into(),
            LayoutMode::SideBySide => {
                let mut rows = Vec::with_capacity(Self::layout_row_count(previews.len(), mode));
                let mut iter = previews.into_iter();

                while let Some(first) = iter.next() {
                    let second = iter.next();
                    let mut row_children =
                        vec![container(first).width(Length::FillPortion(1)).into()];
                    if let Some(second) = second {
                        row_children.push(container(second).width(Length::FillPortion(1)).into());
                    }

                    rows.push(
                        row(row_children)
                            .spacing(12)
                            .width(Length::Fill)
                            .align_items(iced::Alignment::Start)
                            .into(),
                    );
                }

                column(rows).spacing(12).into()
            }
        }
    }

    /// Card used in the main window with full metadata
    fn preview_card(&self, preview: &RenderPreview) -> Element<Message> {
        if let Some(font) = self.fonts.get(preview.font_index) {
            let image_widget = iced_image::Image::new(preview.handle.clone())
                .width(Length::Shrink)
                .height(Length::Shrink);
            let metadata_text = Self::preview_metadata_text(preview, font, &self.render_settings);

            container(
                column![
                    text(format!("{} - Rendered", font.full_name))
                        .size(16)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(
                            0.2, 0.2, 0.8
                        ))),
                    text(format!("Sample: \"{}\"", self.render_settings.sample_text)).size(12),
                    image_widget,
                    text(metadata_text).size(10).style(iced::theme::Text::Color(
                        iced::Color::from_rgb(0.6, 0.6, 0.6)
                    )),
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

    /// Lighter card for the transparent render window
    fn overlay_preview_card(&self, preview: &RenderPreview) -> Element<Message> {
        if let Some(font) = self.fonts.get(preview.font_index) {
            let image_widget = iced_image::Image::new(preview.handle.clone())
                .width(Length::Shrink)
                .height(Length::Shrink);
            let metadata_text = Self::preview_metadata_text(preview, font, &self.render_settings);

            container(
                column![
                    text(metadata_text).size(14).style(iced::theme::Text::Color(
                        iced::Color::from_rgb(0.85, 0.9, 0.95,)
                    )),
                    image_widget,
                ]
                .spacing(8),
            )
            .padding(12)
            .style(iced::theme::Container::Transparent)
            .width(Length::FillPortion(1))
            .into()
        } else {
            container(text("Font data missing for render preview").size(12).style(
                iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.7, 0.7)),
            ))
            .padding(12)
            .style(iced::theme::Container::Transparent)
            .width(Length::FillPortion(1))
            .into()
        }
    }

    /// Build preview rows honoring the selected layout
    fn preview_rows(&self, condensed: bool) -> Element<Message> {
        let cards: Vec<Element<Message>> = self
            .render_previews
            .iter()
            .filter_map(|preview| {
                let font = self.fonts.get(preview.font_index)?;
                let passes_filter = self.font_matches_filter(font)
                    || (self.render_selected_only
                        && self.selected_font == Some(preview.font_index));
                if !passes_filter {
                    return None;
                }
                Some(if condensed {
                    self.overlay_preview_card(preview)
                } else {
                    self.preview_card(preview)
                })
            })
            .collect();

        Self::layout_previews(cards, self.layout_mode)
    }

    /// Human-readable metadata string for a render preview
    fn preview_metadata_text(
        preview: &RenderPreview,
        font: &FontInfo,
        settings: &RenderSettings,
    ) -> String {
        format!(
            "Dimensions: {}x{} | Format: {} | Backend: {} | Style: {} | Family: {} | Render time: {} ms",
            preview.width,
            preview.height,
            preview.format,
            settings.backend,
            font.style,
            font.family_name,
            preview.duration_ms
        )
    }

    /// Derive user-friendly metadata lines for the selected font
    fn font_metadata_lines(font: &FontInfo, file_size_bytes: Option<u64>) -> Vec<String> {
        let mut lines = vec![
            format!("Name: {}", font.full_name),
            format!("Family: {}", font.family_name),
            format!("Style: {}", font.style),
            format!("PostScript: {}", font.postscript_name),
            format!(
                "Install state: {}",
                if font.is_installed {
                    "Installed"
                } else {
                    "Not installed"
                }
            ),
        ];

        if let Some(size) = file_size_bytes {
            lines.push(format!("File size: {}", Self::format_file_size(size)));
        }

        lines.push(format!("Path: {}", font.path.display()));
        lines
    }

    fn font_file_size(path: &Path) -> Option<u64> {
        fs::metadata(path).ok().map(|m| m.len())
    }

    fn format_file_size(bytes: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;

        if bytes < 1024 {
            return format!("{} B", bytes);
        }

        let bytes_f = bytes as f64;
        if bytes_f < MB {
            return format!("{:.1} KB", bytes_f / KB);
        }

        format!("{:.1} MB", bytes_f / MB)
    }

    fn process_dropped_paths(&mut self, paths: Vec<PathBuf>) {
        let mut added_count = 0;
        let mut font_paths = Vec::new();
        let mut aggregated_stats = ScanStats::default();
        let mut invalid_paths: Vec<(PathBuf, DropPathKind)> = Vec::new();

        // Process each path - handle both files and folders
        for path in paths {
            match Self::classify_drop_path(&path) {
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
                    // Folder - recursively search for font files
                    self.status = format!("Scanning folder for fonts: {:?}", path);
                    match Self::scan_folder_for_fonts(&path) {
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
                            self.status = format!("Failed to scan folder {:?}: {}", path, e);
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

        let ext_stats = Self::extension_stats(&font_paths);

        // Add all discovered font files
        for font_path in font_paths {
            match self.engine.font_manager().add_font(&font_path) {
                Ok(font_info) => {
                    self.fonts.push(font_info);
                    added_count += 1;
                }
                Err(e) => {
                    self.status = format!("Failed to load font {:?}: {}", font_path, e);
                    return;
                }
            }
        }

        if added_count > 0 {
            self.status = format!("Dropped and added {} font(s)", added_count);
        } else {
            self.status = "No valid font files found".to_string();
        }

        if added_count > 0 {
            self.invalidate_render_cache();
            self.render_previews.clear();
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
            self.status = format!(
                "{} | Skipped {} unsupported and {} missing item(s) ({}){}",
                self.status, unsupported, missing, details, suffix
            );
        }

        if aggregated_stats.files_checked > 0 || aggregated_stats.directories_scanned > 0 {
            self.last_scan_stats = Some(aggregated_stats.clone());
            let sample_preview = if !aggregated_stats.sample_files.is_empty() {
                format!(" e.g. {}", aggregated_stats.sample_files.join(", "))
            } else {
                String::new()
            };

            let ext_summary = Self::format_extension_summary(&ext_stats);

            self.status = format!(
                "Scanned {} folder(s), checked {} file(s), found {} font(s){}{}",
                aggregated_stats.directories_scanned,
                aggregated_stats.files_checked,
                aggregated_stats.fonts_found,
                sample_preview,
                ext_summary
            );
        }
    }

    /// Convert Typf RenderResult (RGBA8) into an iced image handle
    fn image_handle_from_render(render_result: &RenderResult) -> Result<Handle, String> {
        let expected_len = (render_result.width as usize)
            .saturating_mul(render_result.height as usize)
            .saturating_mul(4);
        if expected_len == 0 {
            return Err("Render result contains no pixel data".to_string());
        }

        if render_result.data.len() != expected_len {
            return Err(format!(
                "Pixel data length mismatch (expected {}, got {})",
                expected_len,
                render_result.data.len()
            ));
        }

        // typf returns BitmapFormat::Rgba8; ensure we only render RGBA data
        if !render_result.format.to_lowercase().contains("rgba") {
            return Err(format!(
                "Unsupported render format for preview: {}",
                render_result.format
            ));
        }

        Ok(Handle::from_pixels(
            render_result.width,
            render_result.height,
            render_result.data.clone(),
        ))
    }

    /// Save a render preview to a PNG file on disk
    fn export_preview_to_path(preview: &RenderPreview, path: &Path) -> Result<(), String> {
        if preview.width == 0 || preview.height == 0 {
            return Err("Preview has zero dimensions".to_string());
        }

        if preview.pixels.len().saturating_mul(1)
            != (preview.width as usize)
                .saturating_mul(preview.height as usize)
                .saturating_mul(4)
        {
            return Err("Preview pixel data length is invalid".to_string());
        }

        ::image::save_buffer_with_format(
            path,
            &preview.pixels,
            preview.width,
            preview.height,
            ::image::ColorType::Rgba8,
            ::image::ImageFormat::Png,
        )
        .map_err(|e| format!("Failed to write preview PNG: {}", e))
    }

    /// Export all current render previews into the given folder, returning count written
    fn export_previews_to_folder(&self, folder: &Path) -> Result<usize, String> {
        std::fs::create_dir_all(folder)
            .map_err(|e| format!("Failed to create export folder {:?}: {}", folder, e))?;

        let mut written = 0usize;

        for (i, preview) in self.render_previews.iter().enumerate() {
            let stem = self
                .fonts
                .get(preview.font_index)
                .map(|f| Self::sanitized_file_stem(&f.full_name))
                .unwrap_or_else(|| format!("font{}", i + 1));

            let mut candidate = folder.join(format!("{:02}_{}.png", i + 1, stem));
            let mut suffix = 1;
            while candidate.exists() {
                candidate = folder.join(format!("{:02}_{}_{suffix}.png", i + 1, stem));
                suffix += 1;
            }

            Self::export_preview_to_path(preview, &candidate)?;
            written += 1;
        }

        Ok(written)
    }

    /// Make a filesystem-safe filename stem from user-facing font names
    fn sanitized_file_stem(name: &str) -> String {
        let mut stem: String = name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();

        while stem.contains("__") {
            stem = stem.replace("__", "_");
        }

        stem.trim_matches('_')
            .chars()
            .take(40)
            .collect::<String>()
            .to_lowercase()
    }

    /// Parse a hex RGBA color string into a tuple. Accepts #RRGGBB or #RRGGBBAA.
    fn parse_rgba_hex(value: &str) -> Option<(u8, u8, u8, u8)> {
        let trimmed = value.trim();
        let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
        let len = hex.len();

        if len != 6 && len != 8 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = if len == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()?
        } else {
            255
        };

        Some((r, g, b, a))
    }

    fn invalidate_render_cache(&mut self) {
        self.last_render_settings = None;
        self.last_render_font_paths.clear();
    }

    fn render_cache_hit(&self, font_paths: &[PathBuf]) -> bool {
        Self::should_use_cache(
            &self.last_render_settings,
            &self.last_render_font_paths,
            &self.render_settings,
            font_paths,
            !self.render_previews.is_empty(),
        )
    }

    fn should_use_cache(
        last_settings: &Option<RenderSettings>,
        last_paths: &[PathBuf],
        current_settings: &RenderSettings,
        current_paths: &[PathBuf],
        has_previews: bool,
    ) -> bool {
        if !has_previews {
            return false;
        }

        match last_settings {
            Some(previous) if previous == current_settings => last_paths == current_paths,
            _ => false,
        }
    }

    fn font_matches_filter(&self, font: &FontInfo) -> bool {
        if self.font_filter.trim().is_empty() {
            return true;
        }

        let needle = self.font_filter.to_lowercase();
        let fields = [
            font.full_name.to_lowercase(),
            font.family_name.to_lowercase(),
            font.style.to_lowercase(),
            font.postscript_name.to_lowercase(),
            font.path.display().to_string().to_lowercase(),
        ];

        fields.iter().any(|field| field.contains(&needle))
    }

    fn visible_font_indices(&self) -> Vec<usize> {
        self.fonts
            .iter()
            .enumerate()
            .filter(|(_, font)| self.font_matches_filter(font))
            .map(|(i, _)| i)
            .collect()
    }

    fn derive_render_targets(
        selected: Option<usize>,
        visible_indices: &[usize],
        render_selected_only: bool,
    ) -> Vec<usize> {
        if render_selected_only {
            return selected.into_iter().collect();
        }

        visible_indices.to_vec()
    }

    fn shortcut_to_message(event: &keyboard::Event) -> Option<Message> {
        if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
            if !modifiers.command() {
                return None;
            }

            let key_lower = match key.as_ref() {
                Key::Character(ch) => ch.to_lowercase(),
                Key::Named(keyboard::key::Named::Enter) => "enter".to_string(),
                _ => String::new(),
            };

            match key_lower.as_str() {
                "o" => Some(Message::AddFonts),
                "r" => Some(Message::RenderPreviews),
                "e" => Some(Message::ExportPreviews),
                "w" => Some(Message::OpenRenderWindow),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Update the cached install flag for a font, returning whether an update occurred
    fn set_install_state(fonts: &mut [FontInfo], index: usize, is_installed: bool) -> bool {
        if let Some(font) = fonts.get_mut(index) {
            font.is_installed = is_installed;
            true
        } else {
            false
        }
    }

    /// Re-check installation status using FontLift and update the local cache
    fn refresh_install_status(&mut self, index: usize) {
        if index >= self.fonts.len() {
            return;
        }

        let path = self.fonts[index].path.clone();
        match self.engine.font_manager().is_font_installed(&path) {
            Ok(is_installed) => {
                Self::set_install_state(&mut self.fonts, index, is_installed);
            }
            Err(e) => {
                self.status = format!("Could not verify install status: {}", e);
            }
        }
    }

    /// Get available rendering backends for the current platform
    fn get_available_backends(&self) -> Vec<RendererBackend> {
        self.available_backends.clone()
    }

    /// Get human-readable description for a rendering backend
    fn get_backend_description(&self, backend: &RendererBackend) -> String {
        match backend {
            RendererBackend::Orge => "Opixa (High quality raster)".to_string(),
            RendererBackend::Json => "JSON (debug output)".to_string(),
            #[cfg(target_os = "macos")]
            RendererBackend::CoreGraphics => "CoreGraphics (macOS native)".to_string(),
            #[cfg(feature = "render-skia")]
            RendererBackend::Skia => "Skia (GPU accelerated)".to_string(),
            #[cfg(feature = "render-zeno")]
            RendererBackend::Zeno => "Zeno (experimental vector)".to_string(),
        }
    }

    /// Provide backend-specific capabilities for quick reference
    fn backend_capabilities(&self, backend: &RendererBackend) -> &'static str {
        match backend {
            RendererBackend::Orge => "Opixa: fast raster previews; best default choice",
            RendererBackend::Json => "JSON backend outputs metadata only (hidden in picker)",
            #[cfg(target_os = "macos")]
            RendererBackend::CoreGraphics => "CoreGraphics: macOS native, color-managed previews",
            #[cfg(feature = "render-skia")]
            RendererBackend::Skia => "Skia: GPU-accelerated bitmaps; enable render-skia feature",
            #[cfg(feature = "render-zeno")]
            RendererBackend::Zeno => "Zeno: experimental vector backend",
        }
    }

    fn config_path() -> PathBuf {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("testypf-config.json")
    }

    fn load_config() -> Result<AppConfig, String> {
        let path = Self::config_path();
        Self::load_config_from(&path)
    }

    fn load_config_from(path: &Path) -> Result<AppConfig, String> {
        let contents =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;
        serde_json::from_str(&contents).map_err(|e| format!("Failed to parse config: {}", e))
    }

    fn save_config(config: &AppConfig) -> Result<(), String> {
        let path = Self::config_path();
        Self::save_config_to(&path, config)
    }

    fn save_config_to(path: &Path, config: &AppConfig) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        let contents = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(path, contents).map_err(|e| format!("Failed to write config: {}", e))
    }
}

fn main() -> iced::Result {
    env_logger::init();

    let settings = Settings {
        window: iced::window::Settings {
            size: iced::Size::new(800.0, 600.0),
            min_size: Some(iced::Size::new(600.0, 400.0)),
            ..Default::default()
        },
        ..Default::default()
    };

    TestypfApp::run(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    #[ignore = "requires typf Python module on PYTHONPATH"]
    fn test_app_creation() {
        let flags = ();
        let (app, _command) = TestypfApp::new(flags);
        assert!(app.fonts.is_empty());
        assert_eq!(app.status, "Ready");
    }

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
    #[ignore = "requires typf Python module on PYTHONPATH"]
    fn install_actions_are_blocked_without_platform_support() {
        let flags = ();
        let (mut app, _command) = TestypfApp::new(flags);
        app.font_ops_available = false;
        app.fonts.push(FontInfo {
            path: PathBuf::from("demo.ttf"),
            postscript_name: "DemoPS".into(),
            full_name: "Demo Font".into(),
            family_name: "Demo".into(),
            style: "Regular".into(),
            is_installed: false,
        });

        let _ = app.update(Message::InstallFont(0));

        assert!(
            app.status.contains("Font install unavailable"),
            "status should explain why install is blocked"
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

        let handle = TestypfApp::image_handle_from_render(&render_result)
            .expect("Should build image handle");

        // Handle should be cloneable for display reuse
        let _clone = handle.clone();
    }

    #[test]
    fn set_install_state_updates_flag() {
        let mut fonts = vec![FontInfo {
            path: PathBuf::from("demo.ttf"),
            postscript_name: "DemoPS".into(),
            full_name: "Demo Font".into(),
            family_name: "Demo".into(),
            style: "Regular".into(),
            is_installed: false,
        }];

        let updated = TestypfApp::set_install_state(&mut fonts, 0, true);

        assert!(updated, "should report an update occurred");
        assert!(fonts[0].is_installed, "flag should flip to installed");
    }

    #[test]
    fn set_install_state_out_of_bounds_is_noop() {
        let mut fonts = Vec::<FontInfo>::new();

        let updated = TestypfApp::set_install_state(&mut fonts, 3, true);

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

        let err = TestypfApp::image_handle_from_render(&render_result)
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

        TestypfApp::export_preview_to_path(&preview, &path).expect("export should succeed");

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

        let (fonts, stats) = TestypfApp::scan_folder_for_fonts(&base).expect("scan should succeed");

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

        TestypfApp::save_config_to(&path, &cfg).expect("save config");
        let loaded = TestypfApp::load_config_from(&path).expect("load config");

        assert_eq!(loaded, cfg);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn detects_font_extensions() {
        assert!(TestypfApp::is_font_file(&PathBuf::from("font.TTF")));
        assert!(TestypfApp::is_font_file(&PathBuf::from("font.woff2")));
        assert!(!TestypfApp::is_font_file(&PathBuf::from("font.txt")));
    }

    #[test]
    fn supports_additional_font_formats() {
        assert!(TestypfApp::is_font_file(&PathBuf::from("demo.dfont")));
        assert!(TestypfApp::is_font_file(&PathBuf::from("demo.eot")));
        assert!(TestypfApp::is_font_file(&PathBuf::from("demo.svg")));
        assert!(TestypfApp::is_font_file(&PathBuf::from("demo.pfa")));
        assert!(TestypfApp::is_font_file(&PathBuf::from("demo.pfb")));
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

        let stats = TestypfApp::extension_stats(&paths);
        assert_eq!(stats.get("ttf"), Some(&1));
        assert_eq!(stats.get("otf"), Some(&2));
        assert_eq!(stats.get("woff2"), Some(&1));
        assert_eq!(stats.get("woff"), Some(&1));
        assert!(!stats.contains_key("txt"));
    }

    #[test]
    fn parses_hex_colors() {
        assert_eq!(
            TestypfApp::parse_rgba_hex("#112233"),
            Some((0x11, 0x22, 0x33, 0xFF))
        );
        assert_eq!(
            TestypfApp::parse_rgba_hex("44556677"),
            Some((0x44, 0x55, 0x66, 0x77))
        );
        assert_eq!(TestypfApp::parse_rgba_hex("12"), None);
        assert_eq!(TestypfApp::parse_rgba_hex("GGHHII"), None);
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
            !TestypfApp::should_use_cache(&None, &[], &settings, &fonts, true),
            "No previous settings means no cache hit"
        );
        assert!(
            !TestypfApp::should_use_cache(
                &Some(settings.clone()),
                &fonts,
                &settings,
                &fonts,
                false
            ),
            "No previews means no cache hit"
        );
        assert!(
            TestypfApp::should_use_cache(&Some(settings.clone()), &fonts, &settings, &fonts, true),
            "Matching settings and fonts with previews should hit cache"
        );
        assert!(
            !TestypfApp::should_use_cache(&Some(other_settings), &fonts, &settings, &fonts, true),
            "Changed settings invalidates cache"
        );
        assert!(
            !TestypfApp::should_use_cache(
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
            TestypfApp::classify_drop_path(&font_path),
            DropPathKind::FontFile
        );
        assert_eq!(
            TestypfApp::classify_drop_path(&nested_dir),
            DropPathKind::Directory
        );
        assert_eq!(
            TestypfApp::classify_drop_path(&text_path),
            DropPathKind::Unsupported
        );
        let missing = nested_dir.join("missing.otf");
        assert_eq!(
            TestypfApp::classify_drop_path(&missing),
            DropPathKind::Missing
        );

        fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn layout_row_count_matches_layout_mode() {
        assert_eq!(TestypfApp::layout_row_count(0, LayoutMode::Single), 0);
        assert_eq!(TestypfApp::layout_row_count(3, LayoutMode::Single), 3);
        assert_eq!(TestypfApp::layout_row_count(0, LayoutMode::SideBySide), 0);
        assert_eq!(TestypfApp::layout_row_count(1, LayoutMode::SideBySide), 1);
        assert_eq!(TestypfApp::layout_row_count(2, LayoutMode::SideBySide), 1);
        assert_eq!(TestypfApp::layout_row_count(3, LayoutMode::SideBySide), 2);
        assert_eq!(TestypfApp::layout_row_count(4, LayoutMode::SideBySide), 2);
    }

    #[test]
    fn format_file_size_scales_units() {
        assert_eq!(TestypfApp::format_file_size(0), "0 B");
        assert_eq!(TestypfApp::format_file_size(532), "532 B");
        assert_eq!(TestypfApp::format_file_size(1536), "1.5 KB");
        assert_eq!(TestypfApp::format_file_size(5_242_880), "5.0 MB");
    }

    #[test]
    fn preview_metadata_text_includes_duration_and_backend() {
        let font = FontInfo {
            path: PathBuf::from("demo.ttf"),
            postscript_name: "DemoPS".into(),
            full_name: "Demo Font".into(),
            family_name: "Demo".into(),
            style: "Regular".into(),
            is_installed: true,
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

        let text = TestypfApp::preview_metadata_text(&preview, &font, &RenderSettings::default());

        assert!(text.contains("12 ms"), "render duration should be shown");
        assert!(
            text.contains(&RenderSettings::default().backend.to_string()),
            "backend label should be present"
        );
    }

    #[test]
    fn font_metadata_lines_include_path_and_install_state() {
        let font = FontInfo {
            path: PathBuf::from("/tmp/metadata/demo.ttf"),
            postscript_name: "DemoPS".into(),
            full_name: "Demo Font".into(),
            family_name: "Demo".into(),
            style: "Bold".into(),
            is_installed: false,
        };

        let lines = TestypfApp::font_metadata_lines(&font, Some(2048));

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
            TestypfApp::derive_render_targets(Some(2), &visible, true),
            vec![2]
        );
        assert!(
            TestypfApp::derive_render_targets(None, &visible, true).is_empty(),
            "selection required when rendering selected only"
        );
        assert_eq!(
            TestypfApp::derive_render_targets(None, &visible, false),
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
            TestypfApp::shortcut_to_message(&render),
            Some(Message::RenderPreviews)
        ));

        let export = keyboard::Event::KeyPressed {
            key: keyboard::Key::Character("e".into()),
            location: keyboard::Location::Standard,
            modifiers: command,
            text: None,
        };
        assert!(matches!(
            TestypfApp::shortcut_to_message(&export),
            Some(Message::ExportPreviews)
        ));

        let ignore = keyboard::Event::KeyPressed {
            key: keyboard::Key::Character("r".into()),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::empty(),
            text: None,
        };
        assert!(
            TestypfApp::shortcut_to_message(&ignore).is_none(),
            "shortcuts should require command modifier"
        );
    }
}

/// Custom container style for active drag state
struct DragActiveStyle;

impl iced::widget::container::StyleSheet for DragActiveStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
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
