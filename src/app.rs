use crate::{thumbnails, videoutils};
use anyhow::{Error, Result, anyhow};
use catbox;
use eframe::egui::{self, Color32};
use egui_file_dialog::FileDialog;
use egui_notify::Toast;
use glob::glob;
use std::{path::PathBuf, process::Command, sync::mpsc, time::Duration};

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
enum Sorting {
    CreationDate,
    ModificationDate,
    Name,
    Size,
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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ReplayManager {
    replay_folder: Option<PathBuf>,
    replay_format: String,
    replay_prefix: String,

    #[serde(skip)]
    delete_popup: Option<usize>,
    catbox_popup: Option<usize>,
    replays: Vec<PathBuf>,

    #[serde(skip)]
    loading_done: bool,
    sort_order: Sorting,
    ascending: bool,

    #[serde(skip)]
    search_query: String,
    catbox_upload_state: CatboxUploadState,
    #[serde(skip)]
    catbox_upload_recv: mpsc::Receiver<Result<String, String>>,
    #[serde(skip)]
    catbox_upload_send: mpsc::Sender<Result<String, String>>,
    #[serde(skip)]
    settings_popup: bool,
    #[serde(skip)]
    error_modal: bool,
    #[serde(skip)]
    error: Option<Result<(), Error>>,

    #[serde(skip)]
    file_dialog: FileDialog,

    litterbox_upload_time: LitterboxUploadTime,
    catbox_litter: bool,
}

impl Default for ReplayManager {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            replay_folder: Some(PathBuf::from("/home/aredfx/Videos/Replays")),
            replay_format: "mp4".to_string(),
            replay_prefix: "Replay_".to_string(),
            delete_popup: None,
            catbox_popup: None,
            error_modal: false,
            replays: vec![],
            loading_done: false,
            sort_order: Sorting::CreationDate,
            ascending: false,
            search_query: "".to_string(),
            catbox_upload_state: CatboxUploadState::Idle,
            catbox_upload_send: tx,
            catbox_upload_recv: rx,
            settings_popup: false,
            error: None,
            file_dialog: FileDialog::new(),
            litterbox_upload_time: LitterboxUploadTime::ThreeDays,
            catbox_litter: true,
        }
    }
}

impl ReplayManager {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for ReplayManager {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Settings").clicked() {
                        self.settings_popup = true;
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.menu_button("Sort...", |ui| {
                        ui.radio_value(
                            &mut self.sort_order,
                            Sorting::CreationDate,
                            "Creation Date",
                        );
                        ui.radio_value(
                            &mut self.sort_order,
                            Sorting::ModificationDate,
                            "Modification Date",
                        );
                        ui.radio_value(&mut self.sort_order, Sorting::Name, "Name");
                        ui.radio_value(&mut self.sort_order, Sorting::Size, "File Size");
                    });
                    ui.menu_button("Order", |ui| {
                        if ui.radio(self.ascending, "Ascending").clicked() {
                            self.ascending = true;
                        }
                        if ui.radio(!self.ascending, "Descending").clicked() {
                            self.ascending = false;
                        };
                    })
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
            if self.settings_popup {
                let _window = egui::Window::new("Settings")
                    .collapsible(false)
                    .open(&mut self.settings_popup)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_min_width(420.0);
                        ui.heading("Replay settings");
                        ui.horizontal(|ui| {
                            ui.label(
                                "Replay videos folder location (default $HOME/Videos/Replays/): ",
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
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Replay prefix (default: Replay_): ");
                            ui.text_edit_singleline(&mut self.replay_prefix);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Replay format (default: mp4): ");
                            ui.text_edit_singleline(&mut self.replay_format);
                        });

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
                    });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Replay Manager");
            let replay_folder = self.replay_folder.clone().unwrap();

            ui.horizontal(|ui| {
                ui.label(format!("Replays in {}", replay_folder.display()));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);
                if ui.button("Clear").clicked() {
                    self.search_query = "".to_string()
                }
            });

            if !self.replay_folder.is_some() {
                self.error = Some(Err(anyhow!("Replay folder does not exist (is None)")))
            }

            let replays_glob = glob(&format!(
                "{}/**/{}*.{}",
                replay_folder.to_string_lossy(),
                self.replay_prefix,
                self.replay_format
            ));

            if replays_glob.is_ok() {
                self.replays = replays_glob.unwrap().filter_map(|e| e.ok()).collect();
            } else {
                self.error = Some(Err(anyhow!(format!("{}", replays_glob.unwrap_err()))))
            }

            self.replays.sort_by(|a, b| a.cmp(&b));

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

            if let Ok(result) = self.catbox_upload_recv.try_recv() {
                self.catbox_upload_state = match result {
                    Ok(link) => CatboxUploadState::Done(link),
                    Err(err) => CatboxUploadState::Error(err),
                };
                ctx.request_repaint();
            }

