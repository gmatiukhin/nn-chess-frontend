#![warn(clippy::all, rust_2018_idioms)]

use std::fmt::Display;

use chess::{AiGameSettings, GameMode};
use shakmaty::Color;
use tokio::sync::mpsc;

use anyhow::Result;
use egui::{Align2, Button, Grid, Image, ImageButton, Label};
use requests::RequestLoopComm;
use web_types::{EngineDescription, EngineDirectory, EngineRef, EngineVariant};

mod chess;
mod requests;

pub struct App {
    chessboard: chess::ChessBoard,
    game_mode_selection: GameModeSelector,
    fetch_engine_list_first_boot: bool,
    engine_data: EngineData,
    request_loop_sender: mpsc::Sender<requests::RequestLoopComm>,
    engine_dir_receiver: Option<oneshot::Receiver<Result<EngineDirectory>>>,
    engine_desc_receiver: Option<oneshot::Receiver<Result<EngineDescription>>>,
}

#[derive(PartialEq, Eq)]
enum GameModeSelector {
    PlayAgainsAI,
    PlayAgainsYourself,
}
impl Display for GameModeSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameModeSelector::PlayAgainsAI => write!(f, "Play against AI"),
            GameModeSelector::PlayAgainsYourself => write!(f, "Play against Yourself"),
        }
    }
}

#[derive(Default)]
struct EngineData {
    available_engines: Option<EngineDirectory>,
    selected_engine: Option<EngineRef>,
    desc: Option<EngineDescription>,
    variant: Option<EngineVariant>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let req_comm_loop = requests::run_request_loop(cc.egui_ctx.clone());

        Self {
            chessboard: Default::default(),
            game_mode_selection: GameModeSelector::PlayAgainsAI,
            fetch_engine_list_first_boot: true,
            engine_data: EngineData::default(),
            request_loop_sender: req_comm_loop,
            engine_desc_receiver: None,
            engine_dir_receiver: None,
        }
    }
}

