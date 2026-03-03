use crate::thumbnails;
use crate::videoutils;
use anyhow::Result;
use catbox;
use eframe::egui::{self, Color32};
use glob::glob;
use rayon;
use std::sync::mpsc;
use std::{path::PathBuf, process::Command};

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
enum Sorting {
    CreationDate,
    ModificationDate,
    Name,
    Size,
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
    replay_folder: PathBuf,
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
}

impl Default for ReplayManager {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            replay_folder: PathBuf::from("/home/aredfx/Videos/Replays"),
            replay_format: "mp4".to_string(),
            replay_prefix: "Replay_".to_string(),
            delete_popup: None,
            catbox_popup: None,
            replays: vec![],
            loading_done: false,
            sort_order: Sorting::CreationDate,
            ascending: false,
            search_query: "".to_string(),
            catbox_upload_state: CatboxUploadState::Idle,
            catbox_upload_send: tx,
            catbox_upload_recv: rx,
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
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Replay Manager");

            ui.horizontal(|ui| {
                ui.label(format!("Replays in {}", self.replay_folder.display()));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);
                if ui.button("Clear").clicked() {
                    self.search_query = "".to_string()
                }
            });

            let replays_glob = glob(&format!(
                "{}/**/{}*.{}",
                self.replay_folder.to_string_lossy(),
                self.replay_prefix,
                self.replay_format
            ));

            self.replays = replays_glob
                .expect("Could not get replays &Paths iterator")
                .filter_map(|e| e.ok())
                .collect();

            self.replays.sort_by(|a, b| a.cmp(&b));

            match self.sort_order {
                Sorting::CreationDate => self.replays.sort_by(|a, b| {
                    videoutils::get_creation_date(a)
                        .into_iter()
                        .cmp(videoutils::get_creation_date(b))
                }),
                Sorting::Name => self
                    .replays
                    .sort_by(|a, b| videoutils::get_name(a).cmp(&videoutils::get_name(b))),
                Sorting::ModificationDate => self.replays.sort_by(|a, b| {
                    videoutils::get_mod_date(a)
                        .into_iter()
                        .cmp(videoutils::get_mod_date(b))
                }),
                Sorting::Size => self
                    .replays
                    .sort_by(|a, b| videoutils::get_size(a).cmp(&videoutils::get_size(b))),
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
            let column_count = 3;
            let mut row_reset = 0;

            ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::DARK_GRAY;
            ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::LIGHT_BLUE;

            let replay_grid = egui::Grid::new("Replays")
                .min_col_width(320.0)
                .min_row_height(240.0);

            egui::ScrollArea::both().show(ui, |ui| {
                replay_grid.show(ui, |ui| -> Result<()> {
                    for (i, entry) in replay_enumerate {
                        let thumbnail_path = rayon::scope(|_| {
                            return thumbnails::create(
                                &entry,
                                &format!("{}", self.replay_folder.display()),
                                true,
                                0.0,
                            )
                            .expect("Failed to get thumbnail");
                        });
                        let thumbnail_image =
                            egui::Image::from_uri(format!("file://{}", thumbnail_path.display()))
                                .fit_to_exact_size(egui::Vec2::new(300.0, 225.0)) // original res 640x480
                                .corner_radius(5);

                        if format!(
                            "{}",
                            entry
                                .file_stem()
                                .expect("Could not get file stem")
                                .to_string_lossy()
                        )
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                        {
                            let button = egui::Button::image(thumbnail_image)
                                .min_size(egui::Vec2::new(300.0, 225.0))
                                .frame_when_inactive(false);
                            let label = egui::Label::new(format!(
                                "{}",
                                &entry
                                    .file_name()
                                    .expect("Could not use file name")
                                    .to_string_lossy(),
                            ))
                            .halign(egui::Align::Center);

                            ui.vertical(|ui| -> Result<()> {
                                let button_response = ui.add(button);
                                button_response.context_menu(|ui| {
                                    if ui.button("Edit").clicked() {
                                        let _ = Command::new("losslesscut")
                                            .arg(format!("{}", entry.display()))
                                            .output()
                                            .expect("Failed to open losslesscut");
                                    }
                                    if ui.button("View").clicked() {
                                        let _ = Command::new("xdg-open")
                                            .arg(format!("{}", entry.display()))
                                            .output()
                                            .expect("Failed to open video viewer");
                                    }
                                    if ui.button("Delete").clicked() {
                                        self.delete_popup = Some(i);
                                    }
                                    if ui.button("Save to Catbox").clicked() {
                                        self.catbox_upload_state = CatboxUploadState::Uploading;

                                        let entry_path = format!("{}", entry.display());
                                        let tx = self.catbox_upload_send.clone();
                                        let ctx = ctx.clone();

                                        std::thread::spawn(move || {
                                            let rt = tokio::runtime::Runtime::new().unwrap();

                                            let result = rt.block_on(async {
                                                catbox::litter::upload(entry_path, 72).await
                                            });
                                            let _ = tx.send(result.map_err(|e| e.to_string()));
                                            ctx.request_repaint();
                                        });

                                        self.catbox_popup = Some(i);
                                    }
                                });
                                if button_response.double_clicked() {
                                    let _ = Command::new("xdg-open")
                                        .arg(format!("{}", entry.display()))
                                        .output()
                                        .expect("Failed to open video viewer");
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
                                                ui.strong(
                                                    "Your file will be deleted after 3 days.",
                                                );
                                                ui.hyperlink(link);
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
    }
}