            let replay_count = self.replays.iter().count();
            let replay_enumerate = self.replays.iter().enumerate();
            let min_col_width = 160.0;
            let grid_spacing = egui::Vec2::new(
                // ui.available_width() - column_count as f32 * min_col_width,
                10.0, 10.0,
            );
            let ui_minus_spacing = ui.available_width(); //- 2.0*grid_spacing.x - ui.style().spacing.window_margin.left as f32 - ui.style().spacing.window_margin.right as f32 / min_col_width;
            let column_count = (ui_minus_spacing / min_col_width).floor() as usize;

            let mut row_reset = 0;
            let image_size = egui::Vec2::new(160.0, 120.0);

            ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::DARK_GRAY;
            ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::LIGHT_BLUE;
            ui.style_mut().spacing.item_spacing = egui::Vec2::new(8.0, 8.0);

            // let replay_grid =
            let replay_grid = egui::Grid::new("Replays")
                .min_col_width(min_col_width)
                .min_row_height(120.0)
                .spacing(grid_spacing);

            egui::ScrollArea::vertical()
                .max_width(ui.available_width())
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    /*self.infinite_scroll.ui(ui, 10, |ui, _index, item| {
                        ui.label(format!("Item: {item}"));
                    });*/
                    ui.vertical_centered_justified(|ui| {
                        replay_grid.show(ui, |ui| -> Result<()> {
                            for (i, entry) in replay_enumerate {
                                let thumbnail_path_result = thumbnails::create(
                                    &entry,
                                    &format!("{}", replay_folder.display()),
                                    true,
                                    0.0,
                                );
                                let mut thumbnail_path = PathBuf::new();
                                if thumbnail_path_result.is_ok() {
                                    thumbnail_path = thumbnail_path_result.unwrap()
                                } else {
                                    self.error = Some(Err(anyhow!(format!(
                                        "{}",
                                        thumbnail_path_result.unwrap_err()
                                    ))));
                                    self.error_modal = true;
                                }

                                let thumbnail_image = egui::Image::from_uri(format!(
                                    "file://{}",
                                    thumbnail_path.display()
                                ))
                                .fit_to_exact_size(image_size) // original res 640x480
                                .corner_radius(5);

                                let file_stem_opt = entry.file_stem();
                                let file_stem: &std::ffi::OsStr;
                                if file_stem_opt.is_some() {
                                    file_stem = file_stem_opt.unwrap();
                                } else {
                                    self.error = Some(Err(anyhow!(format!("{:?}", file_stem_opt))));
                                    self.error_modal = true;
                                    file_stem = std::ffi::OsStr::new("undefined");
                                }

                                if format!("{}", file_stem.to_string_lossy())
                                    .to_lowercase()
                                    .contains(&self.search_query.to_lowercase())
                                {
                                    let button = egui::Button::image(thumbnail_image)
                                        .min_size(image_size)
                                        .frame_when_inactive(false);
                                    let label = egui::Label::new(format!(
                                        "{}",
                                        file_stem.to_string_lossy(),
                                    ))
                                    .halign(egui::Align::Center);

                                    ui.vertical(|ui| -> Result<()> {
                                        let button_response = ui.add(button);
                                        button_response.context_menu(|ui| {
                                            if ui.button("Edit").clicked() {
                                                let entry_path = format!("{}", entry.display());
                                                match open::with(&entry_path, "losslesscut") {
                                                    Ok(_) => {}
                                                    Err(_) => {
                                                        self.error = Some(Err(anyhow!(format!(
                                                            "Failed to open Losslesscut: {}",
                                                            &entry_path
                                                        ))))
                                                    }
                                                }
                                            }
                                            if ui.button("View").clicked() {
                                                let entry_path = format!("{}", entry.display());
                                                match open::that(&entry_path) {
                                                    Ok(_) => {}
                                                    Err(_) => {
                                                        self.error = Some(Err(anyhow!(format!(
                                                            "Failed to open file: {}",
                                                            &entry_path
                                                        ))))
                                                    }
                                                }
                                            }
                                            if ui.button("Delete").clicked() {
                                                self.delete_popup = Some(i);
                                            }
                                            if ui.button("Save to Catbox").clicked() {
                                                self.catbox_upload_state =
                                                    CatboxUploadState::Uploading;

                                                let entry_path = format!("{}", entry.display());
                                                let tx = self.catbox_upload_send.clone();
                                                let ctx = ctx.clone();

                                                if self.catbox_litter {
                                                    match self.litterbox_upload_time {
                                                        LitterboxUploadTime::OneHour => {
                                                            std::thread::spawn(move || {
                                                                let rt =
                                                                    tokio::runtime::Runtime::new()
                                                                        .unwrap();

                                                                let result = rt.block_on(async {
                                                                    catbox::litter::upload(
                                                                        entry_path, 1,
                                                                    )
                                                                    .await
                                                                });
                                                                let _ = tx
                                                                    .send(result.map_err(|e| {
                                                                        e.to_string()
                                                                    }));
                                                                ctx.request_repaint();
                                                            });
                                                        }
                                                        LitterboxUploadTime::TwelveHours => {
                                                            std::thread::spawn(move || {
                                                                let rt =
                                                                    tokio::runtime::Runtime::new()
                                                                        .unwrap();

                                                                let result = rt.block_on(async {
                                                                    catbox::litter::upload(
                                                                        entry_path, 12,
                                                                    )
                                                                    .await
                                                                });
                                                                let _ = tx
                                                                    .send(result.map_err(|e| {
                                                                        e.to_string()
                                                                    }));
                                                                ctx.request_repaint();
                                                            });
                                                        }
                                                        LitterboxUploadTime::OneDay => {
                                                            std::thread::spawn(move || {
                                                                let rt =
                                                                    tokio::runtime::Runtime::new()
                                                                        .unwrap();

                                                                let result = rt.block_on(async {
                                                                    catbox::litter::upload(
                                                                        entry_path, 24,
                                                                    )
                                                                    .await
                                                                });
                                                                let _ = tx
                                                                    .send(result.map_err(|e| {
                                                                        e.to_string()
                                                                    }));
                                                                ctx.request_repaint();
                                                            });
                                                        }
                                                        LitterboxUploadTime::ThreeDays => {
                                                            std::thread::spawn(move || {
                                                                let rt =
                                                                    tokio::runtime::Runtime::new()
                                                                        .unwrap();

                                                                let result = rt.block_on(async {
                                                                    catbox::litter::upload(
                                                                        entry_path, 72,
                                                                    )
                                                                    .await
                                                                });
                                                                let _ = tx
                                                                    .send(result.map_err(|e| {
                                                                        e.to_string()
                                                                    }));
                                                                ctx.request_repaint();
                                                            });
                                                        }
                                                    }
                                                } else {
                                                    std::thread::spawn(move || {
                                                        let rt =
                                                            tokio::runtime::Runtime::new().unwrap();

                                                        let result = rt.block_on(async {
                                                            catbox::file::from_file(
                                                                entry_path, None,
                                                            )
                                                            .await
                                                        });

                                                        let _ = tx.send(
                                                            result.map_err(|e| e.to_string()),
                                                        );
                                                        ctx.request_repaint();
                                                    });
                                                }

                                                self.catbox_popup = Some(i);
                                            }
                                        });
                                        if button_response.double_clicked() {
                                            let entry_path = format!("{}", entry.display());
                                            match open::that(&entry_path) {
                                                Ok(_) => {}
                                                Err(_) => {
                                                    self.error = Some(Err(anyhow!(format!(
                                                        "Failed to open file: {}",
                                                        &entry_path
                                                    ))))
                                                }
                                            }
                                        }

                                        if button_response.clicked() {
                                            button_response.request_focus();
                                        }

                                        if i == replay_count - 1 && !self.loading_done {
                                            //button_response.scroll_to_me(Some(egui::Align::BOTTOM));
                                            self.loading_done = true
                                        }

                                        ui.add(label);

                                        Ok(())
                                    });

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
                                                ui.strong("This cannot be undone.");

                                                ui.horizontal(|ui| {
                                                    if ui.button("Yes").clicked() {
                                                        let _ = Command::new("rm")
                                                            .arg("-rf")
                                                            .arg(format!("{}", entry.display()))
                                                            .output();
                                                        self.delete_popup = None;
                                                    }
                                                    if ui.button("No").clicked() {
                                                        self.delete_popup = None;
                                                    }
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
                                                    }
                                                    &CatboxUploadState::Uploading => {
                                                        ui.horizontal(|ui| {
                                                            ui.spinner();
                                                            ui.label("Sending file to Catbox");
                                                        });
                                                    }
                                                    CatboxUploadState::Error(error) => {
                                                        ui.colored_label(
                                                            Color32::RED,
                                                            format!("Error: {}", error),
                                                        );
                                                    }
                                                    CatboxUploadState::Done(link) => {
                                                        ui.label("Upload finished!");
                                                        if self.catbox_litter {

                                                            ui.strong(format!(
                                                                                                                        "Your file will be deleted after {}",
                                                                                                                        litter_time
                                                                                                                    ));
                                                        }
                                                        ui.horizontal(|ui| {
                                                            ui.hyperlink(link);
                                                            if ui.button("Copy").clicked() {
                                                                ui.ctx().copy_text(link.clone());
                                                            }
                                                        });
                                                    }
                                                }
                                                ui.horizontal(|ui| {
                                                    if ui.button("Ok").clicked() {
                                                        self.catbox_popup = None;
                                                    }
                                                });
                                            },
                                        );
                                    }
                                    if self.error.is_some() {
                                        egui::Modal::new(egui::Id::new(i)).show(
                                            ctx,
                                            |ui: &mut egui::Ui| {
                                                ui.set_min_width(310.0);

                                                ui.heading("Error");

                                                ui.colored_label(
                                                    Color32::RED,
                                                    // [TODO] Implement error message display
                                                    "Error",
                                                );

                                                ui.horizontal(|ui| {
                                                    if ui.button("Ok").clicked() {
                                                        self.error = None;
                                                    }
                                                });
                                            },
                                        );
                                    }

                                    row_reset += 1;
                                }

                                if row_reset == column_count {
                                    row_reset = 0;
                                    ui.end_row();
                                }
                            }
                            Ok(())
                        });
                    });
                });
        });
    }
}
