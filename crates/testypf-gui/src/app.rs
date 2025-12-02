//! Application state and lifecycle for testypf GUI.

use crate::helpers;
use crate::message::Message;
use crate::types::{InstallScope, LayoutMode, RenderPreview, ScanStats};
use crate::update;
use crate::view;

use iced::{
    event,
    multi_window::{self, Application},
    window, Command, Element, Event, Settings, Subscription, Theme,
};
use std::path::PathBuf;
use testypf_core::{FontInfo, RenderSettings, RendererBackend, TestypfEngine};

/// Main application state.
pub struct TestypfApp {
    pub engine: TestypfEngine,
    pub fonts: Vec<FontInfo>,
    pub render_settings: RenderSettings,
    pub status: String,
    pub render_previews: Vec<RenderPreview>,
    pub is_dragging: bool,
    pub hovered_file: Option<PathBuf>,
    pub available_backends: Vec<RendererBackend>,
    pub last_scan_stats: Option<ScanStats>,
    pub pending_drop_paths: Vec<PathBuf>,
    pub drop_processing_scheduled: bool,
    pub font_ops_available: bool,
    pub render_window_id: Option<window::Id>,
    pub last_render_settings: Option<RenderSettings>,
    pub last_render_font_paths: Vec<PathBuf>,
    pub foreground_input: String,
    pub background_input: String,
    pub background_enabled: bool,
    pub layout_mode: LayoutMode,
    pub install_scope: InstallScope,
    pub selected_font: Option<usize>,
    pub font_filter: String,
    pub render_selected_only: bool,
}

impl multi_window::Application for TestypfApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut engine = TestypfEngine::new().expect("Failed to initialize testypf engine");
        let mut render_settings = RenderSettings::default();
        let mut status = "Ready".to_string();

        // Detect backends and filter out JSON (non-visual)
        let available_backends = engine
            .text_renderer()
            .get_backends()
            .into_iter()
            .filter(|backend| !matches!(backend, RendererBackend::Json))
            .collect::<Vec<_>>();

        let font_ops_available = TestypfEngine::font_ops_available();

        // Load persisted configuration if present
        if let Ok(config) = helpers::load_config() {
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
            Event::Window(id, iced::window::Event::FileHovered(path))
                if id == window::Id::MAIN =>
            {
                Some(Message::FileHovered(path))
            }
            Event::Window(id, iced::window::Event::FileDropped(path))
                if id == window::Id::MAIN =>
            {
                Some(Message::FilesDropped(vec![path]))
            }
            Event::Window(id, iced::window::Event::FilesHoveredLeft)
                if id == window::Id::MAIN =>
            {
                Some(Message::DragLeave)
            }
            Event::Window(id, iced::window::Event::Closed) => Some(Message::WindowClosed(id)),
            Event::Window(id, iced::window::Event::CloseRequested) => {
                Some(Message::WindowClosed(id))
            }
            Event::Keyboard(key_event) => helpers::shortcut_to_message(&key_event),
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
        update::handle_message(self, message)
    }

    fn view(&self, window: window::Id) -> Element<'_, Message> {
        view::render(self, window)
    }

    fn theme(&self, _window: window::Id) -> Theme {
        Theme::Light
    }
}

impl TestypfApp {
    /// Spawn the transparent render window if needed.
    pub fn ensure_render_window(&mut self) -> Command<Message> {
        if let Some(id) = self.render_window_id {
            return window::gain_focus(id);
        }

        let settings = Self::render_window_settings();
        let (id, cmd) = window::spawn(settings);
        self.render_window_id = Some(id);
        cmd
    }

    /// Transparent render window configuration.
    pub fn render_window_settings() -> window::Settings {
        let mut settings = window::Settings::default();
        settings.size = iced::Size::new(900.0, 650.0);
        settings.min_size = Some(iced::Size::new(640.0, 480.0));
        settings.decorations = false;
        settings.transparent = true;
        settings.level = window::Level::AlwaysOnTop;
        settings
    }

    /// Invalidate render cache.
    pub fn invalidate_render_cache(&mut self) {
        self.last_render_settings = None;
        self.last_render_font_paths.clear();
    }

    /// Check if render cache is valid.
    pub fn render_cache_hit(&self, font_paths: &[PathBuf]) -> bool {
        helpers::should_use_cache(
            &self.last_render_settings,
            &self.last_render_font_paths,
            &self.render_settings,
            font_paths,
            !self.render_previews.is_empty(),
        )
    }

    /// Check if a font matches the current filter.
    pub fn font_matches_filter(&self, font: &FontInfo) -> bool {
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

    /// Get indices of fonts matching the current filter.
    pub fn visible_font_indices(&self) -> Vec<usize> {
        self.fonts
            .iter()
            .enumerate()
            .filter(|(_, font)| self.font_matches_filter(font))
            .map(|(i, _)| i)
            .collect()
    }

    /// Get available rendering backends.
    pub fn get_available_backends(&self) -> Vec<RendererBackend> {
        self.available_backends.clone()
    }

    /// Get human-readable description for a backend.
    pub fn get_backend_description(&self, backend: &RendererBackend) -> String {
        helpers::get_backend_description(backend)
    }

    /// Get backend capabilities text.
    pub fn backend_capabilities(&self, backend: &RendererBackend) -> &'static str {
        helpers::backend_capabilities(backend)
    }

    /// Re-check installation status for a font.
    pub fn refresh_install_status(&mut self, index: usize) {
        if index >= self.fonts.len() {
            return;
        }

        let path = self.fonts[index].path.clone();
        match self.engine.font_manager().is_font_installed(&path) {
            Ok(is_installed) => {
                helpers::set_install_state(&mut self.fonts, index, is_installed);
            }
            Err(e) => {
                self.status = format!("Could not verify install status: {}", e);
            }
        }
    }
}

/// Run the application.
pub fn run() -> iced::Result {
    pyo3::prepare_freethreaded_python();
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
