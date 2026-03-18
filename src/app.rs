use crate::{favorites, thumbnails, videoutils};
use anyhow::{Result, anyhow};
use eframe::egui::{self, Color32, Key, KeyboardShortcut, Modifiers};
use egui_file_dialog::FileDialog;
use egui_notify::Toasts;
use glob::{MatchOptions, glob_with};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::mpsc,
    time::Duration,
};

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
enum Sorting {
    CreationDate,
    ModificationDate,
    Name,
    Size,
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
enum DisplayMode {
    List,
    Grid,
}

#[derive(PartialEq, Debug, serde::Deserialize, serde::Serialize)]
enum LitterboxUploadTime {
    OneHour,
    TwelveHours,
    OneDay,
    ThreeDays,
}
#[derive(Default, serde::Deserialize, serde::Serialize)]
enum CatboxUploadState {
    #[default]
    Idle,
    Uploading,
    Done(String),
    Error(String),
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
enum DefaultFileAction {
    View,
    Edit,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ReplayManager {
    replay_folder: Option<PathBuf>,
    replay_format: String,
    replay_prefix: String,
    display_mode: DisplayMode,
    default_file_action: DefaultFileAction,

    #[serde(skip)]
    delete_popup: Option<usize>,
    #[serde(skip)]
    catbox_popup: Option<usize>,
    replays: Vec<PathBuf>,

    #[serde(skip)]
    loading_done: bool,
    sort_order: Sorting,
    ascending: bool,

    #[serde(skip)]
    search_query: String,
    #[serde(skip)]
    catbox_upload_state: CatboxUploadState,
    #[serde(skip)]
    catbox_upload_recv: mpsc::Receiver<Result<String, String>>,
    #[serde(skip)]
    catbox_upload_send: mpsc::Sender<Result<String, String>>,
    #[serde(skip)]
    settings_popup: bool,

    #[serde(skip)]
    file_dialog: FileDialog,

    litterbox_upload_time: LitterboxUploadTime,
    catbox_litter: bool,

    #[serde(skip)]
    toasts: Toasts,

    #[serde(skip)]
    thumb_recv: mpsc::Receiver<(PathBuf, Result<PathBuf>)>,
    #[serde(skip)]
    thumb_send: mpsc::Sender<(PathBuf, Result<PathBuf>)>,
    #[serde(skip)]
    thumb_queue: HashSet<PathBuf>,
    thumb_cache: HashMap<PathBuf, PathBuf>,
    favorites_cache: HashMap<PathBuf, PathBuf>,
    #[serde(skip)]
    thumb_errors: HashSet<PathBuf>,

    video_editor: String,
    #[serde(skip)]
    refresh: bool,

    show_hidden_files: bool,

    settings_shortcut: KeyboardShortcut,
    edit_shortcut: KeyboardShortcut,
    view_shortcut: KeyboardShortcut,
    delete_shortcut: KeyboardShortcut,
    catbox_shortcut: KeyboardShortcut,
    search_shortcut: KeyboardShortcut,
    refresh_shortcut: KeyboardShortcut,

    find_recursively: bool,
    #[serde(skip)]
    favorites_mode: bool,
}

impl Default for ReplayManager {
    fn default() -> Self {
        let (catbox_tx, catbox_rx) = mpsc::channel();
        let (thumb_tx, thumb_rx) = mpsc::channel();
        Self {
            replay_folder: dirs::video_dir(),
            replay_format: "mp4".to_string(),
            replay_prefix: "".to_string(),
            delete_popup: None,
            catbox_popup: None,
            replays: vec![],
            loading_done: false,
            sort_order: Sorting::ModificationDate,
            ascending: false,
            search_query: "".to_string(),
            catbox_upload_state: CatboxUploadState::Idle,
            catbox_upload_send: catbox_tx,
            catbox_upload_recv: catbox_rx,
            settings_popup: false,
            file_dialog: FileDialog::new(),
            litterbox_upload_time: LitterboxUploadTime::ThreeDays,
            catbox_litter: true,
            toasts: Toasts::default(),
            display_mode: DisplayMode::Grid,
            video_editor: if cfg!(target_os = "windows") {
                String::from("LosslessCut")
            } else {
                String::from("losslesscut")
            },
            default_file_action: DefaultFileAction::View,
            thumb_recv: thumb_rx,
            thumb_send: thumb_tx,
            thumb_queue: HashSet::new(),
            thumb_cache: HashMap::new(),
            thumb_errors: HashSet::new(),
            refresh: true,
            show_hidden_files: false,
            settings_shortcut: KeyboardShortcut::new(Modifiers::ALT, Key::S),
            edit_shortcut: KeyboardShortcut::new(Modifiers::SHIFT, Key::E),
            view_shortcut: KeyboardShortcut::new(Modifiers::SHIFT, Key::O),
            delete_shortcut: KeyboardShortcut::new(Modifiers::NONE, Key::Delete),
            catbox_shortcut: KeyboardShortcut::new(Modifiers::CTRL, Key::S),
            search_shortcut: KeyboardShortcut::new(Modifiers::CTRL, Key::F),
            refresh_shortcut: KeyboardShortcut::new(Modifiers::CTRL, Key::R),
            find_recursively: false,
            favorites_cache: HashMap::new(),
            favorites_mode: false,
        }
    }
}

impl ReplayManager {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_material_icons::initialize(&cc.egui_ctx);
        let mut repman: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        repman
            .thumb_cache
            .retain(|_video_path, thumb_path| thumb_path.exists());
        repman
            .thumb_cache
            .retain(|video_path, _thumb_path| video_path.exists());