impl App {
    fn update_top_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            _frame.close();
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });
    }

    fn update_bottom_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }

    fn fetch_engine_description(&mut self) {
        if let Some(selected_engine) = &self.engine_data.selected_engine {
            let (sender, receiver) = oneshot::channel();
            let req = RequestLoopComm::FetchEngineDescription(selected_engine.clone(), sender);
            self.request_loop_sender
                .try_send(req)
                .expect("Error communicating with request loop");
            self.engine_desc_receiver = Some(receiver);
        }
    }

    fn fetch_engine_dir(&mut self) {
        // Build a request to the request loop.
        let (sender, receiver) = oneshot::channel();
        let req = RequestLoopComm::FetchEngines(sender);
        self.request_loop_sender
            .try_send(req)
            .expect("Error communicating with request loop");

        self.engine_dir_receiver = Some(receiver);
    }

    fn update_right_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("engine_info").show(ctx, |ui| {
            ui.heading("Game settings");

            egui::ComboBox::from_id_source("gamemode_selection")
                .width(140f32)
                .selected_text(format!("{}", self.game_mode_selection))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            &mut self.game_mode_selection,
                            GameModeSelector::PlayAgainsAI,
                            format!("{}", GameModeSelector::PlayAgainsAI),
                        )
                        .clicked()
                    {
                        self.chessboard.stop_game()
                    }
                    if ui
                        .selectable_value(
                            &mut self.game_mode_selection,
                            GameModeSelector::PlayAgainsYourself,
                            format!("{}", GameModeSelector::PlayAgainsYourself),
                        )
                        .clicked()
                    {
                        self.chessboard.stop_game()
                    }
                });

            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            ImageButton::new(
                                Image::new(egui::include_image!("../assets/wk.svg"))
                                    .maintain_aspect_ratio(true)
                                    .fit_to_exact_size([40f32, 40f32].into()),
                            )
                            .selected(self.chessboard.player_color == Color::White),
                        )
                        .clicked()
                    {
                        self.chessboard.player_color = Color::White;
                    }
                    if ui
                        .add(
                            ImageButton::new(
                                Image::new(egui::include_image!("../assets/bk.svg"))
                                    .maintain_aspect_ratio(true)
                                    .fit_to_exact_size([40f32, 40f32].into()),
                            )
                            .selected(self.chessboard.player_color == Color::Black),
                        )
                        .clicked()
                    {
                        self.chessboard.player_color = Color::Black;
                    }
                })
            });

            if self.game_mode_selection == GameModeSelector::PlayAgainsYourself {
                self.chessboard.game_mode = GameMode::PlayAgainsYourself;
            } else {
                ui.heading("Select engine");
                if ui.button("Update info").clicked() || self.fetch_engine_list_first_boot {
                    self.engine_data.available_engines = None;
                    self.fetch_engine_dir();
                    self.fetch_engine_list_first_boot = false;
                }

                if let Some(recv) = &self.engine_dir_receiver {
                    if let Ok(Ok(engines)) = recv.try_recv() {
                        self.engine_data.available_engines = Some(engines.clone());
                        self.engine_data.selected_engine = Some(engines.engines[0].clone());
                        self.engine_dir_receiver = None;
                    } else {
                        ui.label("Loading engine list...");
                        ui.spinner();
                    }
                }
                if let Some(data) = self.engine_data.selected_engine.as_mut() {
                    let cbox_resp = egui::ComboBox::from_id_source("engine_selection")
                        .selected_text(data.name.to_string())
                        .show_ui(ui, |ui| {
                            let mut is_clicked = false;
                            for engine in self
                                .engine_data
                                .available_engines
                                .as_mut()
                                .unwrap()
                                .engines
                                .iter()
                            {
                                if ui
                                    .selectable_value(data, engine.clone(), engine.name.clone())
                                    .clicked()
                                {
                                    is_clicked = true;
                                }
                            }
                            is_clicked
                        });
                    let selected_engine = data.clone();

                    if cbox_resp.inner.is_some_and(|v| v) {
                        log::info!("Engine changed to: {data:?}");
                        self.chessboard.stop_game();
                        self.engine_data.variant = None;
                        self.engine_data.desc = None;
                        self.fetch_engine_description();
                    }

                    Grid::new("current_engine_info").show(ui, |ui| {
                        ui.label("Name");
                        ui.label(selected_engine.name);
                        ui.end_row();
                        ui.label("Id");
                        ui.label(selected_engine.engine_id);
                        ui.end_row();
                        ui.label("URL");
                        ui.hyperlink(selected_engine.entrypoint_url);
                        ui.end_row();
                    });
                }

                if self.engine_data.selected_engine.is_some() {
                    ui.heading("Select variant");
                    if ui.button("Update info").clicked() {
                        self.fetch_engine_description();
                    }
                    if self.engine_data.desc.is_none() && self.engine_desc_receiver.is_none() {
                        self.fetch_engine_description();
                    }
                    if let Some(recv) = &self.engine_desc_receiver {
                        if let Ok(Ok(desc)) = recv.try_recv() {
                            log::info!("Received engine description: {desc:?}");
                            self.engine_data.desc = Some(desc.clone());
                            self.engine_data.variant = None;
                            self.engine_desc_receiver = None;
                        } else {
                            ui.label("Loading engine description...");
                            ui.spinner();
                        }
                    }
                    if let Some(desc) = &mut self.engine_data.desc {
                        ui.heading(desc.name.clone());
                        ui.add(Label::new(desc.text_description.clone()).wrap(true));

                        if self.engine_data.variant.is_none() {
                            self.engine_data.variant = Some(desc.best_available_variant.clone());
                        }

                        let mut checkpoint = self.engine_data.variant.as_ref().unwrap().clone();
                        if egui::ComboBox::from_id_source("variant_selection")
                            .selected_text(checkpoint.name.to_string())
                            .show_ui(ui, |ui| {
                                let mut is_clicked = false;
                                for variant in &desc.variants {
                                    is_clicked |= ui
                                        .selectable_value(
                                            &mut checkpoint,
                                            variant.clone(),
                                            variant.name.clone(),
                                        )
                                        .clicked();
                                }
                                is_clicked
                            })
                            .inner
                            .is_some_and(|v| v)
                        {
                            log::info!("Changed variant: new is {checkpoint:?}");
                            self.engine_data.variant = Some(checkpoint);
                            self.chessboard.stop_game()
                        }
                    }
                }
            }
            match self.game_mode_selection {
                GameModeSelector::PlayAgainsAI => {
                    if let Some(variant) = &self.engine_data.variant {
                        if ui.button("Play vs AI").clicked() {
                            log::info!("Starting AI game!");
                            self.chessboard.game_mode =
                                GameMode::PlayAgainsAI(AiGameSettings::new(
                                    variant.clone(),
                                    self.request_loop_sender.clone(),
                                ));
                            self.chessboard.start_game();
                        }
                    } else {
                        ui.add_enabled(false, Button::new("Play vs AI"))
                            .on_disabled_hover_text("Select an engine and variant first!");
                    }
                }
                GameModeSelector::PlayAgainsYourself => {
                    if ui.button("Start game").clicked() {
                        log::info!("Starting self game!");
                        self.chessboard.game_mode = GameMode::PlayAgainsYourself;
                        self.chessboard.start_game();
                    }
                }
            }

            ui.separator();

            if self.chessboard.is_waiting_for_ai_move() {
                ui.label("Waiting for server's move...");
                ui.spinner();
            }
            if let Some(status) = self.chessboard.last_ai_move_info() {
                Grid::new("ai_move_table").show(ui, |ui| {
                    ui.heading("Latest AI move");
                    ui.end_row();
                    ui.label("Notation");
                    ui.label(status.move_san);
                    ui.end_row();
                    ui.label("Time taken for computation");
                    ui.label(format!("{:?}", status.move_timing));
                    ui.end_row();
                    ui.label("Info");
                    ui.label(status.status_text);
                    ui.end_row();
                });
            }
        });
    }

    fn update_central_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Unchessful Games");
            self.chessboard.update_ai_move();
            egui::Area::new("board_area")
                .anchor(Align2::CENTER_CENTER, [0f32, 0f32])
                .movable(false)
                .show(ctx, |ui| {
                    self.chessboard.show(ctx, ui);
                });
        });
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_top_panel(ctx, _frame);
        self.update_bottom_panel(ctx, _frame);
        self.update_right_panel(ctx, _frame);
        self.update_central_panel(ctx, _frame);
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