        repman.thumb_queue = repman.thumb_cache.keys().cloned().collect();

        repman.file_dialog = repman
            .file_dialog
            .initial_directory(repman.replay_folder.clone().unwrap_or_default());

        repman
    }
}

impl eframe::App for ReplayManager {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok((replay_path, result)) = self.thumb_recv.try_recv() {
            match result {
                Ok(thumb_path) => {
                    self.thumb_cache.insert(replay_path, thumb_path);
                }
                Err(err) => {
                    self.toasts
                        .error(format!(
                            "Could not get thumbnail for {}",
                            replay_path.display()
                        ))
                        .duration(Duration::from_secs(5));
                    eprintln!("Could not get thumbnail: {}", err);
                    self.thumb_errors.insert(replay_path);
                }
            }
            ctx.request_repaint();
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.settings_shortcut)) {
            self.settings_popup = !self.settings_popup
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.refresh_shortcut)) {
            self.refresh = true
        }

        ctx.request_repaint_after(Duration::from_millis(100));
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui
                        .add(
                            egui::Button::new("Settings").shortcut_text(
                                self.settings_shortcut
                                    .format(&egui::ModifierNames::NAMES, cfg!(target_os = "macos")),
                            ),
                        )
                        .clicked()
                    {
                        self.settings_popup = true;
                    }
                    ui.menu_button("Theme...", |ui| {
                        let mut theme_preference = ui.ctx().options(|opt| opt.theme_preference);
                        ui.radio_value(
                            &mut theme_preference,
                            egui::ThemePreference::System,
                            "System",
                        );
                        ui.radio_value(
                            &mut theme_preference,
                            egui::ThemePreference::Light,
                            "Light",
                        );
                        ui.radio_value(&mut theme_preference, egui::ThemePreference::Dark, "Dark");
                        ui.ctx().set_theme(theme_preference);
                    });
                });
                ui.menu_button("View", |ui| {
                    if ui
                        .add(
                            egui::Button::new("Refresh").shortcut_text(
                                self.refresh_shortcut
                                    .format(&egui::ModifierNames::NAMES, cfg!(target_os = "macos")),
                            ),
                        )
                        .clicked()
                    {
                        self.refresh = true;
                    };
                    ui.menu_button("Sort...", |ui| {
                        if ui
                            .radio_value(
                                &mut self.sort_order,
                                Sorting::CreationDate,
                                "Creation Date",
                            )
                            .changed()
                        {
                            self.refresh = true;
                        };
                        if ui
                            .radio_value(
                                &mut self.sort_order,
                                Sorting::ModificationDate,
                                "Modification Date",
                            )
                            .changed()
                        {
                            self.refresh = true;
                        };
                        if ui
                            .radio_value(&mut self.sort_order, Sorting::Name, "Name")
                            .changed()
                        {
                            self.refresh = true;
                        };
                        if ui
                            .radio_value(&mut self.sort_order, Sorting::Size, "File Size")
                            .changed()
                        {
                            self.refresh = true;
                        };
                    });
                    ui.menu_button("Display...", |ui| {
                        ui.radio_value(&mut self.display_mode, DisplayMode::Grid, "Grid");
                        ui.radio_value(&mut self.display_mode, DisplayMode::List, "List");
                    });
                    ui.menu_button("Order...", |ui| {
                        if ui.radio(self.ascending, "Ascending").clicked() {
                            self.ascending = true;
                            self.refresh = true;
                        }
                        if ui.radio(!self.ascending, "Descending").clicked() {
                            self.ascending = false;
                            self.refresh = true;
                        };
                    });
                    ui.checkbox(&mut self.show_hidden_files, "Show hidden files");
                });
                ui.add_space(8.0);
                if ui
                    .small_button(format!(
                        "{} All",
                        egui_material_icons::icons::ICON_COLLECTIONS
                    ))
                    .clicked()
                {
                    if self.favorites_mode {
                        self.favorites_mode = false;
                        self.refresh = true;
                    }
                }
                if ui
                    .small_button(format!(
                        "{} Favorites",
                        egui_material_icons::icons::ICON_FAVORITE_BORDER
                    ))
                    .clicked()
                {
                    if !self.favorites_mode {
                        self.favorites_mode = true;
                        self.refresh = true;
                    }
                }
            });
            if self.settings_popup {
                let _window = egui::Window::new("Settings")
                    .collapsible(false)
                    .open(&mut self.settings_popup)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_max_width(310.0);
                        ui.heading("Replay settings");
                        ui.horizontal(|ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                "Replay videos folder location (default: $HOME/Videos/Replays/):",
                            );
                            /*ui.strong(format!(
                                "{}",
                                self.replay_folder
                                    .unwrap_or(PathBuf::from("error"))
                                    .display()
                            ));*/
                            if ui.button("Choose").clicked() {
                                self.file_dialog.pick_directory();
                            }
                            self.file_dialog.update(ctx);
                            if let Some(path) = self.file_dialog.take_picked() {
                                self.replay_folder = Some(path.to_path_buf());
                                self.refresh = true;
                                self.thumb_queue.clear();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label("Replay prefix (default: Replay_):");
                            if ui.text_edit_singleline(&mut self.replay_prefix).changed() {
                                self.refresh = true;
                            };
                        });
                        ui.horizontal(|ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label("Replay format (default: mp4):");
                            if ui.text_edit_singleline(&mut self.replay_format).changed() {
                                self.refresh = true;
                            };
                        });
                        if ui
                            .checkbox(
                                &mut self.find_recursively,
                                "Loop recursively through subfolders",
                            )
                            .changed()
                        {
                            self.refresh = true
                        };

                        ui.heading("Catbox");
                        ui.checkbox(&mut self.catbox_litter, "Use litterbox");

                        let _dropdown = egui::ComboBox::from_label("Litterbox file deletion time")
                            .selected_text(format!("{:?}", self.litterbox_upload_time))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.litterbox_upload_time,
                                    LitterboxUploadTime::OneHour,
                                    "One hour",
                                );
                                ui.selectable_value(
                                    &mut self.litterbox_upload_time,
                                    LitterboxUploadTime::TwelveHours,
                                    "Twelve hours",
                                );
                                ui.selectable_value(
                                    &mut self.litterbox_upload_time,
                                    LitterboxUploadTime::OneDay,
                                    "A day",
                                );
                                ui.selectable_value(
                                    &mut self.litterbox_upload_time,
                                    LitterboxUploadTime::ThreeDays,
                                    "Three days",
                                );
                            });
                        ui.heading("Programs");
                        ui.horizontal(|ui| {
                            ui.label("Video editor (default losslesscut):");
                            ui.text_edit_singleline(&mut self.video_editor);
                        });
                        let _dropdown = egui::ComboBox::from_label("Default file action")
                            .selected_text(format!("{:?}", self.default_file_action))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.default_file_action,
                                    DefaultFileAction::View,
                                    "View",
                                );
                                ui.selectable_value(
                                    &mut self.default_file_action,
                                    DefaultFileAction::Edit,
                                    "Edit",
                                );
                            });
                    });
            };
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Replay Manager");
            let replay_folder = self.replay_folder.clone().unwrap();

            ui.horizontal(|ui| {
                ui.label(format!("{}eplays in {}", if self.favorites_mode { "Favorite r" } else { "R" }, replay_folder.display()));
            });


            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.small(self.search_shortcut.format(&egui::ModifierNames::NAMES, cfg!(target_os = "macos")));
                    ui.label("Search:");
                });
                let search_text_input_response = ui.add(egui::TextEdit::singleline(&mut self.search_query).desired_width(ui.available_width()));
                let search_text_input_rect = &search_text_input_response.rect;

                let mut draw_clear_button = || {
                    let height = search_text_input_rect.height();
                    let mut min = search_text_input_rect.min;
                    let max = search_text_input_rect.max;
                    min = egui::pos2(max.x - height, min.y);
                    let new_rect = egui::Rect::from_min_max(min, max);
                    let show_button = ui
                        .put(
                            new_rect,
                            egui::Button::new(egui_material_icons::icons::ICON_CLEAR).frame(false)
                        ).on_hover_cursor(egui::CursorIcon::PointingHand);
                    if show_button.clicked() {
                        self.search_query = "".to_string();
                        search_text_input_response.request_focus();
                    }
                };

                draw_clear_button();

                if ctx.input_mut(|i| i.consume_shortcut(&self.search_shortcut)) {
                    search_text_input_response.request_focus();
                }
            });

            ui.separator();

            if self.replay_folder.is_none() {
                self.toasts.error("Replay folder does not exists (is None)").duration(Duration::from_secs(5));
            }

            let replays_pattern: String;

            if self.favorites_mode {
                replays_pattern = format!(
                    "{}/*", favorites::check_subdirectory(self.replay_folder.as_deref()).expect("Could not get path").to_string_lossy()
                );
            } else {
                replays_pattern = format!(
                    "{}/{}{}*{}",
                    replay_folder.to_string_lossy(),
                    if self.find_recursively { "**/" } else { "" },
                    self.replay_prefix,
                    self.replay_format
                );
            }

            let replays_glob_options = MatchOptions {

                require_literal_leading_dot: !self.show_hidden_files,
                ..Default::default()
            };

            if self.refresh {
                let replays_glob = glob_with(&replays_pattern, replays_glob_options);

                if let Ok(replay_paths) = replays_glob {
                    self.replays = replay_paths.filter_map(|e| e.ok()).collect();
                } else if let Err(err) = replays_glob {
                    self.toasts.error(format!("Could not get replays: {}", err)).duration(Duration::from_secs(5));
                }

                match self.sort_order {
                    Sorting::CreationDate => self.replays.sort_by(|a, b| {
                        videoutils::get_creation_date(a)
                            .into_iter()
                            .cmp(videoutils::get_creation_date(b))
                    }),
                    Sorting::Name => self.replays.sort_by(|a, b| {
                        videoutils::get_name(a)
                            .into_iter()
                            .cmp(&mut videoutils::get_name(b).into_iter())
                    }),
                    Sorting::ModificationDate => self.replays.sort_by(|a, b| {
                        videoutils::get_mod_date(a)
                            .into_iter()
                            .cmp(videoutils::get_mod_date(b))
                    }),
                    Sorting::Size => self.replays.sort_by(|a, b| {
                        videoutils::get_size(a)
                            .into_iter()
                            .cmp(&mut videoutils::get_size(b).into_iter())
                    }),
                }

                if !self.ascending {
                    self.replays.reverse()
                }

                self.refresh = false;
                self.loading_done = false;
            }


            if let Ok(result) = self.catbox_upload_recv.try_recv() {
                self.catbox_upload_state = match result {
                    Ok(link) => CatboxUploadState::Done(link),
                    Err(err) => CatboxUploadState::Error(err),
                };
                ctx.request_repaint();
            }

            let replay_count = self.replays.len();
            let replay_enumerate = self.replays.iter().enumerate();
            let min_col_width = 160.0;
            let min_col_height = match self.display_mode {
                DisplayMode::Grid => 120.0,
                DisplayMode::List => 60.0,
            };
            let grid_spacing = egui::Vec2::new(
                // ui.available_width() - column_count as f32 * min_col_width,
                10.0, 10.0,
            );
            let ui_minus_spacing = ui.available_width(); //- 2.0*grid_spacing.x - ui.style().spacing.window_margin.left as f32 - ui.style().spacing.window_margin.right as f32 / min_col_width;
            let column_count = match self.display_mode {
                DisplayMode::Grid => (ui_minus_spacing / min_col_width).floor() as usize,
                DisplayMode::List => 1usize,
            };


            let image_size = match self.display_mode {
                DisplayMode::Grid => egui::Vec2::new(160.0, 120.0),
                DisplayMode::List => egui::Vec2::new(80.0, 60.0),
            };

            ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::DARK_GRAY;
            ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::LIGHT_BLUE;
            ui.style_mut().spacing.item_spacing = egui::Vec2::new(8.0, 8.0);

            let replays_filtered: Vec<(usize, &PathBuf)> = replay_enumerate
                .filter(|(_, entry)| {
                    entry
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_lowercase().contains(&self.search_query.to_lowercase()))
                        .unwrap_or(false)
                })
                .collect();

            let total_rows = (replays_filtered.len() + column_count - 1).max(1) / column_count;
            let row_height = image_size.y + 24.0 + grid_spacing.y; // label height is 24

            egui::ScrollArea::both()
                .max_width(ui.available_width())
                .show_rows(ui, row_height, total_rows, |ui, row_range| {
                    ui.set_width(ui.available_width());
                    egui::Grid::new("Replays")
                        .min_col_width(min_col_width)
                        .min_row_height(min_col_height)
                        .num_columns(column_count)
                        .spacing(grid_spacing)
                        .show(ui, |ui| {
                            for row in row_range {
                                for col in 0..column_count {
                                    let idx = row * column_count + col;
                                    let (i, entry) = match replays_filtered.get(idx) {
                                        Some(e) => e,
                                        None => break,
                                    };
                                    let (i, entry) = (*i, *entry);
                                if !self.thumb_queue.contains(entry) && !self.thumb_cache.contains_key(entry) {
                                    self.thumb_queue.insert(entry.clone());

                                    let entry = entry.clone();
                                    let folder_display = format!("{}", replay_folder.display());
                                    let tx = self.thumb_send.clone();

                                    rayon::spawn(move || {
                                        let result = thumbnails::create(
                                            &entry,
                                            &folder_display,
                                            true,
                                            0.0,
                                        );
                                        let _ = tx.send((entry, result));
                                    });
                                }
                                if self.thumb_errors.contains(entry) {
                                    ui.vertical(|ui| {
                                        egui::Frame::default()
                                            .fill(Color32::DARK_RED)
                                            .corner_radius(5.0)
                                            .show(ui, |ui| {
                                                ui.set_width(image_size.x);
                                                ui.set_height(image_size.y);
                                                ui.centered_and_justified(|ui| {
                                                    ui.colored_label(Color32::WHITE, "Could not load thumbnail")
                                                });
                                            });
                                        ui.label(entry.to_string_lossy());
                                    });
                                    continue;
                                }

                                if !self.thumb_cache.contains_key(entry) {
                                    ui.vertical(|ui| {
                                        egui::Frame::default()
                                            .fill(Color32::DARK_GRAY)
                                            .corner_radius(5.0)
                                            .show(ui, |ui| {
                                                ui.set_width(image_size.x);
                                                ui.set_height(image_size.y);
                                                ui.centered_and_justified(|ui| {
                                                    ui.spinner();
                                                });
                                            });
                                        ui.label(entry.to_string_lossy());
                                    });
                                    continue;
                                }

                                let thumbnail_path = self.thumb_cache.get(entry).cloned().unwrap_or_default();

                                let thumbnail_image =
                                    egui::Image::from_uri(format!(
                                        "file://{}",
                                        thumbnail_path.display()
                                    ))
                                    .fit_to_exact_size(image_size) // original res 640x480
                                    .corner_radius(5);

                                let file_stem_opt = entry.file_stem();
                                let file_stem: &std::ffi::OsStr;
                                if let Some(stem) = file_stem_opt {
                                    file_stem = stem;
                                } else {
                                    self.toasts.error("File stem is None").duration(Duration::from_secs(5));
                                    file_stem = std::ffi::OsStr::new("undefined");
                                }

                                {
                                    let button = match self.display_mode {
                                        DisplayMode::Grid => egui::Button::image(thumbnail_image)
                                            .min_size(image_size)
                                            .frame_when_inactive(false),
                                        DisplayMode::List => egui::Button::image_and_text(
                                            thumbnail_image,
                                            file_stem.to_string_lossy(),
                                        )
                                        .min_size(egui::Vec2::new(
                                            ui.available_width(),
                                            image_size.y,
                                        ))
                                        .frame_when_inactive(false),
                                    };
                                    let label = egui::Label::new(format!(
                                        "{}",
                                        file_stem.to_string_lossy(),
                                    ))
                                    .halign(egui::Align::Center);

                                    let button_ui =
                                        |ui: &mut egui::Ui| -> Result<()> {
                                            let button_response = ui.add(button);
                                            if self.display_mode == DisplayMode::Grid {
                                                ui.add(label);
                                            }
                                            let entry_path = entry.to_path_buf();
                                            let editor = self.video_editor.clone();
                                            let catbox_litter = self.catbox_litter;
                                            let litter_delete_time = match self.litterbox_upload_time {
                                                LitterboxUploadTime::OneHour => 1,
                                                LitterboxUploadTime::TwelveHours => 12,
                                                LitterboxUploadTime::OneDay => 24,
                                                LitterboxUploadTime::ThreeDays => 72,
                                            };
                                            let open_editor = || {
                                                let entry_path = entry_path.clone();
                                                let editor = editor.clone();
                                                std::thread::spawn(move || {
                                                    if let Err(e) = open::with(&entry_path, editor) {
                                                        eprintln!("Failed to open editor: {}", e);
                                                    }
                                                });
                                            };
                                            let open_view = || {
                                                let entry_path = entry_path.clone();
                                                std::thread::spawn(move || {
                                                    if let Err(e) = open::that(&entry_path) {
                                                        eprintln!("Failed to open view: {}", e);
                                                    }
                                                });
                                            };
                                            let catbox_tx = self.catbox_upload_send.clone();
                                            let catbox_ctx = ctx.clone();
                                            let upload_catbox = || {
                                                let entry_path = entry_path.clone();
                                                let catbox_tx = catbox_tx.clone();
                                                let catbox_ctx = catbox_ctx.clone();
                                                std::thread::spawn(move|| {
                                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                                    let result: Result<String, Box<dyn std::error::Error>> = rt.block_on(async move {
                                                        if catbox_litter {
                                                            catbox::litter::upload(entry_path.to_string_lossy(), litter_delete_time).await
                                                        } else {
                                                            catbox::file::from_file(entry_path.to_string_lossy(), None).await
                                                        }
                                                    });
                                                    let _ = catbox_tx.send(result.map_err(|e| e.to_string()));
                                                    catbox_ctx.request_repaint();
                                                });
                                            };
                                            button_response.context_menu(|ui| {
                                                let edit_button = egui::Button::new("Edit").shortcut_text(self.edit_shortcut.format(&egui::ModifierNames::NAMES, cfg!(target_os = "macos")));
                                                let view_button = egui::Button::new("View").shortcut_text(self.view_shortcut.format(&egui::ModifierNames::NAMES, cfg!(target_os = "macos")));
                                                let delete_button = egui::Button::new("Delete").shortcut_text(self.delete_shortcut.format(&egui::ModifierNames::NAMES, cfg!(target_os = "macos")));
                                                let catbox_button = egui::Button::new("Save to Catbox").shortcut_text(self.catbox_shortcut.format(&egui::ModifierNames::NAMES, cfg!(target_os = "macos")));
                                                let edit_button_response = ui.add(edit_button);
                                                let view_button_response = ui.add(view_button);
                                                let delete_button_response = ui.add(delete_button);
                                                let catbox_button_response = ui.add(catbox_button);
                                                if edit_button_response.clicked() {
                                                    open_editor();
                                                    self.toasts.success(format!("Opened {}", self.video_editor)).duration(Duration::from_secs(5));
                                                }
                                                if view_button_response.clicked() {
                                                    open_view();
                                                    self.toasts.success("Opened media viewer").duration(Duration::from_secs(5));
                                                }
                                                if delete_button_response.clicked() {
                                                    self.delete_popup = Some(i);
                                                }
                                                if catbox_button_response.clicked() {
                                                    upload_catbox();
                                                    self.catbox_upload_state = CatboxUploadState::Uploading;
                                                    self.catbox_popup = Some(i);
                                                }
                                            });

                                            if button_response.has_focus() && ctx.input_mut(|i| i.consume_shortcut(&self.edit_shortcut)) {
                                                open_editor();
                                                self.toasts.success(format!("Opened {}", self.video_editor)).duration(Duration::from_secs(5));
                                            }
                                            if button_response.has_focus() && ctx.input_mut(|i| i.consume_shortcut(&self.view_shortcut)) {
                                                open_view();
                                                self.toasts.success("Opened media viewer").duration(Duration::from_secs(5));
                                            }
                                            if button_response.has_focus() && ctx.input_mut(|i| i.consume_shortcut(&self.delete_shortcut)) {
                                                self.delete_popup = Some(i);
                                            }
                                            if button_response.has_focus() && ctx.input_mut(|i| i.consume_shortcut(&self.catbox_shortcut)) {
                                                upload_catbox();
                                                self.catbox_upload_state = CatboxUploadState::Uploading;
                                                self.catbox_popup = Some(i);
                                            }



                                            if button_response.double_clicked() {
                                                let entry_path = entry.to_path_buf();
                                                match self.default_file_action {
                                                    DefaultFileAction::View => {
                                                        std::thread::spawn(move || {
                                                            if let Err(e) = open::that(&entry_path) {
                                                                eprintln!("Failed to open file: {}", e);
                                                            };
                                                        });
                                                    }
                                                    DefaultFileAction::Edit => {
                                                        let editor = self.video_editor.clone();
                                                        std::thread::spawn(move || {
                                                            if let Err(e) =
                                                                open::with(&entry_path, editor)
                                                            {
                                                                eprintln!("Failed to open editor: {}", e);
                                                            };
                                                        });
                                                    }
                                                }
                                            }

                                            if button_response.clicked() {
                                                button_response.request_focus();
                                            }

                                            if !self.refresh && !self.loading_done {
                                                self.toasts
                                                    .success(format!(
                                                        "Finished loading {} replay{}",
                                                        replay_count,
                                                        if replay_count > 1 { "s"} else {""}
                                                    ))
                                                    .duration(Duration::from_secs(5));
                                                self.loading_done = true
                                            }

                                            Ok(())
                                        };

                                    match self.display_mode {
                                        DisplayMode::Grid => {
                                            ui.vertical(button_ui);
                                        }
                                        DisplayMode::List => {
                                            ui.horizontal(button_ui);
                                        }
                                    }

                                    if self.delete_popup == Some(i) {
                                        egui::Modal::new(egui::Id::new(i)).show(
                                            ctx,
                                            |ui: &mut egui::Ui| {
                                                ui.set_min_width(310.0);

                                                ui.heading("Delete");
                                                ui.label(format!(
                                                    "Are you sure you want to delete {}?",
                                                    entry.display()
                                                ));
                                                ui.strong(
                                                    "This is permanent and cannot be undone.",
                                                );

                                                ui.horizontal(|ui| -> Result<()> {
                                                    if ui.button("Yes").clicked() {
                                                        match std::fs::remove_file(entry) {
                                                            Ok(()) => {
                                                                self.toasts
                                                                    .success(format!(
                                                                        "Deleted {}",
                                                                        entry
                                                                            .file_stem()
                                                                            .unwrap_or_default()
                                                                            .to_string_lossy()
                                                                    ))
                                                                    .duration(Duration::from_secs(5));
                                                            }
                                                            Err(e) => {
                                                                return Err(anyhow!("Could not delete file {}: {}", entry.display(), e));
                                                            }
                                                        }
							self.refresh = true;
                                                        self.delete_popup = None;
                                                    }
                                                    if ui.button("No").clicked() {
                                                        self.delete_popup = None;
                                                    }
                                                    Ok(())
                                                })
                                            },
                                        );
                                    }
                                    if self.catbox_popup == Some(i) {
                                        egui::Modal::new(egui::Id::new(i)).show(
                                            ctx,
                                            |ui: &mut egui::Ui| {
                                                ui.set_min_width(310.0);

                                                ui.heading("Share");

                                                let litter_time = match self.litterbox_upload_time {
                                                    LitterboxUploadTime::OneHour => "1 hour",
                                                    LitterboxUploadTime::TwelveHours => "12 hours",
                                                    LitterboxUploadTime::OneDay => "1 day",
                                                    LitterboxUploadTime::ThreeDays => "3 days",
                                                };

                                                match &self.catbox_upload_state {
                                                    &CatboxUploadState::Idle => {
                                                        ui.horizontal(|ui| {
                                                            ui.spinner();
                                                        });
                                                        if ui.button("Close").clicked() {
                                                            self.catbox_popup = None;
                                                        }
                                                    }
                                                    &CatboxUploadState::Uploading => {
                                                        ui.horizontal(|ui| {
                                                            ui.spinner();
                                                            ui.label("Sending file to Catbox");
                                                        });
                                                        if ui.button("Cancel").clicked() {
                                                            self.catbox_popup = None;
                                                        }
                                                    }
                                                    CatboxUploadState::Error(error) => {
                                                        ui.colored_label(
                                                            Color32::RED,
                                                            format!("Error: {}", error),
                                                        );
                                                        if ui.button("Cancel").clicked() {
                                                            self.catbox_popup = None;
                                                        }
                                                    }
                                                    CatboxUploadState::Done(link) => {
                                                        ui.label("Upload finished!");
                                                        if self.catbox_litter {
                                                            ui.strong(format!(
                                                                "Your file will be deleted after {}.",
                                                                litter_time
                                                            ));
                                                        }
                                                        if ui.link(format!("{} {}", link, egui_material_icons::icons::ICON_CONTENT_COPY)).clicked() {
                                                            ui.ctx().copy_text(link.clone());
                                                            self.toasts.info("Copied file link to clipboard").duration(Duration::from_secs(2));
                                                        };
                                                        if ui.button("Ok").clicked() {
                                                            self.catbox_popup = None;
                                                        }
                                                    }
                                                }
                                            },
                                        );
                                    }

                                }
                                }
                                ui.end_row();
                            }
                        });
                    });
        });

        self.toasts.show(ctx);
    }
}
